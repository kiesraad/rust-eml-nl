use std::borrow::Cow;

use crate::{
    EMLError, NS_EML,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName},
    utils::StringValue,
};

/// Document transaction id.
///
/// EML_NL documents contain a transaction id, but this is generally not used
/// and set to `1` as a default.
#[derive(Debug, Clone)]
pub struct TransactionId(pub StringValue<u64>);

impl TransactionId {
    /// Create a new TransactionId from a u64 value.
    pub fn new(id: u64) -> Self {
        TransactionId(StringValue::from_value(id))
    }

    /// Get the raw string value of the transaction id.
    pub fn raw(&self) -> Cow<'_, str> {
        self.0.raw()
    }

    /// Get the parsed u64 value of the transaction id.
    pub fn value(&self) -> Result<u64, EMLError> {
        Ok(self
            .0
            .value_err(("TransactionId", NS_EML), None)?
            .into_owned())
    }
}

impl EMLElement for TransactionId {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("TransactionId", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(TransactionId(elem.string_value()?))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer.text(self.raw().as_ref())?.finish()?;
        Ok(())
    }
}
