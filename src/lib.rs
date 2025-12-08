mod common;
mod error;
mod reader;
mod utils;
mod variants;
mod writer;

pub use common::*;
pub use error::*;
pub use reader::*;
pub use utils::*;
pub use variants::*;
pub use writer::*;

/// Supported EML schema version
pub const EML_SCHEMA_VERSION: &str = "5";

/// Namespace URI for the EML standard
pub const NS_EML: &str = "urn:oasis:names:tc:evs:schema:eml";

/// Namespace URI for the Kiesraad expansions on the EML standard
pub const NS_KR: &str = "http://www.kiesraad.nl/extensions";

/// Namespace URI for the eXtensible Address Language (xAL)
pub const NS_XAL: &str = "urn:oasis:names:tc:ciq:xsdschema:xAL:2.0";

/// Namespace URI for the eXtensible Name Language (xNL)
pub const NS_XNL: &str = "urn:oasis:names:tc:ciq:xsdschema:xNL:2.0";

// /// Namespace URI for XML Digital Signatures
// const NS_DS: &str = "http://www.w3.org/2000/09/xmldsig#";
// /// Namespace URI for XML Schema
// const NS_XMLNS: &str = "http://www.w3.org/2000/xmlns/";
// /// Namespace URI for XML
// const NS_XML: &str = "http://www.w3.org/XML/1998/namespace";

#[derive(Debug, Clone)]
pub struct EML {
    pub variant: EMLVariant,
}

impl EMLParse for EML {
    fn parse_eml_element(elem: &mut reader::EMLElement<'_, '_>) -> Result<Self, EMLError> {
        accepted_root(elem)?;

        let document_id = elem.attribute_value_req("Id", None)?;
        let variant = match document_id.as_ref() {
            EML_ELECTION_DEFINITION_ID => {
                EMLVariant::EMLElectionDefinition(EMLElectionDefinition::parse_eml_element(elem)?)
            }
            EML_POLLING_STATIONS_ID => {
                EMLVariant::EMLPollingStations(EMLPollingStations::parse_eml_element(elem)?)
            }
            EML_CANDIDATE_LIST_ID => {
                EMLVariant::EMLCandidateList(EMLCandidateList::parse_eml_element(elem)?)
            }
            _ => {
                return Err(EMLErrorKind::UnknownDocumentType(document_id.to_string()))
                    .with_span(elem.span());
            }
        };

        Ok(EML { variant })
    }
}

impl EMLWrite for EML {
    fn write_eml_element(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        match &self.variant {
            EMLVariant::EMLElectionDefinition(ed) => ed.write_eml_element(writer),
            EMLVariant::EMLPollingStations(ps) => ps.write_eml_element(writer),
            EMLVariant::EMLCandidateList(cl) => cl.write_eml_element(writer),
        }
    }
}

fn accepted_root(elem: &reader::EMLElement<'_, '_>) -> Result<(), EMLError> {
    if !elem.has_name("EML", Some(NS_EML))? {
        return Err(EMLErrorKind::InvalidRootElement).with_span(elem.span());
    }

    let schema_version = elem.attribute_value_req("SchemaVersion", None)?;
    if schema_version == EML_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(EMLErrorKind::SchemaVersionNotSupported(
            schema_version.to_string(),
        ))
        .with_span(elem.span())
    }
}

#[derive(Debug, Clone)]
pub enum EMLVariant {
    /// 110a
    EMLElectionDefinition(EMLElectionDefinition),
    /// 110b
    EMLPollingStations(EMLPollingStations),
    /// 230b
    EMLCandidateList(EMLCandidateList),
}

impl EMLVariant {
    pub fn to_eml_id(&self) -> &'static str {
        match self {
            EMLVariant::EMLElectionDefinition(_) => EML_ELECTION_DEFINITION_ID,
            EMLVariant::EMLPollingStations(_) => EML_POLLING_STATIONS_ID,
            EMLVariant::EMLCandidateList(_) => EML_CANDIDATE_LIST_ID,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let data = include_str!("test-emls/election_definition/eml110a_test.eml.xml");
        let res = dbg!(EML::parse_eml(data).unwrap());
        println!(
            "{}",
            String::from_utf8_lossy(&res.write_eml_root(true, true).unwrap())
        );
    }
}
