//! EML (Election Markup Language) library written by the
//! [Kiesraad](https://www.kiesraad.nl/) (the Dutch Electoral Council) for
//! parsing and writing EML-NL documents written in safe Rust code only.
//!
//! This library sometimes uses EML and EML-NL interchangeably, but only EML-NL
//! is supported. For details of the EML-NL standard, see the
//! [Kiesraad EML-NL repository](https://github.com/kiesraad/EML_NL/).
//!
//! The main entrypoints for this crate are the [`EML`] enum for parsing any
//! EML document. You can also use the specific structs for specific EML-NL
//! documents, such as [`ElectionDefinition`] for a 110a EML document. The best
//! reference for which documents are supported are the variants in the [`EML`]
//! enum.
//!
//! Reading of EML documents is done through the [`EMLRead`] trait, while
//! writing is done through the [`EMLWrite`] trait.
//!
//! This crate only parses and writes EML documents in memory, it does not
//! support streaming parsing or writing. This was a design decision to keep
//! the code simple and maintainable, and it is expected that EML documents will
//! generally not be extremely large. Up to a few megabytes were expected, but
//! larger documents will work fine as long as enough memory is available.
//! Expect somewhere between 1.2 and 2.0 times the original document size
//! depending on the contents of the file.

// This crate must only use safe Rust code.
#![forbid(unsafe_code)]

mod common;
mod error;
mod qualified_name;
mod reader;
mod utils;
mod variants;
mod writer;

pub use common::*;
pub use error::*;
pub use qualified_name::*;
pub use reader::*;
pub use utils::*;
pub use variants::*;
pub use writer::*;

/// Supported EML schema version
pub(crate) const EML_SCHEMA_VERSION: &str = "5";

/// Namespace URI for the EML standard
pub(crate) const NS_EML: &str = "urn:oasis:names:tc:evs:schema:eml";

/// Namespace URI for the Kiesraad expansions on the EML standard
pub(crate) const NS_KR: &str = "http://www.kiesraad.nl/extensions";

/// Namespace URI for the eXtensible Address Language (xAL)
pub(crate) const NS_XAL: &str = "urn:oasis:names:tc:ciq:xsdschema:xAL:2.0";

/// Namespace URI for the eXtensible Name Language (xNL)
pub(crate) const NS_XNL: &str = "urn:oasis:names:tc:ciq:xsdschema:xNL:2.0";

// /// Namespace URI for XML Digital Signatures
// pub(crate) const NS_DS: &str = "http://www.w3.org/2000/09/xmldsig#";
// /// Namespace URI for XML Schema
// pub(crate) const NS_XMLNS: &str = "http://www.w3.org/2000/xmlns/";
// /// Namespace URI for XML
// pub(crate) const NS_XML: &str = "http://www.w3.org/XML/1998/namespace";

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
    pub fn to_eml_id(&self) -> &'static str {
        match self {
            EML::ElectionDefinition(_) => EML_ELECTION_DEFINITION_ID,
            EML::PollingStations(_) => EML_POLLING_STATIONS_ID,
            EML::CandidateList(_) => EML_CANDIDATE_LIST_ID,
        }
    }
}

impl EMLReadElement for EML {
    fn read_eml_element(elem: &mut reader::EMLElement<'_, '_>) -> Result<Self, EMLError> {
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

fn accepted_root(elem: &reader::EMLElement<'_, '_>) -> Result<(), EMLError> {
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
    use super::*;

    #[test]
    fn test_parsing_arbitrary_eml_documents() {
        let doc = include_str!("test-emls/candidate_list/eml230b_test.eml.xml");
        let eml = EML::parse_eml(doc, true).expect("Failed to parse EML document");
        assert!(matches!(eml, EML::CandidateList(_)));

        let doc = include_str!("test-emls/election_definition/eml110a_test.eml.xml");
        let eml = EML::parse_eml(doc, true).expect("Failed to parse EML document");
        assert!(matches!(eml, EML::ElectionDefinition(_)));

        let doc = include_str!("test-emls/polling_stations/eml110b_test.eml.xml");
        let eml = EML::parse_eml(doc, true).expect("Failed to parse EML document");
        assert!(matches!(eml, EML::PollingStations(_)));
    }

    #[test]
    fn parse_and_write_eml_document_should_not_fail() {
        let doc = include_str!("test-emls/election_definition/eml110a_test.eml.xml");
        let eml = EML::parse_eml(doc, true).expect("Failed to parse EML document");

        eml.write_eml_root_str(true, true)
            .expect("Failed to output EML document");
    }
}
