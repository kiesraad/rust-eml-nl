use std::borrow::Cow;

use crate::{EMLReadElement, EMLWriteElement, StringValue, error::EMLError};

/// Document transaction id.
///
/// EML-NL documents contain a transaction id, but this is generally not used
/// and set to `1` as a default.
#[derive(Debug, Clone)]
pub struct TransactionId(pub StringValue<u64>);

impl TransactionId {
    pub fn raw(&self) -> Cow<'_, str> {
        self.0.raw()
    }

    pub fn value(&self) -> Result<u64, EMLError> {
        Ok(self
            .0
            .value()
            .map_err(|e| EMLError::invalid_value("TransactionId", e))?
            .into_owned())
    }
}

impl EMLReadElement for TransactionId {
    fn read_eml_element(
        elem: &mut crate::reader::EMLElement<'_, '_>,
    ) -> Result<Self, crate::error::EMLError> {
        let text = elem.text_without_children()?;
        Ok(TransactionId(StringValue::from_raw(text)))
    }
}

impl EMLWriteElement for TransactionId {
    fn write_eml_element(&self, writer: crate::EMLElementWriter) -> Result<(), EMLError> {
        writer.text(self.raw().as_ref())?.finish()?;
        Ok(())
    }
}
