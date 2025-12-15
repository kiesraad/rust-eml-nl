use crate::{
    EMLElement, EMLElementWriter, EMLError, EMLReadElement, EMLWriteElement, NS_EML, TransactionId,
    accepted_root, collect_struct,
    error::{EMLErrorKind, EMLResultExt},
};

pub(crate) const EML_POLLING_STATIONS_ID: &str = "110b";

/// Representing a `110b` document, containing polling stations.
#[derive(Debug, Clone)]
pub struct PollingStations {
    pub transaction_id: TransactionId,
}

impl EMLReadElement for PollingStations {
    fn read_eml_element(elem: &mut EMLElement<'_, '_>) -> Result<Self, EMLError> {
        accepted_root(elem)?;

        let document_id = elem.attribute_value_req("Id", None)?;
        if document_id != EML_POLLING_STATIONS_ID {
            return Err(EMLErrorKind::InvalidDocumentType(
                EML_POLLING_STATIONS_ID,
                document_id.to_string(),
            ))
            .with_span(elem.span());
        }

        Ok(collect_struct!(elem, PollingStations {
            ("TransactionId", Some(NS_EML)) as transaction_id => |elem| TransactionId::read_eml_element(elem)?,
        }))
    }
}

impl EMLWriteElement for PollingStations {
    fn write_eml_element(&self, _writer: EMLElementWriter) -> Result<(), EMLError> {
        todo!()
    }
}
