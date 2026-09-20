use std::sync::LazyLock;

use regex::Regex;
use thiserror::Error;

use crate::utils::StringValueData;

/// Regular expression for validating website values.
static WEBSITE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^https?://.+$").expect("Failed to compile website regex"));

/// A website url for the polling station location.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WebsiteType(String);

impl WebsiteType {
    /// Creates a new `LocationWebsite` from the given string.
    pub fn new(s: &str) -> Result<Self, InvalidLocationWebsite> {
        Self::parse_from_str(s)
    }

    /// Returns the raw value of the website as a string slice.
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// An invalid website url for the polling station location.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid website url: {0}")]
pub struct InvalidLocationWebsite(String);

impl StringValueData for WebsiteType {
    type Error = InvalidLocationWebsite;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error> {
        if !s.is_empty() && WEBSITE_RE.is_match(s) {
            Ok(Self(s.into()))
        } else {
            Err(InvalidLocationWebsite(s.into()))
        }
    }

    fn to_raw_value(&self) -> Box<str> {
        self.0.clone().into_boxed_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_website_regex_compiles() {
        LazyLock::force(&WEBSITE_RE);
    }
}
