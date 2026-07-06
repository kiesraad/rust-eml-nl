use crate::{
    documents::candidate_lists::{CandidateLists, CandidateListsCandidate, CandidateListsContest},
    utils::{AffiliationId, CandidateId},
};

/// A trait for resolving the names of candidates and affiliations based on their ids.
///
/// The OSV4-3 format includes names of candidates and affiliations, whereas the
/// ElectionCount document does not include those. To resolve these, the eml2csv
/// utility uses a CandidateLists document. This trait allows external libraries
/// to use alternative implementations for retrieving these details.
pub trait NameResolver {
    /// Resolve the name for an affiliation given its id.
    /// Returns None if the affiliation id is not found.
    fn resolve_affiliation_name(&self, affiliation_id: AffiliationId) -> Option<String>;

    /// Retrieve the name for a candidate from a specific affiliation, given their ids.
    /// Returns None if the candidate/affiliation combination is not found.
    ///
    /// The name should follow the format `{prefix} {last name}, {initials}`.
    fn resolve_candidate_name(
        &self,
        affiliation_id: AffiliationId,
        candidate_id: CandidateId,
    ) -> Option<String>;
}

/// A simple implementation of NameResolver that uses the first contest in the CandidateLists document to resolve names.
impl NameResolver for CandidateLists {
    fn resolve_affiliation_name(&self, affiliation_id: AffiliationId) -> Option<String> {
        self.candidate_list
            .election
            .contests
            .first()?
            .resolve_affiliation_name(affiliation_id)
    }

    fn resolve_candidate_name(
        &self,
        affiliation_id: AffiliationId,
        candidate_id: CandidateId,
    ) -> Option<String> {
        self.candidate_list
            .election
            .contests
            .first()?
            .resolve_candidate_name(affiliation_id, candidate_id)
    }
}

/// A NameResolver that uses the candidates and affiliations in the CandidateListsContest element to resolve names.
impl NameResolver for CandidateListsContest {
    fn resolve_affiliation_name(&self, affiliation_id: AffiliationId) -> Option<String> {
        Some(
            self.affiliations
                .iter()
                .find(|a| a.identifier.id.copied_value().ok() == Some(affiliation_id))?
                .identifier
                .registered_name
                .as_deref()
                // If registered_name is None, this is considered an empty string for the purposes of name resolution
                .unwrap_or("")
                .to_string(),
        )
    }

    fn resolve_candidate_name(
        &self,
        affiliation_id: AffiliationId,
        candidate_id: CandidateId,
    ) -> Option<String> {
        self.affiliations
            .iter()
            .find(|a| a.identifier.id.copied_value().ok() == Some(affiliation_id))
            .and_then(|a| {
                Some(format_candidate(a.candidates.iter().find(|c| {
                    c.identifier.id.copied_value().ok() == Some(candidate_id)
                })?))
            })
    }
}

/// Format a CandidateListsCandidate into a string representation of their name.
fn format_candidate(candidate: &CandidateListsCandidate) -> String {
    let pn = &candidate.full_name.person_name;
    let last = pn.get_last_name();
    let prefix = pn.get_name_prefix();
    let initials = pn.get_initials();

    let last_part = if let Some(prefix) = prefix {
        format!("{} {}", prefix, last)
    } else {
        last.to_string()
    };

    if let Some(initials) = initials {
        format!("{}, {}", last_part, initials)
    } else {
        last_part
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        common::PersonName,
        io::{EMLParsingMode, EMLRead as _},
    };

    use super::*;

    #[test]
    fn test_format_candidate() {
        use crate::documents::candidate_lists::CandidateListsCandidate;

        let candidate = CandidateListsCandidate::builder()
            .identifier(CandidateId::from_u64(1).unwrap())
            .full_name(
                PersonName::new("Jansen")
                    .with_first_name("Jan")
                    .with_initials("J.A.")
                    .with_name_prefix("van"),
            )
            .build()
            .unwrap();
        assert_eq!(format_candidate(&candidate), "van Jansen, J.A.");

        let candidate = CandidateListsCandidate::builder()
            .identifier(CandidateId::from_u64(1).unwrap())
            .full_name(PersonName::new("Tester").with_first_name("Test"))
            .build()
            .unwrap();
        assert_eq!(format_candidate(&candidate), "Tester");
    }

    #[test]
    fn test_resolve_affiliation_name() {
        let doc = CandidateLists::parse_eml(
            include_str!("../../test-files/candidate_lists/eml230b_test.eml.xml"),
            EMLParsingMode::Strict,
        )
        .unwrap();
        assert_eq!(
            doc.resolve_affiliation_name(AffiliationId::from_u64(1).unwrap())
                .unwrap(),
            "Partijdige Partij"
        );

        assert_eq!(
            doc.resolve_affiliation_name(AffiliationId::from_u64(2).unwrap())
                .unwrap(),
            "Lijst van de Kandidaten"
        );

        assert_eq!(
            doc.resolve_affiliation_name(AffiliationId::from_u64(3).unwrap())
                .unwrap(),
            "Partij voor de Stemmer"
        );

        assert!(
            doc.resolve_affiliation_name(AffiliationId::from_u64(4).unwrap())
                .is_none()
        );
    }

    #[test]
    fn test_resolve_candidate_name() {
        let doc = CandidateLists::parse_eml(
            include_str!("../../test-files/candidate_lists/eml230b_test.eml.xml"),
            EMLParsingMode::Strict,
        )
        .unwrap();
        assert_eq!(
            doc.resolve_candidate_name(
                AffiliationId::from_u64(1).unwrap(),
                CandidateId::from_u64(1).unwrap()
            )
            .unwrap(),
            "Oorschot, A.B.C."
        );
        assert_eq!(
            doc.resolve_candidate_name(
                AffiliationId::from_u64(2).unwrap(),
                CandidateId::from_u64(2).unwrap()
            )
            .unwrap(),
            "Arets, W.P."
        );

        assert_eq!(
            doc.resolve_candidate_name(
                AffiliationId::from_u64(3).unwrap(),
                CandidateId::from_u64(2).unwrap()
            )
            .unwrap(),
            "Wolfswinkel, G.W."
        );

        assert!(
            doc.resolve_candidate_name(
                AffiliationId::from_u64(3).unwrap(),
                CandidateId::from_u64(3).unwrap()
            )
            .is_none()
        );

        assert!(
            doc.resolve_candidate_name(
                AffiliationId::from_u64(4).unwrap(),
                CandidateId::from_u64(3).unwrap()
            )
            .is_none()
        );
    }
}
