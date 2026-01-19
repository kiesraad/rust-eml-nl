use std::sync::LazyLock;

use regex::Regex;
use thiserror::Error;

use crate::utils::StringValueData;

/// Regular expression for validating ContestId values.
static CONTEST_ID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([1-9]\d*|geen|alle|M{0,4}(CM|CD|D?C{0,3})(XC|XL|L?X{0,3})(IX|IV|V?I{0,3}))$")
        .expect("Failed to compile Contest ID regex")
});

/// A string of type ContestId as defined in the EML_NL specification
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct ContestIdType(String);

impl ContestIdType {
    /// Create a new `ContestIdType` from a string, validating its format
    pub fn new(s: impl AsRef<str>) -> Result<Self, InvalidContestIdError> {
        StringValueData::parse_from_str(s.as_ref())
    }

    /// Get the raw string value of the `ContestIdType`
    pub fn value(&self) -> &str {
        &self.0
    }

    /// Check if the `ContestIdType` is "geen"
    pub fn is_geen(&self) -> bool {
        self.0 == "geen"
    }

    /// Check if the `ContestIdType` is "alle"
    pub fn is_alle(&self) -> bool {
        self.0 == "alle"
    }

    /// Create a `ContestIdType` representing "geen"
    pub fn geen() -> Self {
        ContestIdType("geen".to_string())
    }

    /// Create a `ContestIdType` representing "alle"
    pub fn alle() -> Self {
        ContestIdType("alle".to_string())
    }
}

/// Error returned when a string could not be parsed as a ContestId
#[derive(Debug, Clone, Error)]
#[error("Invalid ContestId: {0}")]
pub struct InvalidContestIdError(String);

impl StringValueData for ContestIdType {
    type Error = InvalidContestIdError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        if CONTEST_ID_RE.is_match(s) {
            Ok(ContestIdType(s.to_string()))
        } else {
            Err(InvalidContestIdError(s.to_string()))
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
    fn test_contest_id_regex_compiles() {
        LazyLock::force(&CONTEST_ID_RE);
    }
}
