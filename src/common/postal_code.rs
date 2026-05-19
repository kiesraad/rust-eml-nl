use instant_xml::ToXml;

use crate::{
    EMLError, NS_XAL,
    io::{EMLElement, EMLElementReader, QualifiedName, collect_struct},
};

/// Postal code element
#[derive(Debug, Clone, ToXml)]
#[xml(ns(NS_XAL), force_prefix)]
pub struct PostalCode {
    /// Postal code number
    #[xml(rename = "PostalCodeNumber")]
    pub number: PostalCodeNumber,

    /// The `Type` attribute of the `PostalCode` element, if present.
    #[xml(attribute, rename = "Type")]
    pub postal_code_type: Option<String>,
}

impl PostalCode {
    /// Create a new `PostalCode` for a specific postal code number.
    pub fn new(number: impl Into<String>) -> Self {
        PostalCode {
            number: PostalCodeNumber {
                number_type: None,
                code: None,
                number: number.into(),
            },
            postal_code_type: None,
        }
    }

    /// Get the postal code number string
    pub fn number(&self) -> &str {
        &self.number.number
    }

    /// Set the `Type` attribute of the `PostalCode` element.
    pub fn with_type(mut self, postal_code_type: impl Into<String>) -> Self {
        self.postal_code_type = Some(postal_code_type.into());
        self
    }

    /// Set the `Type` attribute of the `PostalCodeNumber` element.
    pub fn with_number_type(mut self, number_type: impl Into<String>) -> Self {
        self.number.number_type = Some(number_type.into());
        self
    }

    /// Set the `Code` attribute of the `PostalCodeNumber` element.
    pub fn with_number_code(mut self, code: impl Into<String>) -> Self {
        self.number.code = Some(code.into());
        self
    }
}

impl From<String> for PostalCode {
    fn from(number: String) -> Self {
        PostalCode::new(number)
    }
}

impl From<&str> for PostalCode {
    fn from(number: &str) -> Self {
        PostalCode::new(number)
    }
}

impl EMLElement for PostalCode {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("PostalCode", Some(NS_XAL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, PostalCode {
            number: PostalCodeNumber::EML_NAME => |elem| PostalCodeNumber::read_eml(elem)?,
            postal_code_type: elem.attribute_value("Type")?.map(|s| s.into_owned()),
        }))
    }
}

/// Postal code number element
#[derive(Debug, Clone, ToXml)]
#[xml(ns(NS_XAL), force_prefix)]
pub struct PostalCodeNumber {
    /// Type attribute of the postal code number
    #[xml(attribute, rename = "Type")]
    pub number_type: Option<String>,
    /// Code attribute of the postal code number
    #[xml(attribute, rename = "Code")]
    pub code: Option<String>,
    /// The postal code value
    #[xml(direct)]
    pub number: String,
}

impl EMLElement for PostalCodeNumber {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("PostalCodeNumber", Some(NS_XAL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let number_type = elem.attribute_value("Type")?.map(|s| s.into_owned());
        let code = elem.attribute_value("Code")?.map(|s| s.into_owned());
        let number = elem.text_without_children()?;
        Ok(PostalCodeNumber {
            number_type,
            code,
            number,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{EMLParsingMode, EMLRead, test_xml_fragment};

    #[test]
    fn test_postal_code_construction() {
        let postal_code = PostalCode::new("1234 AB")
            .with_type("Test")
            .with_number_type("Primary")
            .with_number_code("PC123");
        assert_eq!(postal_code.number(), "1234 AB");
        assert_eq!(postal_code.postal_code_type.as_deref(), Some("Test"));
        assert_eq!(postal_code.number.number_type.as_deref(), Some("Primary"));
        assert_eq!(postal_code.number.code.as_deref(), Some("PC123"));
    }

    #[test]
    fn test_postal_code_parsing() {
        let xml = test_xml_fragment(
            r#"
            <xal:PostalCode xmlns:xal="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" Type="Test">
                <xal:PostalCodeNumber Type="Primary" Code="PC123">1234 AB</xal:PostalCodeNumber>
            </xal:PostalCode>
            "#,
        );
        let postal_code = PostalCode::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(postal_code.number(), "1234 AB");
        assert_eq!(postal_code.postal_code_type.as_deref(), Some("Test"));
        assert_eq!(postal_code.number.number_type.as_deref(), Some("Primary"));
        assert_eq!(postal_code.number.code.as_deref(), Some("PC123"));
    }

    #[test]
    fn test_postal_code_simple_parsing() {
        let xml = test_xml_fragment(
            r#"
            <xal:PostalCode xmlns:xal="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0">
                <xal:PostalCodeNumber>1234 AB</xal:PostalCodeNumber>
            </xal:PostalCode>
            "#,
        );

        let postal_code = PostalCode::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(postal_code.number(), "1234 AB");
        assert_eq!(postal_code.postal_code_type, None);
        assert_eq!(postal_code.number.number_type, None);
        assert_eq!(postal_code.number.code, None);
    }
}
