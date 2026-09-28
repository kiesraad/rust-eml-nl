use std::sync::LazyLock;

use regex::Regex;
use thiserror::Error;

use crate::{EMLError, utils::StringValueData};

/// Regular expression for validating postal code values.
static POSTAL_CODE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d{4} [A-Z]{2}$").expect("Failed to compile postal code regex"));

/// A postal code for the polling station location.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocationPostalCode(String);

impl LocationPostalCode {
    /// Creates a new `LocationPostalCode` from the given string.
    pub fn new(s: &str) -> Result<Self, InvalidLocationPostalCode> {
        Self::parse_from_str(s)
    }

    /// Returns the raw value of the postal code as a string slice.
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// An invalid postal code for the polling station location.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid postal code: {0}")]
pub struct InvalidLocationPostalCode(String);

impl From<InvalidLocationPostalCode> for EMLError {
    fn from(err: InvalidLocationPostalCode) -> Self {
        EMLError::value_conversion(err)
    }
}

impl StringValueData for LocationPostalCode {
    type Error = InvalidLocationPostalCode;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error> {
        if !s.is_empty() && POSTAL_CODE_RE.is_match(s) {
            Ok(Self(s.into()))
        } else {
            Err(InvalidLocationPostalCode(s.into()))
        }
    }

    fn to_raw_value(&self) -> Box<str> {
        self.0.clone().into_boxed_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postal_code_regex_compiles() {
        LazyLock::force(&POSTAL_CODE_RE);
    }
}
