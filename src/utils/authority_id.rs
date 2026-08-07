use std::sync::LazyLock;

use regex::Regex;
use thiserror::Error;

use crate::{EMLError, EMLValueResultExt, utils::StringValueData};

/// EML_NL authority id (called XSBType in the schema) value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct AuthorityId(Box<str>);

impl AuthorityId {
    /// Create a new AuthorityId from a string, validating its format
    pub fn new(s: impl AsRef<str>) -> Result<Self, EMLError> {
        StringValueData::parse_from_str(s.as_ref()).wrap_value_error()
    }

    /// Get the raw string value of the authority id.
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Error type returned when an invalid authority id value is encountered.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[repr(transparent)]
#[error("Invalid authority id: {0}")]
pub struct InvalidAuthorityIdError(String);

impl From<InvalidAuthorityIdError> for EMLError {
    fn from(err: InvalidAuthorityIdError) -> Self {
        EMLError::value_conversion(err)
    }
}

/// Regular expression for validating AuthorityId values.
static AUTHORITY_ID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(CSB|((HSB|SB)\d+)|(\d{4}))$").expect("Failed to compile AuthorityId regex")
});

impl StringValueData for AuthorityId {
    type Error = InvalidAuthorityIdError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        if AUTHORITY_ID_RE.is_match(s) {
            Ok(AuthorityId(s.into()))
        } else {
            Err(InvalidAuthorityIdError(s.into()))
        }
    }

    fn to_raw_value(&self) -> Box<str> {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authority_id_regex_compiles() {
        LazyLock::force(&AUTHORITY_ID_RE);
    }

    #[test]
    fn test_authority_id_valid_values() {
        let valid_values = [
            "CSB", "HSB1", "HSB123", "SB10", "SB999", "0001", "1234", "9999",
        ];

        for value in valid_values {
            let parsed = AuthorityId::parse_from_str(value);
            assert!(
                parsed.is_ok(),
                "Expected '{}' to parse successfully, got error: {:?}",
                value,
                parsed.err()
            );
        }
    }

    #[test]
    fn test_authority_id_invalid_values() {
        let invalid_values = ["CS", "HSB", "SB", "123", "12345", "ABC", "SB-1"];
        for value in invalid_values {
            let parsed = AuthorityId::parse_from_str(value);
            assert!(
                parsed.is_err(),
                "Expected '{}' to fail parsing, but got: {:?}",
                value,
                parsed.ok()
            );
        }
    }
}
