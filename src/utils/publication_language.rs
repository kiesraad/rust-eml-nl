use instant_xml::{FromXml, ToXml};
use thiserror::Error;

use crate::{EMLError, EMLValueResultExt as _, utils::StringValueData};

/// The publication language of something in a document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, FromXml, ToXml)]
#[xml(scalar)]
pub enum PublicationLanguage {
    /// Dutch language.
    #[default]
    #[xml(rename = "nl")]
    Dutch,

    /// Frisian language.
    #[xml(rename = "fy")]
    Frisian,
}

impl PublicationLanguage {
    /// Create a new PublicationLanguage from a string, validating its format.
    pub fn new(s: impl AsRef<str>) -> Result<Self, EMLError> {
        Self::from_eml_value(s).wrap_value_error()
    }

    /// Parse a publication language from its string representation.
    pub fn from_eml_value(s: impl AsRef<str>) -> Result<Self, UnknownPublicationLanguageError> {
        let data = s.as_ref();
        match data {
            "nl" => Ok(PublicationLanguage::Dutch),
            "fy" => Ok(PublicationLanguage::Frisian),
            _ => Err(UnknownPublicationLanguageError(data.to_string())),
        }
    }

    /// Get the `&str` representation of this [`PublicationLanguage`].
    pub fn to_eml_value(&self) -> &'static str {
        match self {
            PublicationLanguage::Dutch => "nl",
            PublicationLanguage::Frisian => "fy",
        }
    }
}

/// Error returned when an unknown publication language string is encountered.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("Unknown publication language: {0}")]
pub struct UnknownPublicationLanguageError(String);

impl StringValueData for PublicationLanguage {
    type Error = UnknownPublicationLanguageError;

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
    fn test_valid_publication_languages() {
        let valid_languages = ["nl", "fy"];
        for lang in valid_languages {
            assert!(
                PublicationLanguage::from_eml_value(lang).is_ok(),
                "PublicationLanguage should accept valid language code: {}",
                lang
            );
        }
    }

    #[test]
    fn test_invalid_publication_languages() {
        let invalid_languages = ["", "de", "dutch", "nederlands"];
        for lang in invalid_languages {
            assert!(
                PublicationLanguage::from_eml_value(lang).is_err(),
                "PublicationLanguage should reject invalid language code: {}",
                lang
            );
        }
    }
}
