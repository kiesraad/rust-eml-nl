use std::sync::LazyLock;

use regex::Regex;
use thiserror::Error;

use crate::{EMLError, EMLValueResultExt, utils::StringValueData};

/// Regular expression for validating ElectionDomainId values.
static ELECTION_DOMAIN_ID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\d{4}|([12]?[0-9]))$").expect("Failed to compile Election Domain ID regex")
});

/// A string of type ElectionDomainId as defined in the EML_NL specification
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ElectionDomainId(Box<str>);

impl ElectionDomainId {
    /// Create a new ElectionDomainId from a string, validating its format
    pub fn new(s: impl AsRef<str>) -> Result<Self, EMLError> {
        StringValueData::parse_from_str(s.as_ref()).wrap_value_error()
    }

    /// Get the raw string value of the ElectionDomainId.
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Error returned when a string could not be parsed as a [`ElectionDomainId`]
#[derive(Debug, Clone, Error)]
#[error("Invalid election domain id: {0}")]
pub struct InvalidElectionDomainIdError(String);

impl From<InvalidElectionDomainIdError> for EMLError {
    fn from(err: InvalidElectionDomainIdError) -> Self {
        EMLError::value_conversion(err)
    }
}

impl StringValueData for ElectionDomainId {
    type Error = InvalidElectionDomainIdError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        if ELECTION_DOMAIN_ID_RE.is_match(s) {
            Ok(ElectionDomainId(s.into()))
        } else {
            Err(InvalidElectionDomainIdError(s.into()))
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
    fn test_election_domain_id_regex_compiles() {
        LazyLock::force(&ELECTION_DOMAIN_ID_RE);
    }

    #[test]
    fn test_valid_election_domain_ids() {
        let valid_ids = ["1", "12", "1234"];
        for id in valid_ids {
            assert!(
                ElectionDomainId::new(id).is_ok(),
                "ElectionDomainId should accept valid id: {}",
                id
            );
        }
    }

    #[test]
    fn test_invalid_election_domain_ids() {
        let invalid_ids = ["", "34", "123", "12345", "abc"];
        for id in invalid_ids {
            assert!(
                ElectionDomainId::new(id).is_err(),
                "ElectionDomainId should reject invalid id: {}",
                id
            );
        }
    }
}
