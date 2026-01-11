//! Document variant for the EML_NL Polling Stations (`110b`) document.

use crate::{
    EML_SCHEMA_VERSION, EMLError, NS_EML,
    common::TransactionId,
    documents::accepted_root,
    error::{EMLErrorKind, EMLResultExt},
    io::{
        EMLElementReader, EMLElementWriter, EMLReadElement, EMLWriteElement, collect_struct,
        write_eml_element,
    },
};

pub(crate) const EML_POLLING_STATIONS_ID: &str = "110b";

/// Representing a `110b` document, containing polling stations.
#[derive(Debug, Clone)]
pub struct PollingStations {
    /// Transaction id of the document.
    pub transaction_id: TransactionId,
}

impl EMLReadElement for PollingStations {
    fn read_eml_element(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        accepted_root(elem)?;

        let document_id = elem.attribute_value_req(("Id", None))?;
        if document_id != EML_POLLING_STATIONS_ID {
            return Err(EMLErrorKind::InvalidDocumentType(
                EML_POLLING_STATIONS_ID,
                document_id.to_string(),
            ))
            .with_span(elem.span());
        }

        Ok(collect_struct!(elem, PollingStations {
            transaction_id: ("TransactionId", NS_EML) => |elem| TransactionId::read_eml_element(elem)?,
        }))
    }
}

impl EMLWriteElement for PollingStations {
    fn write_eml_element(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr(("Id", None), EML_POLLING_STATIONS_ID)?
            .attr(("SchemaVersion", None), EML_SCHEMA_VERSION)?
            .child(
                ("TransactionId", NS_EML),
                write_eml_element(&self.transaction_id),
            )?
            .finish()?;

        Ok(())
    }
}
