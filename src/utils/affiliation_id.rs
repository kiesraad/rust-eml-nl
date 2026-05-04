use std::{
    num::{NonZeroU64, ParseIntError},
    str::FromStr,
};

use thiserror::Error;

use crate::{EMLError, EMLValueResultExt, utils::StringValueData};

/// A string of type affiliation id as defined in the EML_NL specification
///
/// Called AffiliationIdType in the schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct AffiliationId(NonZeroU64);

impl AffiliationId {
    /// Create a new AffiliationId.
    pub fn new(value: NonZeroU64) -> Self {
        AffiliationId(value)
    }

    /// Create a new AffiliationId from a u64 value.
    pub fn from_u64(value: u64) -> Result<Self, InvalidAffiliationIdError> {
        let value = NonZeroU64::new(value).ok_or(InvalidAffiliationIdError::ZeroInteger)?;
        Ok(AffiliationId::new(value))
    }

    /// Get the value of the AffiliationId.
    pub fn value(&self) -> NonZeroU64 {
        self.0
    }
}

impl FromStr for AffiliationId {
    type Err = EMLError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        StringValueData::parse_from_str(s).wrap_value_error()
    }
}

/// Error returned when a string could not be parsed as a AffiliationId
#[derive(Debug, Clone, Error)]
pub enum InvalidAffiliationIdError {
    /// An invalid string was passed for parsing as an affiliation id
    #[error("Failed to parse affiliation id: {0}")]
    ParseError(ParseIntError),
    /// The value was a zero integer, which is not allowed for affiliation ids
    #[error("Affiliation id must be a non-zero positive integer")]
    ZeroInteger,
    /// Affiliation id cannot start with a zero
    #[error("Affiliation id cannot start with a zero")]
    StartsWithZero,
}

impl StringValueData for AffiliationId {
    type Error = InvalidAffiliationIdError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        if s.starts_with("0") {
            return Err(InvalidAffiliationIdError::StartsWithZero);
        }

        let value = u64::from_str(s).map_err(InvalidAffiliationIdError::ParseError)?;
        let value = NonZeroU64::new(value).ok_or(InvalidAffiliationIdError::ZeroInteger)?;
        Ok(AffiliationId::new(value))
    }

    fn to_raw_value(&self) -> String {
        self.0.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_affiliation_ids() {
        let valid_ids = ["1", "12345"];
        for id in valid_ids {
            assert!(
                AffiliationId::from_str(id).is_ok(),
                "AffiliationId should accept valid id: {}",
                id
            );
        }
    }

    #[test]
    fn test_invalid_affiliation_ids() {
        let invalid_ids = ["0", " 0123", "0123", "abc", "", "-1"];
        for id in invalid_ids {
            assert!(
                AffiliationId::from_str(id).is_err(),
                "AffiliationId should reject invalid id: {}",
                id
            );
        }
    }
}
