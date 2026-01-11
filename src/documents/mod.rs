//! Document variants and related types for the all the specific EML_NL documents.

use crate::{
    EML_SCHEMA_VERSION, EMLError, EMLErrorKind, EMLResultExt as _, NS_EML,
    documents::{
        candidate_list::{CandidateList, EML_CANDIDATE_LIST_ID},
        election_definition::{EML_ELECTION_DEFINITION_ID, ElectionDefinition},
        polling_stations::{EML_POLLING_STATIONS_ID, PollingStations},
    },
    io::{EMLElement, EMLElementWriter, EMLReadElement, EMLWriteElement},
};

pub mod candidate_list;
pub mod election_definition;
pub mod polling_stations;

/// Generic EML document that can represent any of the supported EML variants.
///
/// You can use this struct to parse an EML document of any variant if you don't
/// know in advance which variant you will receive.
#[derive(Debug, Clone)]
pub enum EML {
    /// Representing a `110a` document, containing an election definition.
    ElectionDefinition(ElectionDefinition),
    /// Representing a `110b` document, containing polling stations.
    PollingStations(PollingStations),
    /// Representing a `230b` document, containing a candidate list.
    CandidateList(CandidateList),
}

impl EML {
    /// Get the EML document ID string for this document variant (e.g. `110a`).
    pub fn to_eml_id(&self) -> &'static str {
        match self {
            EML::ElectionDefinition(_) => EML_ELECTION_DEFINITION_ID,
            EML::PollingStations(_) => EML_POLLING_STATIONS_ID,
            EML::CandidateList(_) => EML_CANDIDATE_LIST_ID,
        }
    }

    /// Create a generic EML document from an Election Definition (`110a`) document.
    pub fn from_election_definition_doc(ed: ElectionDefinition) -> Self {
        EML::ElectionDefinition(ed)
    }

    /// Check if this EML document is an Election Definition (`110a`) document.
    pub fn is_election_definition_doc(&self) -> bool {
        matches!(self, EML::ElectionDefinition(_))
    }

    /// Convert this EML document into an Election Definition (`110a`) document, if possible.
    pub fn into_election_definition_doc(self) -> Option<ElectionDefinition> {
        match self {
            EML::ElectionDefinition(ed) => Some(ed),
            _ => None,
        }
    }

    /// Get a reference to this EML document as an Election Definition (`110a`) document, if possible.
    pub fn as_election_definition_doc(&self) -> Option<&ElectionDefinition> {
        match self {
            EML::ElectionDefinition(ed) => Some(ed),
            _ => None,
        }
    }

    /// Create a generic EML document from a Polling Stations (`110b`) document.
    pub fn from_polling_stations_doc(ps: PollingStations) -> Self {
        EML::PollingStations(ps)
    }

    /// Check if this EML document is a Polling Stations (`110b`) document.
    pub fn is_polling_stations_doc(&self) -> bool {
        matches!(self, EML::PollingStations(_))
    }

    /// Convert this EML document into a Polling Stations (`110b`) document, if possible.
    pub fn into_polling_stations_doc(self) -> Option<PollingStations> {
        match self {
            EML::PollingStations(ps) => Some(ps),
            _ => None,
        }
    }

    /// Get a reference to this EML document as a Polling Stations (`110b`) document, if possible.
    pub fn as_polling_stations_doc(&self) -> Option<&PollingStations> {
        match self {
            EML::PollingStations(ps) => Some(ps),
            _ => None,
        }
    }

    /// Create a generic EML document from a Candidate List (`230b`) document.
    pub fn from_candidate_list_doc(cl: CandidateList) -> Self {
        EML::CandidateList(cl)
    }

    /// Check if this EML document is a Candidate List (`230b`) document.
    pub fn is_candidate_list_doc(&self) -> bool {
        matches!(self, EML::CandidateList(_))
    }

    /// Convert this EML document into a Candidate List (`230b`) document, if possible.
    pub fn into_candidate_list_doc(self) -> Option<CandidateList> {
        match self {
            EML::CandidateList(cl) => Some(cl),
            _ => None,
        }
    }

    /// Get a reference to this EML document as a Candidate List (`230b`) document, if possible.
    pub fn as_candidate_list_doc(&self) -> Option<&CandidateList> {
        match self {
            EML::CandidateList(cl) => Some(cl),
            _ => None,
        }
    }
}

impl EMLReadElement for EML {
    fn read_eml_element(elem: &mut EMLElement<'_, '_>) -> Result<Self, EMLError> {
        accepted_root(elem)?;

        let document_id = elem.attribute_value_req(("Id", None))?;
        Ok(match document_id.as_ref() {
            EML_ELECTION_DEFINITION_ID => {
                EML::ElectionDefinition(ElectionDefinition::read_eml_element(elem)?)
            }
            EML_POLLING_STATIONS_ID => {
                EML::PollingStations(PollingStations::read_eml_element(elem)?)
            }
            EML_CANDIDATE_LIST_ID => EML::CandidateList(CandidateList::read_eml_element(elem)?),
            _ => {
                return Err(EMLErrorKind::UnknownDocumentType(document_id.to_string()))
                    .with_span(elem.span());
            }
        })
    }
}

impl EMLWriteElement for EML {
    fn write_eml_element(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        match self {
            EML::ElectionDefinition(ed) => ed.write_eml_element(writer),
            EML::PollingStations(ps) => ps.write_eml_element(writer),
            EML::CandidateList(cl) => cl.write_eml_element(writer),
        }
    }
}

fn accepted_root(elem: &EMLElement<'_, '_>) -> Result<(), EMLError> {
    if !elem.has_name(("EML", Some(NS_EML)))? {
        return Err(EMLErrorKind::InvalidRootElement).with_span(elem.span());
    }

    let schema_version = elem.attribute_value_req(("SchemaVersion", None))?;
    if schema_version == EML_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(EMLErrorKind::SchemaVersionNotSupported(
            schema_version.to_string(),
        ))
        .with_span(elem.span())
    }
}

#[cfg(test)]
mod tests {
    use crate::io::{EMLRead as _, EMLWrite as _};

    use super::*;

    #[test]
    fn test_parsing_arbitrary_eml_documents() {
        let doc = include_str!("../test-emls/candidate_list/eml230b_test.eml.xml");
        let eml = EML::parse_eml(doc, true).expect("Failed to parse EML document");
        assert!(matches!(eml, EML::CandidateList(_)));

        let doc = include_str!("../test-emls/election_definition/eml110a_test.eml.xml");
        let eml = EML::parse_eml(doc, true).expect("Failed to parse EML document");
        assert!(matches!(eml, EML::ElectionDefinition(_)));

        let doc = include_str!("../test-emls/polling_stations/eml110b_test.eml.xml");
        let eml = EML::parse_eml(doc, true).expect("Failed to parse EML document");
        assert!(matches!(eml, EML::PollingStations(_)));
    }

    #[test]
    fn parse_and_write_eml_document_should_not_fail() {
        let doc = include_str!("../test-emls/election_definition/eml110a_test.eml.xml");
        let eml = dbg!(EML::parse_eml(doc, true).expect("Failed to parse EML document"));

        eml.write_eml_root_str(true, true)
            .expect("Failed to output EML document");
    }
}
