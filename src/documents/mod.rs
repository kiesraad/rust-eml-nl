//! Document variants and related types for the all the specific EML_NL documents.

use crate::{
    EML_SCHEMA_VERSION, EMLError, EMLErrorKind, EMLResultExt as _, NS_EML,
    documents::{
        candidate_list::{CandidateList, EML_CANDIDATE_LIST_ID},
        election_definition::{EML_ELECTION_DEFINITION_ID, ElectionDefinition},
        polling_stations::{EML_POLLING_STATIONS_ID, PollingStations},
    },
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName},
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
    ElectionDefinition(Box<ElectionDefinition>),
    /// Representing a `110b` document, containing polling stations.
    PollingStations(Box<PollingStations>),
    /// Representing a `230b` document, containing a candidate list.
    CandidateList(Box<CandidateList>),
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

    /// Get a friendly name for this EML document variant.
    pub fn to_friendly_name(&self) -> &'static str {
        match self {
            EML::ElectionDefinition(_) => "Election Definition",
            EML::PollingStations(_) => "Polling Stations",
            EML::CandidateList(_) => "Candidate List",
        }
    }

    /// Create a generic EML document from an Election Definition (`110a`) document.
    pub fn from_election_definition_doc(ed: ElectionDefinition) -> Self {
        EML::ElectionDefinition(Box::new(ed))
    }

    /// Check if this EML document is an Election Definition (`110a`) document.
    pub fn is_election_definition_doc(&self) -> bool {
        matches!(self, EML::ElectionDefinition(_))
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
        EML::PollingStations(Box::new(ps))
    }

    /// Check if this EML document is a Polling Stations (`110b`) document.
    pub fn is_polling_stations_doc(&self) -> bool {
        matches!(self, EML::PollingStations(_))
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
        EML::CandidateList(Box::new(cl))
    }

    /// Check if this EML document is a Candidate List (`230b`) document.
    pub fn is_candidate_list_doc(&self) -> bool {
        matches!(self, EML::CandidateList(_))
    }

    /// Get a reference to this EML document as a Candidate List (`230b`) document, if possible.
    pub fn as_candidate_list_doc(&self) -> Option<&CandidateList> {
        match self {
            EML::CandidateList(cl) => Some(cl),
            _ => None,
        }
    }
}

impl EMLElement for EML {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("EML", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        accepted_root(elem)?;

        let document_id = elem.attribute_value_req(("Id", None))?;
        Ok(match document_id.as_ref() {
            EML_ELECTION_DEFINITION_ID => {
                EML::ElectionDefinition(Box::new(ElectionDefinition::read_eml(elem)?))
            }
            EML_POLLING_STATIONS_ID => {
                EML::PollingStations(Box::new(PollingStations::read_eml(elem)?))
            }
            EML_CANDIDATE_LIST_ID => EML::CandidateList(Box::new(CandidateList::read_eml(elem)?)),
            _ => {
                return Err(EMLErrorKind::UnknownDocumentType(document_id.to_string()))
                    .with_span(elem.span());
            }
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        match self {
            EML::ElectionDefinition(ed) => ed.write_eml(writer),
            EML::PollingStations(ps) => ps.write_eml(writer),
            EML::CandidateList(cl) => cl.write_eml(writer),
        }
    }
}

fn accepted_root(elem: &EMLElementReader<'_, '_>) -> Result<(), EMLError> {
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
    use crate::io::{EMLParsingMode, EMLRead as _, EMLWrite as _};

    use super::*;

    #[test]
    fn test_parsing_arbitrary_eml_documents() {
        let doc = include_str!("../../test-emls/candidate_list/eml230b_test.eml.xml");
        let eml = EML::parse_eml(doc, EMLParsingMode::Strict)
            .ok()
            .expect("Failed to parse EML document");
        assert!(matches!(eml, EML::CandidateList(_)));

        let doc = include_str!("../../test-emls/election_definition/eml110a_test.eml.xml");
        let eml = EML::parse_eml(doc, EMLParsingMode::Strict)
            .ok()
            .expect("Failed to parse EML document");
        assert!(matches!(eml, EML::ElectionDefinition(_)));

        let doc = include_str!("../../test-emls/polling_stations/eml110b_test.eml.xml");
        let eml = EML::parse_eml(doc, EMLParsingMode::Strict)
            .ok()
            .expect("Failed to parse EML document");
        assert!(matches!(eml, EML::PollingStations(_)));
    }

    #[test]
    fn parse_and_write_eml_document_should_not_fail() {
        let doc = include_str!("../../test-emls/election_definition/eml110a_test.eml.xml");
        let eml = dbg!(
            EML::parse_eml(doc, EMLParsingMode::Strict)
                .ok()
                .expect("Failed to parse EML document")
        );

        println!(
            "{}",
            eml.write_eml_root_str(true, true)
                .expect("Failed to output EML document")
        );
    }
}
