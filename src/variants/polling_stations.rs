use crate::{
    EMLElement, EMLElementWriter, EMLError, EMLWrite, NS_EML, TransactionId, accepted_root,
    collect_struct,
    error::{EMLErrorKind, EMLResultExt},
    reader::EMLParse,
};

pub const EML_POLLING_STATIONS_ID: &str = "110b";

#[derive(Debug, Clone)]
pub struct EMLPollingStations {
    pub transaction_id: TransactionId,
}

impl EMLParse for EMLPollingStations {
    fn parse_eml_element(elem: &mut EMLElement<'_, '_>) -> Result<Self, EMLError> {
        accepted_root(elem)?;

        let document_id = elem.attribute_value_req("Id", None)?;
        if document_id != EML_POLLING_STATIONS_ID {
            return Err(EMLErrorKind::InvalidDocumentType(
                EML_POLLING_STATIONS_ID,
                document_id.to_string(),
            ))
            .with_span(elem.span());
        }

        Ok(collect_struct!(elem, EMLPollingStations {
            ("TransactionId", Some(NS_EML)) as transaction_id => |elem| TransactionId::parse_eml_element(elem)?,
        }))
    }
}

impl EMLWrite for EMLPollingStations {
    fn write_eml_element(&self, _writer: EMLElementWriter) -> Result<(), EMLError> {
        todo!()
    }
}
