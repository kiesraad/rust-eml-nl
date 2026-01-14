use std::sync::LazyLock;

use regex::Regex;
use thiserror::Error;

use crate::utils::StringValueData;

/// EML_NL XSBType value.
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct XSBType(String);

impl XSBType {
    /// Create a new XSBType from a string, validating its format
    pub fn new(s: impl AsRef<str>) -> Result<Self, InvalidXSBValueError> {
        StringValueData::parse_from_str(s.as_ref())
    }

    /// Get the raw string value of the XSBType.
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Error type returned when an invalid XSBType value is encountered.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[repr(transparent)]
#[error("Invalid XSBType value, must match ^(CSB|((HSB|SB)\\d+)|(\\d{{4}}))$: {0}")]
pub struct InvalidXSBValueError(String);

/// Regular expression for validating XSBType values.
static XSB_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(CSB|((HSB|SB)\d+)|(\d{4}))$").expect("Failed to compile XSB regex")
});

impl StringValueData for XSBType {
    type Error = InvalidXSBValueError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        if XSB_RE.is_match(s) {
            Ok(XSBType(s.to_string()))
        } else {
            Err(InvalidXSBValueError(s.to_string()))
        }
    }

    fn to_raw_value(&self) -> String {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xsb_regex_compiles() {
        LazyLock::force(&XSB_RE);
    }

    #[test]
    fn test_xsb_valid_values() {
        let valid_values = [
            "CSB", "HSB1", "HSB123", "SB10", "SB999", "0001", "1234", "9999",
        ];

        for value in valid_values {
            let parsed = XSBType::parse_from_str(value);
            assert!(
                parsed.is_ok(),
                "Expected '{}' to parse successfully, got error: {:?}",
                value,
                parsed.err()
            );
        }
    }

    #[test]
    fn test_xsb_invalid_values() {
        let invalid_values = ["CS", "HSB", "SB", "123", "12345", "ABC", "SB-1"];
        for value in invalid_values {
            let parsed = XSBType::parse_from_str(value);
            assert!(
                parsed.is_err(),
                "Expected '{}' to fail parsing, but got: {:?}",
                value,
                parsed.ok()
            );
        }
    }
}
