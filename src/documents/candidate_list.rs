//! Document variant for the EML_NL Candidate List (`230b`) document.

use crate::{
    EML_SCHEMA_VERSION, EMLError, NS_EML,
    common::TransactionId,
    documents::accepted_root,
    error::{EMLErrorKind, EMLResultExt},
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName, collect_struct},
};

pub(crate) const EML_CANDIDATE_LIST_ID: &str = "230b";

/// Representing a `230b` document, containing a candidate list.
#[derive(Debug, Clone)]
pub struct CandidateList {
    /// Transaction id of the document.
    pub transaction_id: TransactionId,
}

impl EMLElement for CandidateList {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("EML", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        accepted_root(elem)?;

        let document_id = elem.attribute_value_req(("Id", None))?;
        if document_id != EML_CANDIDATE_LIST_ID {
            return Err(EMLErrorKind::InvalidDocumentType(
                EML_CANDIDATE_LIST_ID,
                document_id.to_string(),
            ))
            .with_span(elem.span());
        }

        Ok(collect_struct!(elem, CandidateList {
            transaction_id: TransactionId::EML_NAME => |elem| TransactionId::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr(("Id", None), EML_CANDIDATE_LIST_ID)?
            .attr(("SchemaVersion", None), EML_SCHEMA_VERSION)?
            .child_elem(TransactionId::EML_NAME, &self.transaction_id)?
            .finish()?;

        Ok(())
    }
}
