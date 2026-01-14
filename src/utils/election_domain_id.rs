use std::sync::LazyLock;

use regex::Regex;
use thiserror::Error;

use crate::utils::StringValueData;

/// Regular expression for validating ElectionDomainId values.
static ELECTION_DOMAIN_ID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\d{4}|([12]?[0-9])$").expect("Failed to compile Election Domain ID regex")
});

/// A string of type ElectionDomainId as defined in the EML_NL specification
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct ElectionDomainIdType(String);

impl ElectionDomainIdType {
    /// Create a new ElectionDomainIdType from a string, validating its format
    pub fn new(s: impl AsRef<str>) -> Result<Self, InvalidElectionDomainIdError> {
        StringValueData::parse_from_str(s.as_ref())
    }

    /// Get the raw string value of the ElectionIdType.
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Error returned when a string could not be parsed as a ElectionDomainId
#[derive(Debug, Clone, Error)]
#[error("Invalid ElectionDomainId: {0}")]
pub struct InvalidElectionDomainIdError(String);

impl StringValueData for ElectionDomainIdType {
    type Error = InvalidElectionDomainIdError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        if ELECTION_DOMAIN_ID_RE.is_match(s) {
            Ok(ElectionDomainIdType(s.to_string()))
        } else {
            Err(InvalidElectionDomainIdError(s.to_string()))
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
    fn test_election_domain_id_regex_compiles() {
        LazyLock::force(&ELECTION_DOMAIN_ID_RE);
    }
}
