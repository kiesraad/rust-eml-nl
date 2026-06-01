use crate::{
    EMLError, NS_XAL,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName},
};

/// Name of a locality
#[derive(Debug, Clone)]
pub struct LocalityName {
    /// Name of the locality
    pub name: Box<str>,
    /// Type of the locality, if any
    pub locality_type: Option<Box<str>>,
    /// Associated code for the locality, if any
    pub code: Option<Box<str>>,
}

impl LocalityName {
    /// Creates a new `LocalityName` with the given name and no type or code.
    pub fn new(name: impl Into<Box<str>>) -> Self {
        LocalityName {
            name: name.into(),
            locality_type: None,
            code: None,
        }
    }

    /// Sets the type of the locality and returns the modified `LocalityName`.
    pub fn with_type(mut self, locality_type: impl Into<Box<str>>) -> Self {
        self.locality_type = Some(locality_type.into());
        self
    }

    /// Sets the code of the locality and returns the modified `LocalityName`.
    pub fn with_code(mut self, code: impl Into<Box<str>>) -> Self {
        self.code = Some(code.into());
        self
    }
}

impl From<String> for LocalityName {
    fn from(name: String) -> Self {
        LocalityName::new(name)
    }
}

impl From<Box<str>> for LocalityName {
    fn from(name: Box<str>) -> Self {
        LocalityName::new(name)
    }
}

impl From<&str> for LocalityName {
    fn from(name: &str) -> Self {
        LocalityName::new(name)
    }
}

impl EMLElement for LocalityName {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("LocalityName", Some(NS_XAL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(LocalityName {
            name: elem.text_without_children()?,
            locality_type: elem.attribute_value("Type")?.map(|s| s.into()),
            code: elem.attribute_value("Code")?.map(|s| s.into()),
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        let mut writer = writer;
        if let Some(ref locality_type) = self.locality_type {
            writer = writer.attr("Type", locality_type)?;
        }
        if let Some(ref code) = self.code {
            writer = writer.attr("Code", code)?;
        }
        writer.text(&self.name)?.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{EMLParsingMode, EMLRead, test_write_eml_element, test_xml_fragment};

    #[test]
    fn test_locality_name_construction() {
        let loc = LocalityName::new("Amsterdam")
            .with_type("City")
            .with_code("AMS");
        assert_eq!(loc.name.as_ref(), "Amsterdam");
        assert_eq!(loc.locality_type.as_deref(), Some("City"));
        assert_eq!(loc.code.as_deref(), Some("AMS"));
    }

    #[test]
    fn test_locality_name_parsing() {
        let xml = test_xml_fragment(
            r#"<xal:LocalityName xmlns:xal="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" Type="City" Code="AMS">Amsterdam</xal:LocalityName>"#,
        );
        let loc = LocalityName::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(loc.name.as_ref(), "Amsterdam");
        assert_eq!(loc.locality_type.as_deref(), Some("City"));
        assert_eq!(loc.code.as_deref(), Some("AMS"));

        let xml_output = test_write_eml_element(&loc, &[NS_XAL]).unwrap();
        assert_eq!(xml_output, xml);
    }
}
