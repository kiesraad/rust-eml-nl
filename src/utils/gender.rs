use instant_xml::{FromXml, ToXml};
use thiserror::Error;

use crate::{EMLError, EMLValueResultExt as _, utils::StringValueData};

/// Gender of a candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromXml, ToXml)]
#[xml(scalar)]
pub enum Gender {
    /// Male gender
    #[xml(rename = "male")]
    Male,
    /// Female gender
    #[xml(rename = "female")]
    Female,
    /// Gender unknown
    #[xml(rename = "unknown")]
    Unknown,
}

impl Gender {
    /// Create a new Gender from a string, validating its format.
    pub fn new(s: impl AsRef<str>) -> Result<Self, EMLError> {
        Self::from_eml_value(s).wrap_value_error()
    }

    /// Create a Gender from a `&str`, if possible.
    pub fn from_eml_value(s: impl AsRef<str>) -> Result<Self, UnknownGenderError> {
        let data = s.as_ref();
        match data {
            "male" => Ok(Gender::Male),
            "female" => Ok(Gender::Female),
            "unknown" => Ok(Gender::Unknown),
            _ => Err(UnknownGenderError(data.to_string())),
        }
    }

    /// Get the `&str` representation of this Gender.
    pub fn to_eml_value(&self) -> &'static str {
        match self {
            Gender::Male => "male",
            Gender::Female => "female",
            Gender::Unknown => "unknown",
        }
    }
}

/// Error returned when an unknown gender string is encountered.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("Unknown gender: {0}")]
pub struct UnknownGenderError(String);

impl StringValueData for Gender {
    type Error = UnknownGenderError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Self::from_eml_value(s)
    }

    fn to_raw_value(&self) -> String {
        self.to_eml_value().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_gender_types() {
        let valid_genders = ["male", "female", "unknown"];
        for gender in valid_genders {
            assert!(
                Gender::from_eml_value(gender).is_ok(),
                "Gender should accept valid gender: {}",
                gender
            );
        }
    }

    #[test]
    fn test_invalid_gender_types() {
        let invalid_genders = ["", "test", "abc"];
        for gender in invalid_genders {
            assert!(
                Gender::from_eml_value(gender).is_err(),
                "Gender should reject invalid gender: {}",
                gender
            );
        }
    }
}
