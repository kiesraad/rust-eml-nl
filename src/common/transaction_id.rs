use std::borrow::Cow;

use crate::{
    EMLError, EMLValueResultExt, NS_EML,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EMLVersion,
        io::{EMLParsingMode, EMLRead as _, test_write_eml_element, test_xml_fragment},
    };

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
        let transaction_id =
            TransactionId::parse_eml_fragment(&xml, EMLParsingMode::Strict, EMLVersion::default())
                .unwrap();
        assert_eq!(transaction_id.raw(), "5678");
        assert_eq!(transaction_id.value().unwrap(), 5678);

        let xml_output =
            test_write_eml_element(&transaction_id, &[NS_EML], EMLVersion::default()).unwrap();
        assert_eq!(xml_output, xml);
    }
}
