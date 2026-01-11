//! Document variant for the EML_NL Election Definition (`110a`) document.

use crate::{
    EML_SCHEMA_VERSION, EMLError, NS_EML,
    common::{CreationDateTime, IssueDate, ManagingAuthority, TransactionId},
    documents::accepted_root,
    error::{EMLErrorKind, EMLResultExt},
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName, collect_struct},
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
    /// Managing authority of the election, if present.
    pub managing_authority: Option<ManagingAuthority>,
    /// The election event defined in this document.
    pub election_event: ElectionDefinitionElectionEvent,
}

impl EMLElement for ElectionDefinition {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("EML", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
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
            transaction_id: TransactionId::EML_NAME => |elem| TransactionId::read_eml(elem)?,
            creation_date_time: CreationDateTime::EML_NAME => |elem| CreationDateTime::read_eml(elem)?,
            issue_date as Option: IssueDate::EML_NAME => |elem| IssueDate::read_eml(elem)?,
            managing_authority as Option: ManagingAuthority::EML_NAME => |elem| ManagingAuthority::read_eml(elem)?,
            election_event: ElectionDefinitionElectionEvent::EML_NAME => |elem| ElectionDefinitionElectionEvent::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr(("Id", None), EML_ELECTION_DEFINITION_ID)?
            .attr(("SchemaVersion", None), EML_SCHEMA_VERSION)?
            .child_elem(TransactionId::EML_NAME, &self.transaction_id)?
            .child_elem(CreationDateTime::EML_NAME, &self.creation_date_time)?
            .child_elem_option(IssueDate::EML_NAME, self.issue_date.as_ref())?
            .child_elem_option(
                ManagingAuthority::EML_NAME,
                self.managing_authority.as_ref(),
            )?
            .child_elem(
                ElectionDefinitionElectionEvent::EML_NAME,
                &self.election_event,
            )?
            .finish()?;

        Ok(())
    }
}

/// Election event defined in the election definition document.
#[derive(Debug, Clone)]
pub struct ElectionDefinitionElectionEvent {}

impl EMLElement for ElectionDefinitionElectionEvent {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ElectionEvent", Some(NS_EML));

    fn read_eml(_elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(ElectionDefinitionElectionEvent {})
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer.empty()?;
        Ok(())
    }
}
