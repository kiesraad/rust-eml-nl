use std::borrow::Cow;

use crate::{
    EMLValueResultExt, NS_EML,
    error::EMLError,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName},
    utils::{StringValue, XsDateOrDateTime},
};

/// Document issue date.
///
/// Can be either a date or a date with time.
#[derive(Debug, Clone)]
pub struct IssueDate(pub StringValue<XsDateOrDateTime>);

impl IssueDate {
    /// Create a new IssueDate from a `XsDateOrDateTime` value.
    pub fn new(dt: XsDateOrDateTime) -> Self {
        IssueDate(StringValue::from_value(dt))
    }

    /// Get the raw string value of the issue date.
    pub fn raw(&self) -> Cow<'_, str> {
        self.0.raw()
    }

    /// Get the parsed XsDateOrDateTime value of the issue date.
    pub fn value(&self) -> Result<XsDateOrDateTime, EMLError> {
        Ok(self
            .0
            .value_err()
            .wrap_field_value_error(IssueDate::EML_NAME)?
            .into_owned())
    }
}

impl From<XsDateOrDateTime> for IssueDate {
    fn from(value: XsDateOrDateTime) -> Self {
        IssueDate::new(value)
    }
}

impl EMLElement for IssueDate {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("IssueDate", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(IssueDate(elem.string_value()?))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer.text(self.raw().as_ref())?.finish()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use super::*;
    use crate::{
        EMLVersion,
        io::{EMLParsingMode, EMLRead, test_write_eml_element, test_xml_fragment},
        utils::XsDateTime,
    };

    #[test]
    fn test_issue_date_construction() {
        let dt =
            XsDateOrDateTime::DateTime(XsDateTime::from_str("2024-06-01T12:34:56+00:00").unwrap());
        let id = IssueDate::new(dt);
        assert_eq!(id.raw(), "2024-06-01T12:34:56+00:00");
        assert_eq!(id.value().unwrap(), dt);
    }

    #[test]
    fn test_issue_date_parsing() {
        let xml = test_xml_fragment(
            r#"<IssueDate xmlns="urn:oasis:names:tc:evs:schema:eml">2024-06-01</IssueDate>"#,
        );
        let id = IssueDate::parse_eml_fragment(&xml, EMLParsingMode::Strict, EMLVersion::default())
            .unwrap();
        assert_eq!(id.raw(), "2024-06-01");

        let xml_output = test_write_eml_element(&id, &[NS_EML], EMLVersion::default()).unwrap();
        assert_eq!(xml_output, xml);
    }
}
