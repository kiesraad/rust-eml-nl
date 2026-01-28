use std::sync::LazyLock;

use regex::Regex;
use thiserror::Error;

use crate::utils::StringValueData;

/// Regular expression for validating NameShortCodeType values.
static NAME_SHORT_CODE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([1-9]\d*)?$").expect("Failed to compile Name Short Code regex")
});

/// A string of type NameShortCodeType as defined in the EML_NL specification
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct NameShortCodeType(String);

impl NameShortCodeType {
    /// Create a new NameShortCodeType from a string, validating its format
    pub fn new(s: impl AsRef<str>) -> Result<Self, InvalidNameShortCodeError> {
        StringValueData::parse_from_str(s.as_ref())
    }

    /// Get the raw string value of the NameShortCodeType.
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Error returned when a string could not be parsed as a ElectionId
#[derive(Debug, Clone, Error)]
#[error("Invalid NameShortCodeType: {0}")]
pub struct InvalidNameShortCodeError(String);

impl StringValueData for NameShortCodeType {
    type Error = InvalidNameShortCodeError;
    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        // suggested alternative by clippy is not more clear in this case
        #[expect(clippy::len_zero)]
        if s.len() >= 1 && s.len() <= 15 && NAME_SHORT_CODE_RE.is_match(s) {
            Ok(NameShortCodeType(s.to_string()))
        } else {
            Err(InvalidNameShortCodeError(s.to_string()))
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
    fn test_name_short_code_regex_compiles() {
        LazyLock::force(&NAME_SHORT_CODE_RE);
    }
}
