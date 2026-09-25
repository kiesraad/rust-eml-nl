use std::sync::LazyLock;

use regex::Regex;
use thiserror::Error;

use crate::{
    EMLError,
    utils::{StringValue, StringValueData},
};

/// The RD (Rijksdriehoek) geographic location of a polling station
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RDGeographicLocation {
    /// The X coordinate in RD (Rijksdriehoek, i.e. EPSG:28992)
    pub x: StringValue<RDCoordinate>,

    /// The Y coordinate in RD (Rijksdriehoek, i.e. EPSG:28992)
    pub y: StringValue<RDCoordinate>,
}

impl RDGeographicLocation {
    /// Creates a new geographic location with the given latitude and longitude.
    pub fn new(x: impl Into<RDCoordinate>, y: impl Into<RDCoordinate>) -> Self {
        Self {
            x: StringValue::from_value(x.into()),
            y: StringValue::from_value(y.into()),
        }
    }

    /// Returns the coordinates as a tuple of latitude and longitude.
    pub fn coordinates(&self) -> Result<(RDCoordinate, RDCoordinate), EMLError> {
        let x = self.x.cloned_value()?;
        let y = self.y.cloned_value()?;
        Ok((x, y))
    }
}

/// Regular expression for validating RD coordinate values.
static RD_COORDINATE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\d{1,6}(.\d{1,})?$").expect("Failed to compile RD coordinate regex")
});

/// Invalid RD coordinate syntax error
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("Invalid RD coordinate syntax: {0}")]
pub struct InvalidRDCoordinate(String);

impl From<InvalidRDCoordinate> for EMLError {
    fn from(err: InvalidRDCoordinate) -> Self {
        EMLError::value_conversion(err)
    }
}

/// An RD coordinate in meters (rijksdriehoek, i.e. EPSG:28992)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RDCoordinate(String);

impl RDCoordinate {
    /// Creates a new `RDCoordinate` from the given string.
    pub fn new(s: &str) -> Result<Self, InvalidRDCoordinate> {
        Self::parse_from_str(s)
    }

    /// Returns the raw value of the RD coordinate as a string slice.
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl StringValueData for RDCoordinate {
    type Error = InvalidRDCoordinate;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error> {
        if !s.is_empty() && RD_COORDINATE_RE.is_match(s) {
            Ok(Self(s.into()))
        } else {
            Err(InvalidRDCoordinate(s.into()))
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
    fn test_rd_coordinate_regex_compiles() {
        LazyLock::force(&RD_COORDINATE_RE);
    }
}
