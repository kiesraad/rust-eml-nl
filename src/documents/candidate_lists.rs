//! Document variant for the EML_NL Candidate List (`230b`) document.

use crate::{
    EML_SCHEMA_VERSION, EMLError, NS_EML,
    common::{
        CanonicalizationMethod, CreationDateTime, IssueDate, ManagingAuthority, TransactionId,
    },
    documents::accepted_root,
    error::{EMLErrorKind, EMLResultExt},
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName, collect_struct},
};

pub(crate) const EML_CANDIDATE_LISTS_ID: &str = "230b";

/// Representing a `230b` document, containing the candidate lists.
#[derive(Debug, Clone)]
pub struct CandidateLists {
    /// Transaction id of the document.
    pub transaction_id: TransactionId,

    /// Managing authority of the document.
    pub managing_authority: ManagingAuthority,

    /// Issue date of the document.
    pub issue_date: IssueDate,

    /// Creation date and time of the document.
    pub creation_date_time: CreationDateTime,

    /// Canonicalization method used in this document, if present.
    pub canonicalization_method: Option<CanonicalizationMethod>,
}

impl EMLElement for CandidateLists {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("EML", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        accepted_root(elem)?;

        let document_id = elem.attribute_value_req(("Id", None))?;
        if document_id != EML_CANDIDATE_LISTS_ID {
            return Err(EMLErrorKind::InvalidDocumentType(
                EML_CANDIDATE_LISTS_ID,
                document_id.to_string(),
            ))
            .with_span(elem.span());
        }

        Ok(collect_struct!(elem, CandidateLists {
            transaction_id: TransactionId::EML_NAME => |elem| TransactionId::read_eml(elem)?,
            managing_authority: ManagingAuthority::EML_NAME => |elem| ManagingAuthority::read_eml(elem)?,
            issue_date: IssueDate::EML_NAME => |elem| IssueDate::read_eml(elem)?,
            creation_date_time: CreationDateTime::EML_NAME => |elem| CreationDateTime::read_eml(elem)?,
            canonicalization_method as Option: CanonicalizationMethod::EML_NAME => |elem| CanonicalizationMethod::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr(("Id", None), EML_CANDIDATE_LISTS_ID)?
            .attr(("SchemaVersion", None), EML_SCHEMA_VERSION)?
            .child_elem(TransactionId::EML_NAME, &self.transaction_id)?
            .child_elem(ManagingAuthority::EML_NAME, &self.managing_authority)?
            .child_elem(IssueDate::EML_NAME, &self.issue_date)?
            .child_elem(CreationDateTime::EML_NAME, &self.creation_date_time)?
            // Note: we don't output the CanonicalizationMethod because we aren't canonicalizing our output
            // .child_elem_option(
            //     CanonicalizationMethod::EML_NAME,
            //     self.canonicalization_method.as_ref(),
            // )?
            .finish()?;

        Ok(())
    }
}
