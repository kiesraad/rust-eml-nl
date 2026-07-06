use crate::EMLError;
use crate::error::EMLValueResultExt;
use crate::utils::StringValueData;
use thiserror::Error;

/// Region category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegionCategory {
    /// Todo: Unknown meaning
    Deelgemeente,
    /// Todo: Unknown meaning
    Gemeente,
    /// Todo: Unknown meaning
    Kieskring,
    /// Todo: Unknown meaning
    Provincie,
    /// Todo: Unknown meaning
    ProvinciaalKieskring,
    /// Todo: Unknown meaning
    ProvinciaalStembureau,
    /// Todo: Unknown meaning
    Staat,
    /// Todo: Unknown meaning
    Stembureau,
    /// Todo: Unknown meaning
    Waterschap,
    /// Todo: Unknown meaning
    WaterschapKieskring,
    /// Todo: Unknown meaning
    WaterschapGemeente,
}

impl RegionCategory {
    /// Create a new ElectionCategory from a string, validating its format
    pub fn new(s: impl AsRef<str>) -> Result<Self, EMLError> {
        Self::from_eml_value(s).wrap_value_error()
    }

    /// Create a [`RegionCategory`] from a `&str`, if possible.
    pub fn from_eml_value(s: impl AsRef<str>) -> Result<Self, UnknownRegionCategoryError> {
        let data = s.as_ref();
        match data {
            "DEELGEMEENTE" => Ok(Self::Deelgemeente),
            "GEMEENTE" => Ok(Self::Gemeente),
            "KIESKRING" => Ok(Self::Kieskring),
            "PROVINCIE" => Ok(Self::Provincie),
            "PROVINCIAAL_KIESKRING" => Ok(Self::ProvinciaalKieskring),
            "PROVINCIAAL_STEMBUREAU" => Ok(Self::ProvinciaalStembureau),
            "STAAT" => Ok(Self::Staat),
            "STEMBUREAU" => Ok(Self::Stembureau),
            "WATERSCHAP" => Ok(Self::Waterschap),
            "WATERSCHAP_KIESKRING" => Ok(Self::WaterschapKieskring),
            "WATERSCHAP_GEMEENTE" => Ok(Self::WaterschapGemeente),
            _ => Err(UnknownRegionCategoryError(data.to_string())),
        }
    }

    /// Get the `&str` representation of this [`RegionCategory`].
    pub fn to_eml_value(&self) -> &'static str {
        match self {
            RegionCategory::Deelgemeente => "DEELGEMEENTE",
            RegionCategory::Gemeente => "GEMEENTE",
            RegionCategory::Kieskring => "KIESKRING",
            RegionCategory::Provincie => "PROVINCIE",
            RegionCategory::ProvinciaalKieskring => "PROVINCIAAL_KIESKRING",
            RegionCategory::ProvinciaalStembureau => "PROVINCIAAL_STEMBUREAU",
            RegionCategory::Staat => "STAAT",
            RegionCategory::Stembureau => "STEMBUREAU",
            RegionCategory::Waterschap => "WATERSCHAP",
            RegionCategory::WaterschapKieskring => "WATERSCHAP_KIESKRING",
            RegionCategory::WaterschapGemeente => "WATERSCHAP_GEMEENTE",
        }
    }
}

/// Error returned when an unknown election category string is encountered.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("Unknown region category: {0}")]
pub struct UnknownRegionCategoryError(String);

impl StringValueData for RegionCategory {
    type Error = UnknownRegionCategoryError;

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
