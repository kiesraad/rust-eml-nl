use instant_xml::{FromXml, ToXml};

use crate::NS_XAL;

/// Country name code information.
#[derive(Debug, Clone, PartialEq, Eq, FromXml, ToXml)]
#[xml(rename_all = "PascalCase", ns(NS_XAL), force_prefix)]
pub struct CountryNameCode {
    /// The Scheme attribute, if present.
    #[xml(attribute)]
    pub scheme: Option<String>,
    /// The Code attribute, if present.
    #[xml(attribute)]
    pub code: Option<String>,
    /// The country name code value.
    #[xml(direct)]
    pub value: String,
}

impl CountryNameCode {
    /// Create a new CountryNameCode.
    pub fn new(value: impl Into<String>) -> Self {
        CountryNameCode {
            value: value.into(),
            scheme: None,
            code: None,
        }
    }

    /// Set the Scheme attribute.
    pub fn with_scheme(mut self, scheme: impl Into<String>) -> Self {
        self.scheme = Some(scheme.into());
        self
    }

    /// Set the Code attribute.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

impl From<String> for CountryNameCode {
    fn from(value: String) -> Self {
        CountryNameCode::new(value)
    }
}

impl From<&str> for CountryNameCode {
    fn from(value: &str) -> Self {
        CountryNameCode::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{EMLParsingMode, EMLRead, test_xml_fragment};

    #[test]
    fn test_country_name_code_construction() {
        let cnc = CountryNameCode::new("Netherlands")
            .with_scheme("ISO3166")
            .with_code("NL");
        assert_eq!(cnc.value, "Netherlands");
        assert_eq!(cnc.scheme.as_deref(), Some("ISO3166"));
        assert_eq!(cnc.code.as_deref(), Some("NL"));
    }

    #[test]
    fn test_country_name_code_parsing() {
        let xml = test_xml_fragment(
            r#"<xal:CountryNameCode xmlns:xal="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" Scheme="ISO3166" Code="NL">Netherlands</xal:CountryNameCode>"#,
        );
        let cnc = CountryNameCode::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(cnc.value, "Netherlands");
        assert_eq!(cnc.scheme.as_deref(), Some("ISO3166"));
        assert_eq!(cnc.code.as_deref(), Some("NL"));
    }
}
