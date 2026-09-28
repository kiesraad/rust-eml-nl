use thiserror::Error;

use crate::{EMLError, utils::StringValueData};

/// The usage of the building where the polling station is located
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BuildingUsage {
    /// "Wonen"
    Residential,

    /// "Bijeenkomst"
    Meeting,

    /// "Winkel"
    Shop,

    /// "Gezondheidszorg"
    Healthcare,

    /// "Kantoor"
    Office,

    /// "Logies"
    Accommodation,

    /// "Industrie"
    Industrial,

    /// "Onderwijs"
    Education,

    /// "Sport"
    Sport,

    /// "Overig"
    Other,

    /// "Cel"
    Cell,
}

impl BuildingUsage {
    /// Create a BuildingUsage from a `&str`, if possible.
    pub fn from_eml_value(s: impl AsRef<str>) -> Result<Self, InvalidBuildingUsage> {
        let data = s.as_ref();
        match data {
            "Wonen" => Ok(BuildingUsage::Residential),
            "Bijeenkomst" => Ok(BuildingUsage::Meeting),
            "Winkel" => Ok(BuildingUsage::Shop),
            "Gezondheidszorg" => Ok(BuildingUsage::Healthcare),
            "Kantoor" => Ok(BuildingUsage::Office),
            "Logies" => Ok(BuildingUsage::Accommodation),
            "Industrie" => Ok(BuildingUsage::Industrial),
            "Onderwijs" => Ok(BuildingUsage::Education),
            "Sport" => Ok(BuildingUsage::Sport),
            "Overig" => Ok(BuildingUsage::Other),
            "Cel" => Ok(BuildingUsage::Cell),
            _ => Err(InvalidBuildingUsage(data.to_string())),
        }
    }

    /// Get the `&str` representation of this building usage value.
    pub fn to_eml_value(&self) -> &'static str {
        match self {
            BuildingUsage::Residential => "Wonen",
            BuildingUsage::Meeting => "Bijeenkomst",
            BuildingUsage::Shop => "Winkel",
            BuildingUsage::Healthcare => "Gezondheidszorg",
            BuildingUsage::Office => "Kantoor",
            BuildingUsage::Accommodation => "Logies",
            BuildingUsage::Industrial => "Industrie",
            BuildingUsage::Education => "Onderwijs",
            BuildingUsage::Sport => "Sport",
            BuildingUsage::Other => "Overig",
            BuildingUsage::Cell => "Cel",
        }
    }
}

/// An error that occurs when the building usage is unknown
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("Unknown building usage: {0}")]
pub struct InvalidBuildingUsage(String);

impl From<InvalidBuildingUsage> for EMLError {
    fn from(err: InvalidBuildingUsage) -> Self {
        EMLError::value_conversion(err)
    }
}

impl StringValueData for BuildingUsage {
    type Error = InvalidBuildingUsage;

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
