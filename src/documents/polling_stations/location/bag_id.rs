use std::sync::LazyLock;

use regex::Regex;
use thiserror::Error;

use crate::{EMLError, utils::StringValueData};

/// Regular expression for validating BAG ID values.
static BAG_ID_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d{16}$").expect("Failed to compile BAG ID regex"));

/// A BAG ID for the polling station location.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocationBagId(String);

impl LocationBagId {
    /// Creates a new `LocationBagId` from the given string.
    pub fn new(s: &str) -> Result<Self, InvalidLocationBagId> {
        Self::parse_from_str(s)
    }

    /// Returns the raw value of the BAG ID as a string slice.
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// An invalid BAG ID for the polling station location.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid BAG ID: {0}")]
pub struct InvalidLocationBagId(String);

impl From<InvalidLocationBagId> for EMLError {
    fn from(err: InvalidLocationBagId) -> Self {
        EMLError::value_conversion(err)
    }
}

impl StringValueData for LocationBagId {
    type Error = InvalidLocationBagId;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error> {
        if !s.is_empty() && BAG_ID_RE.is_match(s) {
            Ok(Self(s.into()))
        } else {
            Err(InvalidLocationBagId(s.into()))
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
    fn test_bag_id_regex_compiles() {
        LazyLock::force(&BAG_ID_RE);
    }
}
