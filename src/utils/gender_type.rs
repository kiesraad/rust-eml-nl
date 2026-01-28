use thiserror::Error;

use crate::utils::StringValueData;

/// Voting method used in the election.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenderType {
    /// Male gender
    Male,
    /// Female gender
    Female,
    /// Gender unknown
    Unknown,
}

impl GenderType {
    /// Create a GenderType from a `&str`, if possible.
    pub fn from_str_value(s: &str) -> Option<Self> {
        match s {
            "male" => Some(GenderType::Male),
            "female" => Some(GenderType::Female),
            "unknown" => Some(GenderType::Unknown),
            _ => None,
        }
    }

    /// Get the `&str` representation of this GenderType.
    pub fn to_str_value(&self) -> &'static str {
        match self {
            GenderType::Male => "male",
            GenderType::Female => "female",
            GenderType::Unknown => "unknown",
        }
    }
}

/// Error returned when an unknown gender type string is encountered.
#[derive(Debug, Clone, Error)]
#[error("Unknown gender type: {0}")]
pub struct UnknownGenderTypeError(String);

impl StringValueData for GenderType {
    type Error = UnknownGenderTypeError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Self::from_str_value(s).ok_or(UnknownGenderTypeError(s.to_string()))
    }

    fn to_raw_value(&self) -> String {
        self.to_str_value().to_string()
    }
}
