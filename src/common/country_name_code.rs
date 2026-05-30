use crate::{
    EMLError, NS_XAL,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName},
};

/// Country name code information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountryNameCode {
    /// The country name code value.
    pub value: Box<str>,
    /// The Scheme attribute, if present.
    pub scheme: Option<Box<str>>,
    /// The Code attribute, if present.
    pub code: Option<Box<str>>,
}

impl CountryNameCode {
    /// Create a new CountryNameCode.
    pub fn new(value: impl Into<Box<str>>) -> Self {
        CountryNameCode {
            value: value.into(),
            scheme: None,
            code: None,
        }
    }

    /// Set the Scheme attribute.
    pub fn with_scheme(mut self, scheme: impl Into<Box<str>>) -> Self {
        self.scheme = Some(scheme.into());
        self
    }

    /// Set the Code attribute.
    pub fn with_code(mut self, code: impl Into<Box<str>>) -> Self {
        self.code = Some(code.into());
        self
    }
}

impl From<String> for CountryNameCode {
    fn from(value: String) -> Self {
        CountryNameCode::new(value)
    }
}

impl From<Box<str>> for CountryNameCode {
    fn from(value: Box<str>) -> Self {
        CountryNameCode::new(value)
    }
}

impl From<&str> for CountryNameCode {
    fn from(value: &str) -> Self {
        CountryNameCode::new(value)
    }
}

impl EMLElement for CountryNameCode {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("CountryNameCode", Some(NS_XAL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(CountryNameCode {
            value: elem.text_without_children()?,
            scheme: elem.attribute_value("Scheme")?.map(Into::into),
            code: elem.attribute_value("Code")?.map(Into::into),
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr_opt("Scheme", self.scheme.as_ref())?
            .attr_opt("Code", self.code.as_ref())?
            .text(self.value.as_ref())?
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{EMLParsingMode, EMLRead, test_write_eml_element, test_xml_fragment};

    #[test]
    fn test_country_name_code_construction() {
        let cnc = CountryNameCode::new("Netherlands")
            .with_scheme("ISO3166")
            .with_code("NL");
        assert_eq!(cnc.value.as_ref(), "Netherlands");
        assert_eq!(cnc.scheme.as_deref(), Some("ISO3166"));
        assert_eq!(cnc.code.as_deref(), Some("NL"));
    }

    #[test]
    fn test_country_name_code_parsing() {
        let xml = test_xml_fragment(
            r#"<xal:CountryNameCode xmlns:xal="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" Scheme="ISO3166" Code="NL">Netherlands</xal:CountryNameCode>"#,
        );
        let cnc = CountryNameCode::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(cnc.value.as_ref(), "Netherlands");
        assert_eq!(cnc.scheme.as_deref(), Some("ISO3166"));
        assert_eq!(cnc.code.as_deref(), Some("NL"));

        let xml_output = test_write_eml_element(&cnc, &[NS_XAL]).unwrap();
        assert_eq!(xml_output, xml);
    }
}
