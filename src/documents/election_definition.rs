//! Document variant for the EML_NL Election Definition (`110a`) document.

use crate::{
    EML_SCHEMA_VERSION, EMLError, NS_EML, NS_KR,
    common::{CreationDateTime, IssueDate, TransactionId},
    documents::accepted_root,
    error::{EMLErrorKind, EMLResultExt},
    io::{
        EMLElement, EMLElementWriter, EMLReadElement, EMLWriteElement, collect_struct,
        write_eml_element,
    },
};

pub(crate) const EML_ELECTION_DEFINITION_ID: &str = "110a";

/// Representing a `110a` document, containing an election definition.
#[derive(Debug, Clone)]
pub struct ElectionDefinition {
    /// Transaction id of the document.
    pub transaction_id: TransactionId,
    /// Time this document was created.
    pub creation_date_time: CreationDateTime,
    /// Issue date of the election definition, if present.
    pub issue_date: Option<IssueDate>,
    /// The election event defined in this document.
    pub election_event: ElectionDefinitionElectionEvent,
}

impl EMLReadElement for ElectionDefinition {
    fn read_eml_element(elem: &mut EMLElement<'_, '_>) -> Result<Self, EMLError> {
        accepted_root(elem)?;

        let document_id = elem.attribute_value_req(("Id", None))?;
        if document_id != EML_ELECTION_DEFINITION_ID {
            return Err(EMLErrorKind::InvalidDocumentType(
                EML_ELECTION_DEFINITION_ID,
                document_id.to_string(),
            ))
            .with_span(elem.span());
        }

        Ok(collect_struct!(elem, ElectionDefinition {
            transaction_id: ("TransactionId", NS_EML) => |elem| TransactionId::read_eml_element(elem)?,
            creation_date_time: ("CreationDateTime", NS_KR) => |elem| CreationDateTime::read_eml_element(elem)?,
            issue_date as Option: ("IssueDate", NS_KR) => |elem| IssueDate::read_eml_element(elem)?,
            election_event: ("ElectionEvent", NS_EML) => |elem| ElectionDefinitionElectionEvent::read_eml_element(elem)?,
        }))
    }
}

impl EMLWriteElement for ElectionDefinition {
    fn write_eml_element(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr(("Id", None), EML_ELECTION_DEFINITION_ID)?
            .attr(("SchemaVersion", None), EML_SCHEMA_VERSION)?
            .child(
                ("TransactionId", NS_EML),
                write_eml_element(&self.transaction_id),
            )?
            .child(
                ("CreationDateTime", NS_KR),
                write_eml_element(&self.creation_date_time),
            )?
            .child(
                ("ElectionEvent", NS_EML),
                write_eml_element(&self.election_event),
            )?
            .finish()?;

        Ok(())
    }
}

/// Election event defined in the election definition document.
#[derive(Debug, Clone)]
pub struct ElectionDefinitionElectionEvent {}

impl EMLReadElement for ElectionDefinitionElectionEvent {
    fn read_eml_element(_elem: &mut EMLElement<'_, '_>) -> Result<Self, EMLError> {
        Ok(ElectionDefinitionElectionEvent {})
    }
}

impl EMLWriteElement for ElectionDefinitionElectionEvent {
    fn write_eml_element(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer.empty()?;
        Ok(())
    }
}
