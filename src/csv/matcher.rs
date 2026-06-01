use tracing::{debug, error};

use crate::documents::{EML, candidate_lists::CandidateLists, election_count::ElectionCount};

/// Error during matching of EML documents for CSV processing using [`find_matching_documents`]
#[derive(thiserror::Error, Debug)]
pub enum EMLMatchError {
    /// EML-510 counts document found, but missing EML-230b candidates document
    #[error("EML-510 counts document found, but missing EML-230b candidate lists document")]
    NoValidCandidatesDocument,
    /// EML-230b candidates document found, but missing EML-510 counts document
    #[error("EML-230b candidates document found, but missing EML-510 counts document")]
    NoValidCountsDocument,
    /// Missing EML-510 counts document and missing EML-230b candidates document
    #[error("Missing EML-510 counts document and missing EML-230b candidate lists document")]
    NoValidDocuments,
    /// Election ids of provided documents do not match
    #[error(
        "Election id of counts file ('{counts_id}') does not match candidate lists id ('{candidates_id}')"
    )]
    MismatchedElectionIds {
        /// Election id in the counts document
        counts_id: String,
        /// Election id in the candidate lists document
        candidates_id: String,
    },
    /// Missing contest in the EML-510 counts document
    #[error("Missing contest in the EML-510 counts document")]
    MissingContestInCountsDocument,
    /// Missing contest in the EML-230b candidate lists document
    #[error("Missing contest element in the EML-230b candidate lists document")]
    MissingContestInCandidateDocument,
    /// Contest ids of provided documents do not match
    #[error(
        "Contest id of counts file ('{counts_id}') does not match candidate lists id ('{candidates_id}')"
    )]
    MismatchedContestIds {
        /// Contest identifier in the counts document
        counts_id: String,
        /// Contest identifier in the candidate lists document
        candidates_id: String,
    },
}

/// Given two parsed EML files, find out which one is which.
///
/// This ensures that the two documents match and returns references back to
/// the [`ElectionCount`] and [`CandidateLists`] document. It does not matter in
/// what order the documents are passed, but this function checks that they are
/// about the same election.
pub fn find_matching_documents<'a>(
    first_xml: &'a EML,
    second_xml: &'a EML,
) -> Result<(&'a ElectionCount, &'a CandidateLists), EMLMatchError> {
    // Determine which file is counts and which is candidates based on document type
    let (counts, candidates) = if let Some(election_count) = first_xml.as_count_doc() {
        debug!("First file is identified as counts document");
        if let Some(candidate_lists) = second_xml.as_candidate_lists_doc() {
            debug!("Second file is identified as candidates document");
            (election_count, candidate_lists)
        } else {
            error!("Second file does not contain a valid EML-230b candidates document");
            return Err(EMLMatchError::NoValidCandidatesDocument);
        }
    } else if let Some(candidate_lists) = first_xml.as_candidate_lists_doc() {
        debug!("First file is identified as candidates document");
        if let Some(election_count) = second_xml.as_count_doc() {
            debug!("Second file is identified as counts document");
            (election_count, candidate_lists)
        } else {
            error!("Second file does not contain a valid EML-510 counts document");
            return Err(EMLMatchError::NoValidCountsDocument);
        }
    } else {
        error!("Neither file provided contains a valid counts or candidates document");
        return Err(EMLMatchError::NoValidDocuments);
    };

    // Make sure both files are talking about the same election
    let counts_election_id = &counts.count.election.identifier.id;
    let candidates_election_id = &candidates.candidate_list.election.identifier.id;
    if counts_election_id != candidates_election_id {
        error!("Failed to match election ids of documents provided");
        return Err(EMLMatchError::MismatchedElectionIds {
            counts_id: counts_election_id.raw().into_owned(),
            candidates_id: candidates_election_id.raw().into_owned(),
        });
    }

    // Extract the contests from both files
    let count_contest = counts
        .count
        .election
        .contests
        .first()
        .ok_or(EMLMatchError::MissingContestInCountsDocument)?;
    let candidates_contest = candidates
        .candidate_list
        .election
        .contests
        .first()
        .ok_or(EMLMatchError::MissingContestInCandidateDocument)?;

    // Make sure both files are talking about the same contest
    if count_contest.identifier.id.raw() != candidates_contest.identifier.id.raw() {
        error!("Failed to match contest ids of documents provided");
        return Err(EMLMatchError::MismatchedContestIds {
            counts_id: count_contest.identifier.id.raw().into_owned(),
            candidates_id: candidates_contest.identifier.id.raw().into_owned(),
        });
    }

    Ok((counts, candidates))
}
