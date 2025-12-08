use crate::{
    EML_SCHEMA_VERSION, EMLElement, EMLElementWriter, EMLError, EMLWrite, NS_EML, TransactionId,
    accepted_root, collect_struct,
    error::{EMLErrorKind, EMLResultExt},
    reader::EMLParse,
    write_eml_element,
};

pub const EML_CANDIDATE_LIST_ID: &str = "230b";

#[derive(Debug, Clone)]
pub struct EMLCandidateList {
    pub transaction_id: TransactionId,
}

impl EMLParse for EMLCandidateList {
    fn parse_eml_element(elem: &mut EMLElement<'_, '_>) -> Result<Self, EMLError> {
        accepted_root(elem)?;

        let document_id = elem.attribute_value_req("Id", None)?;
        if document_id != EML_CANDIDATE_LIST_ID {
            return Err(EMLErrorKind::InvalidDocumentType(
                EML_CANDIDATE_LIST_ID,
                document_id.to_string(),
            ))
            .with_span(elem.span());
        }

        Ok(collect_struct!(elem, EMLCandidateList {
            ("TransactionId", Some(NS_EML)) as transaction_id => |elem| TransactionId::parse_eml_element(elem)?,
        }))
    }
}

impl EMLWrite for EMLCandidateList {
    fn write_eml_element(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr("Id", None, EML_CANDIDATE_LIST_ID)?
            .attr("SchemaVersion", None, EML_SCHEMA_VERSION)?
            .child(
                "TransactionId",
                Some(NS_EML),
                write_eml_element(&self.transaction_id),
            )?
            .finish()?;

        Ok(())
    }
}
