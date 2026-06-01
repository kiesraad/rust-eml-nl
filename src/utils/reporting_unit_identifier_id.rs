use std::sync::LazyLock;

use regex::Regex;
use thiserror::Error;

use crate::{EMLError, EMLValueResultExt, utils::StringValueData};

/// Regular expression for validating ReportingUnitIdentifier id values.
static REPORTING_UNIT_IDENTIFIER_ID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^((HSB\d+)|((HSB\d+::)?\d{4})|(((HSB\d+::)?\d{4}::)?SB\d+)|(HSB\d+::SB\d+))$")
        .expect("Failed to compile ReportingUnitIdentifier id regex")
});

/// A string of type ReportingUnitIdentifier id as defined in the EML_NL specification
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ReportingUnitIdentifierId(Box<str>);

impl ReportingUnitIdentifierId {
    /// Create a new ReportingUnitIdentifierId from a string, validating its format
    pub fn new(s: impl AsRef<str>) -> Result<Self, EMLError> {
        StringValueData::parse_from_str(s.as_ref()).wrap_value_error()
    }

    /// Get the raw string value of the ReportingUnitIdentifierId.
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Error returned when a string could not be parsed as a [`ReportingUnitIdentifierId`]
#[derive(Debug, Clone, Error)]
#[error("Invalid reporting unit identifier id: {0}")]
pub struct InvalidReportingUnitIdentifierIdError(String);

impl StringValueData for ReportingUnitIdentifierId {
    type Error = InvalidReportingUnitIdentifierIdError;
    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        if REPORTING_UNIT_IDENTIFIER_ID_RE.is_match(s) {
            Ok(ReportingUnitIdentifierId(s.into()))
        } else {
            Err(InvalidReportingUnitIdentifierIdError(s.into()))
        }
    }

    fn to_raw_value(&self) -> Box<str> {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reporting_unit_identifier_id_regex_compiles() {
        LazyLock::force(&REPORTING_UNIT_IDENTIFIER_ID_RE);
    }

    #[test]
    fn test_valid_reporting_unit_identifier_ids() {
        let valid_ids = [
            "HSB123",
            "HSB123::5678",
            "5678",
            "SB456",
            "HSB123::SB456",
            "HSB123::5678::SB456",
        ];
        for id in valid_ids {
            assert!(
                ReportingUnitIdentifierId::new(id).is_ok(),
                "ReportingUnitIdentifierId should accept valid id: {}",
                id
            );
        }
    }

    #[test]
    fn test_invalid_reporting_unit_identifier_ids() {
        let invalid_ids = [
            "",
            "HSB",
            "HSB::123",
            "HSB123::",
            "::5678",
            "HSB123::5678::",
            "HSB123::5678::SB",
        ];
        for id in invalid_ids {
            assert!(
                ReportingUnitIdentifierId::new(id).is_err(),
                "ReportingUnitIdentifierId should reject invalid id: {}",
                id
            );
        }
    }
}
