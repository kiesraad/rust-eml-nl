use std::sync::LazyLock;

use regex::Regex;
use thiserror::Error;

use crate::{EMLError, EMLValueResultExt as _, utils::StringValueData};

/// Regular expression for validating NameShortCode values.
static NAME_SHORT_CODE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\p{L}*\d{0,7})$").expect("Failed to compile Name Short Code regex")
});

/// A string of type NameShortCode as defined in the EML_NL specification
///
/// Called NameShortCodeType in the schema.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NameShortCode(Box<str>);

impl NameShortCode {
    /// Create a new NameShortCode from a string, validating its format
    pub fn new(s: impl AsRef<str>) -> Result<Self, EMLError> {
        StringValueData::parse_from_str(s.as_ref()).wrap_value_error()
    }

    /// Get the raw string value of the NameShortCode.
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Error returned when a string could not be parsed as a NameShortCode
#[derive(Debug, Clone, Error)]
#[error("Invalid name short code: {0}")]
pub struct InvalidNameShortCodeError(String);

impl StringValueData for NameShortCode {
    type Error = InvalidNameShortCodeError;
    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        // suggested alternative by clippy is not more clear in this case
        #[expect(clippy::len_zero)]
        if s.len() >= 1 && s.len() <= 15 && NAME_SHORT_CODE_RE.is_match(s) {
            Ok(NameShortCode(s.into()))
        } else {
            Err(InvalidNameShortCodeError(s.into()))
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
    fn test_name_short_code_regex_compiles() {
        LazyLock::force(&NAME_SHORT_CODE_RE);
    }

    #[test]
    fn test_valid_name_short_codes() {
        let valid_codes = ["1", "12345", "ABC1234567"];
        for code in valid_codes {
            assert!(
                NameShortCode::new(code).is_ok(),
                "NameShortCode should accept valid code: {}",
                code
            );
        }
    }

    #[test]
    fn test_invalid_name_short_codes() {
        let invalid_codes = ["", "verylongstringthatistoolong", "012345678", "123abc"];
        for code in invalid_codes {
            assert!(
                NameShortCode::new(code).is_err(),
                "NameShortCode should reject invalid code: {}",
                code
            );
        }
    }
}
