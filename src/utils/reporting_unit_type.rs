use thiserror::Error;

use crate::{
    EMLError, EMLValueResultExt as _, EMLVersion, EMLVersionRange, NS_KR,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName},
    utils::{StringValue, StringValueData},
};

/// The type of reporting unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReportingUnitType {
    /// A reporting unit with a fixed location.
    FixedLocation,
    /// A reporting unit can move around different locations.
    Mobile,
    /// A 'special' reporting unit.
    Special,
}

impl ReportingUnitType {
    /// Create a new ReportingUnitType from a string, validating its format.
    pub fn new(s: impl AsRef<str>) -> Result<Self, EMLError> {
        Self::from_eml_value(s).wrap_value_error()
    }

    /// Create a ReportingUnitType from a `&str`, if possible.
    pub fn from_eml_value(s: impl AsRef<str>) -> Result<Self, UnknownReportingUnitTypeError> {
        let data = s.as_ref();
        match data {
            "FixedLocation" => Ok(ReportingUnitType::FixedLocation),
            "Mobile" => Ok(ReportingUnitType::Mobile),
            "Special" => Ok(ReportingUnitType::Special),
            _ => Err(UnknownReportingUnitTypeError(data.to_string())),
        }
    }

    /// Get the `&str` representation of this ReportingUnitType.
    pub fn to_eml_value(&self) -> &'static str {
        match self {
            ReportingUnitType::FixedLocation => "FixedLocation",
            ReportingUnitType::Mobile => "Mobile",
            ReportingUnitType::Special => "Special",
        }
    }
}

/// Error returned when an unknown reporting unit type string is encountered.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("Unknown reporting unit type: {0}")]
pub struct UnknownReportingUnitTypeError(String);

impl From<UnknownReportingUnitTypeError> for EMLError {
    fn from(err: UnknownReportingUnitTypeError) -> Self {
        EMLError::value_conversion(err)
    }
}

impl StringValueData for ReportingUnitType {
    type Error = UnknownReportingUnitTypeError;

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

impl EMLElement for StringValue<ReportingUnitType> {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ReportingUnitType", Some(NS_KR));
    const EML_VERSIONS: EMLVersionRange = EMLVersionRange::since(EMLVersion::V1_3);

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        elem.string_value()
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer.text(self.raw().as_ref())?.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_reporting_unit_types() {
        let valid_types = ["FixedLocation", "Mobile", "Special"];
        for type_name in valid_types {
            assert!(
                ReportingUnitType::from_eml_value(type_name).is_ok(),
                "ReportingUnitType should accept valid type: {}",
                type_name
            );
        }
    }

    #[test]
    fn test_invalid_reporting_unit_types() {
        let invalid_types = ["", "test", "abc"];
        for type_name in invalid_types {
            assert!(
                ReportingUnitType::from_eml_value(type_name).is_err(),
                "ReportingUnitType should reject invalid type: {}",
                type_name
            );
        }
    }
}
