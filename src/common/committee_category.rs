use crate::EMLError;
use crate::error::EMLValueResultExt;
use crate::utils::StringValueData;
use thiserror::Error;

/// Committee category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommitteeCategory {
    /// Todo: Unknown meaning
    CSB,
    /// Todo: Unknown meaning
    HSB,
    /// Todo: Unknown meaning
    ProvSB,
    /// Todo: Unknown meaning
    PSB,
}

impl CommitteeCategory {
    /// Create a new CommitteeCategory from a string, validating its format
    pub fn new(s: impl AsRef<str>) -> Result<Self, EMLError> {
        Self::from_eml_value(s).wrap_value_error()
    }

    /// Create a [`CommitteeCategory`] from a `&str`, if possible.
    pub fn from_eml_value(s: impl AsRef<str>) -> Result<Self, UnknownCommitteeCategoryError> {
        let data = s.as_ref();
        match data {
            "CSB" => Ok(Self::CSB),
            "HSB" => Ok(Self::HSB),
            "PROV_SB" => Ok(Self::ProvSB),
            "PSB" => Ok(Self::PSB),
            _ => Err(UnknownCommitteeCategoryError(data.to_string())),
        }
    }

    /// Get the `&str` representation of this [`CommitteeCategory`].
    pub fn to_eml_value(&self) -> &'static str {
        match self {
            CommitteeCategory::CSB => "CSB",
            CommitteeCategory::HSB => "HSB",
            CommitteeCategory::ProvSB => "PROV_SB",
            CommitteeCategory::PSB => "PSB",
        }
    }
}

/// Error returned when an unknown election category string is encountered.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("Unknown committee category: {0}")]
pub struct UnknownCommitteeCategoryError(String);

impl StringValueData for CommitteeCategory {
    type Error = UnknownCommitteeCategoryError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Self::from_eml_value(s)
    }

    fn to_raw_value(&self) -> Box<str> {
        self.to_eml_value().into()
    }
}
