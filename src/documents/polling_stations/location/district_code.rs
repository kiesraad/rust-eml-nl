use std::sync::LazyLock;

use regex::Regex;
use thiserror::Error;

use crate::{EMLError, utils::StringValueData};

/// Regular expression for validating location district code values.
static LOCATION_DISTRICT_CODE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^WK\d{6}$").expect("Failed to compile location district code regex")
});

/// A district (i.e. 'wijk') code for the polling station location.
///
/// Note this is different from the district as defined by the rest of the
/// EML specification, but specifically relates to the BAG meaning of a
/// district.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocationDistrictCode(String);

impl LocationDistrictCode {
    /// Creates a new `LocationDistrictCode` from the given string.
    pub fn new(s: &str) -> Result<Self, InvalidLocationDistrictCode> {
        Self::parse_from_str(s)
    }

    /// Returns the raw value of the location district code as a string slice.
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// An invalid location district code for the polling station location.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid district code: {0}")]
pub struct InvalidLocationDistrictCode(String);

impl From<InvalidLocationDistrictCode> for EMLError {
    fn from(err: InvalidLocationDistrictCode) -> Self {
        EMLError::value_conversion(err)
    }
}

impl StringValueData for LocationDistrictCode {
    type Error = InvalidLocationDistrictCode;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error> {
        if !s.is_empty() && LOCATION_DISTRICT_CODE_RE.is_match(s) {
            Ok(Self(s.into()))
        } else {
            Err(InvalidLocationDistrictCode(s.into()))
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
    fn test_location_district_code_regex_compiles() {
        LazyLock::force(&LOCATION_DISTRICT_CODE_RE);
    }
}
