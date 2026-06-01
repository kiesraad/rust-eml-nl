use std::{
    collections::{BTreeMap, btree_map::Entry},
    sync::LazyLock,
};

use regex::Regex;

use crate::{
    EMLError, EMLErrorKind,
    csv::NameResolver,
    documents::election_count::{
        CountType, ElectionCountContest, ElectionCountSelectionType, ReportingUnitVotes, TotalVotes,
    },
    utils::{AffiliationId, CandidateId, StringValueData as _},
};

/// Regular expression to extract postal code from polling station
static POSTCODE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\(postcode: (\d{4}\s*\w{2})\)").expect("Failed to compile postcode regex")
});

/// Regular expression to clean stembureau id prefixes
static SB_ID_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d+::SB").expect("Failed to compile stembureau id regex"));

/// Resolved polling station information for the CSV output
pub struct CsvPollingStation<'a> {
    /// The cleaned name of the polling station, with prefixes and postal code removed
    pub cleaned_name: String,
    /// The cleaned polling station id, with prefixes removed (only for GSB counts)
    pub plain_ps_id: String,
    /// The postal code of the polling station (only for GSB counts)
    pub postal_code: String,
    /// A reference to the original ReportingUnitVotes struct for this polling station
    pub reporting_unit: &'a ReportingUnitVotes,
}

/// Extract polling station information from the count contest, including cleaned names and postal codes.
pub fn extract_polling_stations(
    count_type: CountType,
    count_contest: &ElectionCountContest,
) -> Result<Vec<CsvPollingStation<'_>>, EMLError> {
    count_contest
        .reporting_unit_votes
        .iter()
        .map(|ru| {
            let raw_name = &ru.identifier.name;
            let id = ru.identifier.id.cloned_value()?;
            let raw_id = id.to_raw_value();

            let (plain_ps_id, postal_code, cleaned_name) = if count_type == CountType::Municipal {
                // for GSB counts we do some preprocessing of the data
                let plain_ps_id = SB_ID_RE.replace(&raw_id, "").to_string();
                let postal_code = POSTCODE_RE
                    .captures(raw_name)
                    .map(|c| c[1].to_string())
                    .unwrap_or_default();
                let name = POSTCODE_RE.replace(raw_name, "").to_string();
                let cleaned_name = name
                    .strip_prefix("Stembureau ")
                    .or_else(|| name.strip_prefix("Briefstembureau "))
                    .unwrap_or(&name)
                    .trim()
                    .to_string();
                (plain_ps_id, postal_code, cleaned_name)
            } else {
                // Other counts: no preprocssing of data
                (raw_id.as_ref().into(), "".into(), raw_name.as_ref().into())
            };

            Ok(CsvPollingStation {
                cleaned_name,
                plain_ps_id,
                postal_code,
                reporting_unit: ru,
            })
        })
        .collect()
}

pub struct AffiliationWithVotes {
    pub id: AffiliationId,
    pub name: String,
    pub total_votes: u64,
    pub reporting_unit_votes: Vec<u64>,
    pub candidates: BTreeMap<CandidateId, CandidateWithVotes>,
}

pub struct CandidateWithVotes {
    pub id: CandidateId,
    pub name: String,
    pub total_votes: u64,
    pub reporting_unit_votes: Vec<u64>,
}

/// Extract vote counts for each affiliation and candidate
/// from the TotalVotes and Selections in the document.
///
/// The names for each affiliation and candidate are resolved using the provided
/// NameResolver.
///
/// To construct the CsvPollingStation structs you can use the
/// [`extract_polling_stations`] function in this module.
pub fn extract_vote_counts(
    name_resolver: &impl NameResolver,
    total_votes: &TotalVotes,
    stations: &[CsvPollingStation],
) -> Result<BTreeMap<AffiliationId, AffiliationWithVotes>, EMLError> {
    // first gather total votes per affiliation and candidate from the
    // TotalVotes part of the document
    let mut results = gather_total_votes(name_resolver, total_votes)?;

    // Next: iterate over all the polling stations (i.e. reporting units) in the doc
    for station in stations {
        let mut current_affiliation = None;
        let mut current_aff_entry = None;
        for selection in station.reporting_unit.selections.iter() {
            match &selection.selection_type {
                // Affiliation found: set the current affiliation active and add selection votes for this polling station
                ElectionCountSelectionType::Affiliation(aff) => {
                    let aff_id = aff.id.copied_value()?;
                    current_affiliation = Some(aff_id);

                    let Entry::Occupied(mut entry) = results.entry(aff_id) else {
                        return Err(EMLErrorKind::UnknownAffiliation(aff_id).without_span());
                    };
                    entry
                        .get_mut()
                        .reporting_unit_votes
                        .push(selection.valid_votes.copied_value()?);
                    current_aff_entry = Some(entry);
                }
                // Candidate found, and there is an active affiliation, set votes for candidate in affiliation
                ElectionCountSelectionType::Candidate(cand)
                    if let Some(aff_id) = current_affiliation
                        && let Some(aff_entry) = &mut current_aff_entry =>
                {
                    let cand_id = cand.identifier.id.copied_value()?;
                    let Entry::Occupied(mut entry) = aff_entry.get_mut().candidates.entry(cand_id)
                    else {
                        return Err(EMLErrorKind::UnknownCandidate(aff_id, cand_id).without_span());
                    };
                    entry
                        .get_mut()
                        .reporting_unit_votes
                        .push(selection.valid_votes.copied_value()?);
                }
                // Candidate found, but no active affiliation, this is an error
                ElectionCountSelectionType::Candidate(_) => {
                    return Err(EMLErrorKind::CandidateWithoutAffiliationFound.without_span());
                }
                // Referendum options not supported by this method
                ElectionCountSelectionType::ReferendumOption(_) => {
                    return Err(EMLErrorKind::UnexpectedReferendumOptionSelection.without_span());
                }
            }
        }
    }

    Ok(results)
}

fn gather_total_votes(
    name_resolver: &impl NameResolver,
    total_votes: &TotalVotes,
) -> Result<BTreeMap<AffiliationId, AffiliationWithVotes>, EMLError> {
    let mut results = BTreeMap::new();
    for aff in total_votes.selections_per_affiliation()? {
        let aff_id = aff.affiliation.id.copied_value()?;
        let aff_name = name_resolver
            .resolve_affiliation_name(aff_id)
            .ok_or_else(|| EMLErrorKind::UnknownAffiliation(aff_id).without_span())?;
        let candidates = aff
            .candidates
            .iter()
            .map(|cand| {
                let cand_id = cand.candidate.identifier.id.copied_value()?;
                let cand_name = name_resolver
                    .resolve_candidate_name(aff_id, cand_id)
                    .ok_or_else(|| {
                        EMLErrorKind::UnknownCandidate(aff_id, cand_id).without_span()
                    })?;
                Ok((
                    cand_id,
                    CandidateWithVotes {
                        id: cand_id,
                        name: cand_name,
                        total_votes: cand.valid_votes,
                        reporting_unit_votes: vec![],
                    },
                ))
            })
            .collect::<Result<BTreeMap<_, _>, EMLError>>()?;
        results.insert(
            aff_id,
            AffiliationWithVotes {
                id: aff_id,
                name: aff_name,
                total_votes: aff.valid_votes,
                reporting_unit_votes: vec![],
                candidates,
            },
        );
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postcode_re_compiles() {
        LazyLock::force(&POSTCODE_RE);
    }

    #[test]
    fn test_sb_id_re_compiles() {
        LazyLock::force(&SB_ID_RE);
    }
}
