use crate::{EMLError, EMLErrorKind, NS_KR, io::QualifiedName};

/// Describes the EML_NL version of the EML standard.
///
/// Since EML 1.3, EML_NL adds a `kr:Schema` element within the top level
/// element to indicate the specific EML_NL schema version in use. If this
/// element is not present, the legacy unversioned EML_NL schema is assumed,
/// in practice this means that this document should follow the EML_NL 1.2.2
/// schema.
///
/// Note: newer EML_NL version must always be appended at the end of this enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum EMLVersion {
    /// A legacy unversioned EML_NL document, assumed to follow EML_NL 1.2.2
    V1_2_2,

    /// An EML_NL version 1.3 document
    #[default]
    V1_3,
}

impl EMLVersion {
    /// Name of the element that is used to indicate the EML_NL schema version.
    pub const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Schema", Some(NS_KR));

    /// Returns the string representation of this EML version, if one is known.
    ///
    /// Returns `None` for [`EMLVersion::V1_2_2`].
    pub fn to_str(&self) -> Option<&'static str> {
        match self {
            EMLVersion::V1_2_2 => None,
            EMLVersion::V1_3 => Some("1.3"),
        }
    }

    /// Returns `true` if this is an EML_NL version 1.3 document.
    pub fn is_v1_3(&self) -> bool {
        matches!(self, EMLVersion::V1_3)
    }

    /// Returns `true` if this is a legacy unversioned EML_NL document.
    pub fn is_v1_2_2(&self) -> bool {
        matches!(self, EMLVersion::V1_2_2)
    }
}

impl std::fmt::Display for EMLVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str().unwrap_or("1.2.2"))
    }
}

impl std::str::FromStr for EMLVersion {
    type Err = UnsupportedEMLVersion;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1.3" => Ok(EMLVersion::V1_3),
            _ => Err(UnsupportedEMLVersion(s.to_owned())),
        }
    }
}

/// EML_NL version is not known to this version of the library.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Unsupported EML_NL version: {0}")]
pub struct UnsupportedEMLVersion(String);

impl From<UnsupportedEMLVersion> for EMLError {
    fn from(value: UnsupportedEMLVersion) -> Self {
        EMLErrorKind::UnsupportedEMLVersion(value.0).without_span()
    }
}

/// A range of EML_NL versions for which an EML_NL feature is supported.
pub struct EMLVersionRange {
    /// Inclusive lower bound of the range (i.e. first version that supports the feature)
    since: Option<EMLVersion>,
    /// Exclusive upper bound of the range (i.e. first version that does not support the feature)
    until: Option<EMLVersion>,
}

impl EMLVersionRange {
    /// A range that covers all EML_NL versions.
    pub const ALL: Self = Self {
        since: None,
        until: None,
    };

    /// A range that covers all EML_NL versions since the given version (inclusive).
    pub const fn since(version: EMLVersion) -> Self {
        Self {
            since: Some(version),
            until: None,
        }
    }

    /// A range that covers all EML_NL versions until the given version (exclusive).
    pub const fn until(version: EMLVersion) -> Self {
        Self {
            since: None,
            until: Some(version),
        }
    }

    /// A range that covers all EML_NL versions between the given versions
    /// (including the lower bound, but excluding the upper bound).
    pub const fn between(version: EMLVersion, until: EMLVersion) -> Self {
        Self {
            since: Some(version),
            until: Some(until),
        }
    }

    /// Returns whether the given version is within this range.
    pub fn contains(&self, version: EMLVersion) -> bool {
        match (self.since, self.until) {
            (None, None) => true,
            (Some(since), Some(until)) => version >= since && version < until,
            (Some(since), None) => version >= since,
            (None, Some(until)) => version < until,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eml_version_to_str() {
        assert_eq!(EMLVersion::V1_3.to_str(), Some("1.3"));
        assert_eq!(EMLVersion::V1_2_2.to_str(), None);
    }

    #[test]
    fn test_eml_version_from_str() {
        assert_eq!("1.3".parse::<EMLVersion>(), Ok(EMLVersion::V1_3));
        assert!("1.2".parse::<EMLVersion>().is_err());
    }

    #[test]
    fn test_comparison_of_eml_versions() {
        assert!(EMLVersion::V1_3 > EMLVersion::V1_2_2);
        assert!(EMLVersion::V1_3 >= EMLVersion::V1_3);
        assert!(EMLVersion::V1_2_2 <= EMLVersion::V1_3);
        assert!(EMLVersion::V1_2_2 < EMLVersion::V1_3);
        assert!(EMLVersion::V1_3 != EMLVersion::V1_2_2);
        assert!(EMLVersion::V1_3 == EMLVersion::V1_3);
    }
}
