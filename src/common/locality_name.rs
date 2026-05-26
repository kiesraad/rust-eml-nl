use instant_xml::{FromXml, ToXml};

use crate::NS_XAL;

/// Name of a locality
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(ns(NS_XAL), force_prefix)]
pub struct LocalityName {
    /// Type of the locality, if any
    #[xml(attribute, rename = "Type")]
    pub locality_type: Option<String>,
    /// Associated code for the locality, if any
    #[xml(attribute, rename = "Code")]
    pub code: Option<String>,
    /// Name of the locality
    #[xml(direct)]
    pub name: String,
}

impl LocalityName {
    /// Creates a new `LocalityName` with the given name and no type or code.
    pub fn new(name: impl Into<String>) -> Self {
        LocalityName {
            name: name.into(),
            locality_type: None,
            code: None,
        }
    }

    /// Sets the type of the locality and returns the modified `LocalityName`.
    pub fn with_type(mut self, locality_type: impl Into<String>) -> Self {
        self.locality_type = Some(locality_type.into());
        self
    }

    /// Sets the code of the locality and returns the modified `LocalityName`.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

impl From<String> for LocalityName {
    fn from(name: String) -> Self {
        LocalityName::new(name)
    }
}

impl From<&str> for LocalityName {
    fn from(name: &str) -> Self {
        LocalityName::new(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{EMLRead, test_xml_fragment};

    #[test]
    fn test_locality_name_construction() {
        let loc = LocalityName::new("Amsterdam")
            .with_type("City")
            .with_code("AMS");
        assert_eq!(loc.name, "Amsterdam");
        assert_eq!(loc.locality_type.as_deref(), Some("City"));
        assert_eq!(loc.code.as_deref(), Some("AMS"));
    }

    #[test]
    fn test_locality_name_parsing() {
        let xml = test_xml_fragment(
            r#"<xal:LocalityName xmlns:xal="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" Type="City" Code="AMS">Amsterdam</xal:LocalityName>"#,
        );
        let loc = LocalityName::parse_eml(&xml).unwrap();
        assert_eq!(loc.name, "Amsterdam");
        assert_eq!(loc.locality_type.as_deref(), Some("City"));
        assert_eq!(loc.code.as_deref(), Some("AMS"));
    }
}
