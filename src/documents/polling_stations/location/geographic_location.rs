use std::sync::LazyLock;

use regex::Regex;
use thiserror::Error;

use crate::{
    EMLError,
    utils::{StringValue, StringValueData},
};

/// The geographic location of a polling station
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeographicLocation {
    /// The latitude in degrees (i.e. EPSG:4326/WGS84)
    pub latitude: StringValue<Coordinate>,

    /// The longitude in degrees (i.e. EPSG:4326/WGS84)
    pub longitude: StringValue<Coordinate>,
}

impl GeographicLocation {
    /// Creates a new geographic location with the given latitude and longitude.
    pub fn new(latitude: impl Into<Coordinate>, longitude: impl Into<Coordinate>) -> Self {
        Self {
            latitude: StringValue::from_value(latitude.into()),
            longitude: StringValue::from_value(longitude.into()),
        }
    }

    /// Returns the coordinates as a tuple of latitude and longitude.
    pub fn coordinates(&self) -> Result<(Coordinate, Coordinate), EMLError> {
        let latitude = self.latitude.cloned_value()?;
        let longitude = self.longitude.cloned_value()?;
        Ok((latitude, longitude))
    }
}

/// Regular expression for validating coordinate values.
static COORDINATE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d{1,2}\.\d{4,}$").expect("Failed to compile coordinate regex"));

/// The lattitude or longitude in degrees (i.e. EPSG:4326/WGS84)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Coordinate(String);

impl Coordinate {
    /// Creates a new `Coordinate` from the given string.
    pub fn new(s: &str) -> Result<Self, InvalidCoordinate> {
        Self::parse_from_str(s)
    }

    /// Returns the raw value of the coordinate as a string slice.
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Invalid coordinate syntax error
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("Invalid coordinate syntax: {0}")]
pub struct InvalidCoordinate(String);

impl From<InvalidCoordinate> for EMLError {
    fn from(err: InvalidCoordinate) -> Self {
        EMLError::value_conversion(err)
    }
}

impl StringValueData for Coordinate {
    type Error = InvalidCoordinate;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error> {
        if !s.is_empty() && COORDINATE_RE.is_match(s) {
            Ok(Self(s.into()))
        } else {
            Err(InvalidCoordinate(s.into()))
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
    fn test_coordinate_regex_compiles() {
        LazyLock::force(&COORDINATE_RE);
    }
}
