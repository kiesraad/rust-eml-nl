use crate::{EMLError, EMLValueResultExt as _, utils::StringValueData};
use thiserror::Error;

/// Committee category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommitteeCategory {
    /// Central electoral committee, or 'centraal stembureau' in Dutch.
    CSB,
    /// District electoral committee, or 'hoofdstembureau' in Dutch.
    HSB,
    /// Provincial electoral committee, or 'provinciaal stembureau' in Dutch.
    /// This is used during elections for the 'eerste kamer' (the senate).
    ProvSB,
    /// In Dutch 'plaatselijk stembureau', literally 'local polling station'.
    /// Used before the 'Wet nieuwe procedure vaststelling verkiezingsuitslagen',
    /// which came into effect on 1st of Januari 2023. It is roughly similar
    /// to the GSB ('gemeentelijk stembureau', municipal electoral committee)
    /// that is being used since that date.
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

/// Error returned when an unknown committee category string is encountered.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("Unknown committee category: {0}")]
pub struct UnknownCommitteeCategoryError(String);

impl From<UnknownCommitteeCategoryError> for EMLError {
    fn from(err: UnknownCommitteeCategoryError) -> Self {
        EMLError::value_conversion(err)
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_committee_category_from_str() {
        assert_eq!(
            CommitteeCategory::from_eml_value("CSB"),
            Ok(CommitteeCategory::CSB)
        );
        assert_eq!(
            CommitteeCategory::from_eml_value("PROV_SB"),
            Ok(CommitteeCategory::ProvSB)
        );
        assert_eq!(
            CommitteeCategory::from_eml_value("UNKNOWN"),
            Err(UnknownCommitteeCategoryError("UNKNOWN".to_string()))
        );
    }

    #[test]
    fn test_committee_category_to_str() {
        assert_eq!(CommitteeCategory::CSB.to_eml_value(), "CSB");
        assert_eq!(CommitteeCategory::ProvSB.to_eml_value(), "PROV_SB");
    }
}
