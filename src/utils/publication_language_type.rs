use thiserror::Error;

use crate::utils::StringValueData;

/// The publication language of something in a document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PublicationLanguageType {
    /// Dutch language.
    #[default]
    Dutch,

    /// Frisian language.
    Frisian,
}

impl PublicationLanguageType {
    /// Parse a publication language type from its string representation.
    pub fn from_str_value(s: &str) -> Option<Self> {
        match s {
            "nl" => Some(PublicationLanguageType::Dutch),
            "fy" => Some(PublicationLanguageType::Frisian),
            _ => None,
        }
    }

    /// Get the `&str` representation of this PublicationLanguageType.
    pub fn to_str_value(&self) -> &'static str {
        match self {
            PublicationLanguageType::Dutch => "nl",
            PublicationLanguageType::Frisian => "fy",
        }
    }
}

/// Error returned when an unknown publication language type string is encountered.
#[derive(Debug, Clone, Error)]
#[error("Unknown publication language type: {0}")]
pub struct UnknownPublicationLanguageType(String);

impl StringValueData for PublicationLanguageType {
    type Error = UnknownPublicationLanguageType;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Self::from_str_value(s).ok_or(UnknownPublicationLanguageType(s.to_string()))
    }

    fn to_raw_value(&self) -> String {
        self.to_str_value().to_string()
    }
}
