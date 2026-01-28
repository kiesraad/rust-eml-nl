use std::sync::LazyLock;

use regex::Regex;
use thiserror::Error;

use crate::utils::StringValueData;

/// Regular expression for validating AffiliationIdType values.
static AFFILIATION_ID_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([1-9]\d*)?$").expect("Failed to compile Affiliation ID regex"));

/// A string of type AffiliationIdType as defined in the EML_NL specification
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct AffiliationIdType(String);

impl AffiliationIdType {
    /// Create a new AffiliationIdType from a string, validating its format
    pub fn new(s: impl AsRef<str>) -> Result<Self, InvalidAffiliationIdError> {
        StringValueData::parse_from_str(s.as_ref())
    }

    /// Get the raw string value of the AffiliationIdType.
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Error returned when a string could not be parsed as a ElectionId
#[derive(Debug, Clone, Error)]
#[error("Invalid AffiliationIdType: {0}")]
pub struct InvalidAffiliationIdError(String);

impl StringValueData for AffiliationIdType {
    type Error = InvalidAffiliationIdError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        if AFFILIATION_ID_RE.is_match(s) {
            Ok(AffiliationIdType(s.to_string()))
        } else {
            Err(InvalidAffiliationIdError(s.to_string()))
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
    fn test_affiliation_id_regex_compiles() {
        LazyLock::force(&AFFILIATION_ID_RE);
    }
}
