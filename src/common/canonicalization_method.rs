use instant_xml::{FromXml, ToXml};

use crate::NS_DS;

/// XML CanonicalizationMethod element
#[derive(Debug, Clone, PartialEq, Eq, FromXml, ToXml)]
#[xml(rename_all = "PascalCase", ns(NS_DS), force_prefix)]
pub struct CanonicalizationMethod {
    #[xml(attribute)]
    algorithm: String,
}

impl CanonicalizationMethod {
    /// Create a new CanonicalizationMethod with the given algorithm.
    pub fn new(algorithm: impl Into<String>) -> Self {
        CanonicalizationMethod {
            algorithm: algorithm.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{EMLParsingMode, EMLRead, test_xml_fragment};

    #[test]
    fn test_canonicalization_method_construction() {
        let method = CanonicalizationMethod::new("test-algorithm");
        assert_eq!(method.algorithm, "test-algorithm");
    }

    #[test]
    fn test_canonicalization_method_parsing() {
        let xml = test_xml_fragment(
            r#"<ds:CanonicalizationMethod xmlns:ds="http://www.w3.org/2000/09/xmldsig#" Algorithm="test-algorithm"/>"#,
        );
        let method = CanonicalizationMethod::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(method.algorithm, "test-algorithm");
    }
}
