use std::{
    fmt::Display,
    num::{NonZeroU64, ParseIntError},
    str::FromStr,
};

use thiserror::Error;

use crate::{EMLError, EMLValueResultExt, utils::StringValueData};

/// A string of type candidate id as defined in the EML_NL specification
///
/// Called CandidateIdType in the schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct CandidateId(NonZeroU64);

impl CandidateId {
    /// Create a new CandidateId.
    pub fn new(value: NonZeroU64) -> Self {
        CandidateId(value)
    }

    /// Create a new CandidateId from a u64 value.
    pub fn from_u64(value: u64) -> Result<Self, InvalidCandidateIdError> {
        let value = NonZeroU64::new(value).ok_or(InvalidCandidateIdError::ZeroInteger)?;
        Ok(CandidateId::new(value))
    }

    /// Get the value of the CandidateId.
    pub fn value(&self) -> NonZeroU64 {
        self.0
    }
}

impl FromStr for CandidateId {
    type Err = EMLError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        StringValueData::parse_from_str(s).wrap_value_error()
    }
}

impl Display for CandidateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Error returned when a string could not be parsed as a CandidateId
#[derive(Debug, Clone, Error)]
pub enum InvalidCandidateIdError {
    /// An invalid string was passed for parsing as an candidate id
    #[error("Failed to parse candidate id: {0}")]
    ParseError(ParseIntError),
    /// The value was a zero integer, which is not allowed for candidate ids
    #[error("Candidate id must be a non-zero positive integer")]
    ZeroInteger,
    /// Candidate id cannot start with a zero
    #[error("Candidate id cannot start with a zero")]
    StartsWithZero,
}

impl From<InvalidCandidateIdError> for EMLError {
    fn from(err: InvalidCandidateIdError) -> Self {
        EMLError::value_conversion(err)
    }
}

impl StringValueData for CandidateId {
    type Error = InvalidCandidateIdError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        if s.starts_with("0") {
            return Err(InvalidCandidateIdError::StartsWithZero);
        }

        let value = u64::from_str(s).map_err(InvalidCandidateIdError::ParseError)?;
        let value = NonZeroU64::new(value).ok_or(InvalidCandidateIdError::ZeroInteger)?;
        Ok(CandidateId::new(value))
    }

    fn to_raw_value(&self) -> Box<str> {
        self.0.to_string().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_candidate_ids() {
        let valid_ids = ["1", "12345"];
        for id in valid_ids {
            assert!(
                CandidateId::from_str(id).is_ok(),
                "CandidateId should accept valid id: {}",
                id
            );
        }
    }

    #[test]
    fn test_invalid_candidate_ids() {
        let invalid_ids = ["", "0", " 123", "0123", "abc", "123abc", "-1"];
        for id in invalid_ids {
            assert!(
                CandidateId::from_str(id).is_err(),
                "CandidateId should reject invalid id: {}",
                id
            );
        }
    }
}
