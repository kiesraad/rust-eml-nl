use std::sync::LazyLock;

use regex::Regex;
use thiserror::Error;

use crate::{EMLError, utils::StringValueData};

/// Regular expression for validating neighbourhood code values.
static NEIGHBOURHOOD_CODE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^BU\d{8}$").expect("Failed to compile neighbourhood code regex"));

/// A neighbourhood (i.e. 'buurt') code for the polling station location.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocationNeighbourhoodCode(String);

impl LocationNeighbourhoodCode {
    /// Creates a new `LocationNeighbourhoodCode` from the given string.
    pub fn new(s: &str) -> Result<Self, InvalidLocationNeighbourhoodCode> {
        Self::parse_from_str(s)
    }

    /// Returns the raw value of the neighbourhood code as a string slice.
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// An invalid neighbourhood code for the polling station location.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid neighbourhood code: {0}")]
pub struct InvalidLocationNeighbourhoodCode(String);

impl From<InvalidLocationNeighbourhoodCode> for EMLError {
    fn from(err: InvalidLocationNeighbourhoodCode) -> Self {
        EMLError::value_conversion(err)
    }
}

impl StringValueData for LocationNeighbourhoodCode {
    type Error = InvalidLocationNeighbourhoodCode;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error> {
        if !s.is_empty() && NEIGHBOURHOOD_CODE_RE.is_match(s) {
            Ok(Self(s.into()))
        } else {
            Err(InvalidLocationNeighbourhoodCode(s.into()))
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
    fn test_neighbourhood_code_regex_compiles() {
        LazyLock::force(&NEIGHBOURHOOD_CODE_RE);
    }
}
