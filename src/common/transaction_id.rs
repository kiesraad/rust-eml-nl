use std::borrow::Cow;

use instant_xml::{FromXml, ToXml};

use crate::{EMLError, EMLValueResultExt, NS_EML, utils::StringValue};

/// Document transaction id.
///
/// EML_NL documents contain a transaction id, but this is generally not used
/// and set to `1` as a default.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(ns(NS_EML))]
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
            .value_err()
            .wrap_field_value_error(("TransactionId", NS_EML))?
            .into_owned())
    }
}

impl From<u64> for TransactionId {
    fn from(value: u64) -> Self {
        TransactionId(StringValue::from_value(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{EMLParsingMode, EMLRead as _, test_xml_fragment};

    #[test]
    fn test_transaction_id_construction() {
        let transaction_id = TransactionId::new(1234);
        assert_eq!(transaction_id.raw(), "1234");
        assert_eq!(transaction_id.value().unwrap(), 1234);
    }

    #[test]
    fn test_transaction_id_parsing() {
        let xml = test_xml_fragment(
            r#"<TransactionId xmlns="urn:oasis:names:tc:evs:schema:eml">5678</TransactionId>"#,
        );
        let transaction_id = TransactionId::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(transaction_id.raw(), "5678");
        assert_eq!(transaction_id.value().unwrap(), 5678);
    }
}
