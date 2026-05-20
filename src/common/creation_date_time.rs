use std::borrow::Cow;

use instant_xml::{FromXml, ToXml};

use crate::{
    EMLError, EMLValueResultExt, NS_KR,
    utils::{StringValue, XsDateTime},
};

/// Document creation date time.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(ns(NS_KR), force_prefix)]
pub struct CreationDateTime(pub StringValue<XsDateTime>);

impl CreationDateTime {
    /// Create a new CreationDateTime from a XsDateTime value.
    pub fn new(dt: XsDateTime) -> Self {
        CreationDateTime(StringValue::from_value(dt))
    }

    /// Get the raw string value of the creation date time.
    pub fn raw(&self) -> Cow<'_, str> {
        self.0.raw()
    }

    /// Get the parsed XsDateTime value of the creation date time.
    pub fn value(&self) -> Result<XsDateTime, EMLError> {
        Ok(self
            .0
            .value_err()
            .wrap_field_value_error(("CreationDateTime", NS_KR))?
            .into_owned())
    }
}

impl From<XsDateTime> for CreationDateTime {
    fn from(value: XsDateTime) -> Self {
        CreationDateTime::new(value)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use super::*;
    use crate::io::{EMLParsingMode, EMLRead, test_xml_fragment};

    #[test]
    fn test_creation_date_time_construction() {
        let dt = XsDateTime::from_str("2024-06-01T12:34:56+00:00").unwrap();
        let cdt = CreationDateTime::new(dt);
        assert_eq!(cdt.raw(), "2024-06-01T12:34:56+00:00");
        assert_eq!(
            cdt.value().unwrap().datetime_utc().to_rfc3339(),
            "2024-06-01T12:34:56+00:00"
        );
    }

    #[test]
    fn test_creation_date_time_parsing() {
        let xml = test_xml_fragment(
            r#"<kr:CreationDateTime xmlns:kr="http://www.kiesraad.nl/extensions">2024-06-01T12:34:56+00:00</kr:CreationDateTime>"#,
        );
        let cdt = CreationDateTime::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(cdt.raw(), "2024-06-01T12:34:56+00:00");
    }
}
