use crate::{EMLError, EMLValueResultExt as _, utils::StringValueData};
use thiserror::Error;

/// Region category
///
/// Note: these are ordered from highest to lowest, so a category that sorts
/// before another category sits at a higher level in the election tree. Use
/// [`RegionCategory::is_higher_level_than`] rather than comparing directly, since the
/// direction of the comparison is easy to get wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RegionCategory {
    /// The highest level of government, the 'staat'.
    State,
    /// A 'waterschap'
    WaterAuthority,
    /// A 'provincie'
    Province,
    /// A 'kiescollege'
    /// Note: for EK elections before EML_NL v1.3, these appear as [`Self::Province`] instead.
    ElectoralCollege,
    /// 'provinciaal stembureau'
    /// Note: these should not appear in regular EML_NL documents, and should be
    /// replaced with [`Self::PollingStation`] instead.
    ProvincePollingStation,
    /// A 'kieskring'
    ElectoralDistrict,
    /// 'waterschap kieskring'
    /// Note: these should not appear in regular EML_NL documents, and should be
    /// replaced with [`Self::ElectoralDistrict`] instead.
    WaterAuthorityElectoralDistrict,
    /// 'provinciaal kieskring'
    /// Note: these should not appear in regular EML_NL documents, and should be
    /// replaced with [`Self::ElectoralDistrict`] instead.
    ProvinceElectoralDistrict,
    /// 'waterschap gemeente'
    /// Note: these should not appear in regular EML_NL documents, and should be
    /// replaced with [`Self::Municipality`] instead.
    WaterAuthorityMunicipality,
    /// A 'gemeente', the lowest level of government region in mainland Netherlands.
    Municipality,
    /// An 'eilandsgemeente' (more commonly known as  an "openbaar lichaam")
    /// Note: these should not appear in regular EML_NL documents, and should be
    /// replaced with [`Self::Municipality`] instead.
    IslandMunicipality,
    /// A 'deelgemeente'. Note that these no longer exist since 2014.
    SubMunicipality,
    /// A 'stembureau'
    PollingStation,
}

impl RegionCategory {
    /// Create a new RegionCategory from a string, validating its format
    pub fn new(s: impl AsRef<str>) -> Result<Self, EMLError> {
        Self::from_eml_value(s).wrap_value_error()
    }

    /// Whether this region category sits at a higher level in the election tree
    /// than the given region category.
    ///
    /// [`RegionCategory::State`] is the highest level and
    /// [`RegionCategory::PollingStation`] the lowest.
    pub fn is_higher_level_than(self, other: Self) -> bool {
        self < other
    }

    /// Create a [`RegionCategory`] from a `&str`, if possible.
    pub fn from_eml_value(s: impl AsRef<str>) -> Result<Self, UnknownRegionCategoryError> {
        let data = s.as_ref();
        match data {
            "STAAT" => Ok(Self::State),
            "WATERSCHAP" => Ok(Self::WaterAuthority),
            "PROVINCIE" => Ok(Self::Province),
            "KIESCOLLEGE" => Ok(Self::ElectoralCollege),
            "KIESKRING" => Ok(Self::ElectoralDistrict),
            "WATERSCHAP_KIESKRING" => Ok(Self::WaterAuthorityElectoralDistrict),
            "PROVINCIAAL_KIESKRING" => Ok(Self::ProvinceElectoralDistrict),
            "PROVINCIAAL_STEMBUREAU" => Ok(Self::ProvincePollingStation),
            "WATERSCHAP_GEMEENTE" => Ok(Self::WaterAuthorityMunicipality),
            "GEMEENTE" => Ok(Self::Municipality),
            "EILANDSGEMEENTE" => Ok(Self::IslandMunicipality),
            "DEELGEMEENTE" => Ok(Self::SubMunicipality),
            "STEMBUREAU" => Ok(Self::PollingStation),
            _ => Err(UnknownRegionCategoryError(data.to_string())),
        }
    }

    /// Get the `&str` representation of this [`RegionCategory`].
    pub fn to_eml_value(&self) -> &'static str {
        match self {
            RegionCategory::State => "STAAT",
            RegionCategory::WaterAuthority => "WATERSCHAP",
            RegionCategory::Province => "PROVINCIE",
            RegionCategory::ElectoralCollege => "KIESCOLLEGE",
            RegionCategory::ElectoralDistrict => "KIESKRING",
            RegionCategory::WaterAuthorityElectoralDistrict => "WATERSCHAP_KIESKRING",
            RegionCategory::ProvinceElectoralDistrict => "PROVINCIAAL_KIESKRING",
            RegionCategory::ProvincePollingStation => "PROVINCIAAL_STEMBUREAU",
            RegionCategory::WaterAuthorityMunicipality => "WATERSCHAP_GEMEENTE",
            RegionCategory::Municipality => "GEMEENTE",
            RegionCategory::IslandMunicipality => "EILANDSGEMEENTE",
            RegionCategory::SubMunicipality => "DEELGEMEENTE",
            RegionCategory::PollingStation => "STEMBUREAU",
        }
    }
}

/// Error returned when an unknown region category string is encountered.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("Unknown region category: {0}")]
pub struct UnknownRegionCategoryError(String);

impl From<UnknownRegionCategoryError> for EMLError {
    fn from(err: UnknownRegionCategoryError) -> Self {
        EMLError::value_conversion(err)
    }
}

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
    fn test_region_category_levels() {
        // The state is the highest level, the polling station the lowest.
        assert!(RegionCategory::State.is_higher_level_than(RegionCategory::Municipality));
        assert!(
            RegionCategory::ElectoralDistrict.is_higher_level_than(RegionCategory::PollingStation)
        );
        assert!(!RegionCategory::Municipality.is_higher_level_than(RegionCategory::State));
        assert!(!RegionCategory::PollingStation.is_higher_level_than(RegionCategory::State));

        // No category sits above itself.
        assert!(!RegionCategory::Municipality.is_higher_level_than(RegionCategory::Municipality));
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
