use crate::{EMLError, EMLValueResultExt as _, utils::StringValueData};
use thiserror::Error;

/// Region category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegionCategory {
    /// A 'deelgemeente'. Note that these no longer exist since 2014.
    SubMunicipality,
    /// A 'gemeente', the lowest level of government region in mainland Netherlands.
    Municipality,
    /// A 'kieskring'
    ElectoralDistrict,
    /// A 'provincie'
    Province,
    /// 'provinciaal kieskring'
    /// Note: it is currently unclear when this is used.
    ProvinceElectoralDistrict,
    /// 'provinciaal stembureau'
    /// Note: it is currently unclear when this is used.
    ProvincePollingStation,
    /// The highest level of government, the 'staat'.
    State,
    /// A 'stembureau'
    PollingStation,
    /// A 'waterschap'
    WaterAuthority,
    /// 'waterschap kieskring'
    /// Note: it is currently unclear when this is used.
    WaterAuthorityElectoralDistrict,
    /// 'waterschap gemeente'
    /// Note: it is currently unclear when this is used.
    WaterAuthorityMunicipality,
}

impl RegionCategory {
    /// Create a new RegionCategory from a string, validating its format
    pub fn new(s: impl AsRef<str>) -> Result<Self, EMLError> {
        Self::from_eml_value(s).wrap_value_error()
    }

    /// Create a [`RegionCategory`] from a `&str`, if possible.
    pub fn from_eml_value(s: impl AsRef<str>) -> Result<Self, UnknownRegionCategoryError> {
        let data = s.as_ref();
        match data {
            "DEELGEMEENTE" => Ok(Self::SubMunicipality),
            "GEMEENTE" => Ok(Self::Municipality),
            "KIESKRING" => Ok(Self::ElectoralDistrict),
            "PROVINCIE" => Ok(Self::Province),
            "PROVINCIAAL_KIESKRING" => Ok(Self::ProvinceElectoralDistrict),
            "PROVINCIAAL_STEMBUREAU" => Ok(Self::ProvincePollingStation),
            "STAAT" => Ok(Self::State),
            "STEMBUREAU" => Ok(Self::PollingStation),
            "WATERSCHAP" => Ok(Self::WaterAuthority),
            "WATERSCHAP_KIESKRING" => Ok(Self::WaterAuthorityElectoralDistrict),
            "WATERSCHAP_GEMEENTE" => Ok(Self::WaterAuthorityMunicipality),
            _ => Err(UnknownRegionCategoryError(data.to_string())),
        }
    }

    /// Get the `&str` representation of this [`RegionCategory`].
    pub fn to_eml_value(&self) -> &'static str {
        match self {
            RegionCategory::SubMunicipality => "DEELGEMEENTE",
            RegionCategory::Municipality => "GEMEENTE",
            RegionCategory::ElectoralDistrict => "KIESKRING",
            RegionCategory::Province => "PROVINCIE",
            RegionCategory::ProvinceElectoralDistrict => "PROVINCIAAL_KIESKRING",
            RegionCategory::ProvincePollingStation => "PROVINCIAAL_STEMBUREAU",
            RegionCategory::State => "STAAT",
            RegionCategory::PollingStation => "STEMBUREAU",
            RegionCategory::WaterAuthority => "WATERSCHAP",
            RegionCategory::WaterAuthorityElectoralDistrict => "WATERSCHAP_KIESKRING",
            RegionCategory::WaterAuthorityMunicipality => "WATERSCHAP_GEMEENTE",
        }
    }
}

/// Error returned when an unknown region category string is encountered.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_category_from_str() {
        assert_eq!(
            RegionCategory::from_eml_value("GEMEENTE"),
            Ok(RegionCategory::Municipality)
        );
        assert_eq!(
            RegionCategory::from_eml_value("STAAT"),
            Ok(RegionCategory::State)
        );
        assert_eq!(
            RegionCategory::from_eml_value("WATERSCHAP_GEMEENTE"),
            Ok(RegionCategory::WaterAuthorityMunicipality)
        );
        assert_eq!(
            RegionCategory::from_eml_value("UNKNOWN"),
            Err(UnknownRegionCategoryError("UNKNOWN".to_string()))
        );
    }

    #[test]
    fn test_region_category_to_str() {
        assert_eq!(RegionCategory::Municipality.to_eml_value(), "GEMEENTE");
        assert_eq!(RegionCategory::State.to_eml_value(), "STAAT");
        assert_eq!(
            RegionCategory::WaterAuthorityMunicipality.to_eml_value(),
            "WATERSCHAP_GEMEENTE"
        );
    }
}
