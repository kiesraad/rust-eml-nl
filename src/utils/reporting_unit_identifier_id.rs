use std::sync::LazyLock;

use regex::Regex;
use thiserror::Error;

use crate::utils::StringValueData;

/// Regular expression for validating ReportingUnitIdentifier id values.
static REPORTING_UNIT_IDENTIFIER_ID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^((HSB\d+)|((HSB\d+::)?\d{4})|(((HSB\d+::)?\d{4}::)?SB\d+)|(HSB\d+::SB\d+))$")
        .expect("Failed to compile ReportingUnitIdentifier id regex")
});

/// A string of type ReportingUnitIdentifier id as defined in the EML_NL specification
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct ReportingUnitIdentifierId(String);

impl ReportingUnitIdentifierId {
    /// Create a new ReportingUnitIdentifierId from a string, validating its format
    pub fn new(s: impl AsRef<str>) -> Result<Self, InvalidReportingUnitIdentifierIdError> {
        StringValueData::parse_from_str(s.as_ref())
    }

    /// Get the raw string value of the ReportingUnitIdentifierId.
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Error returned when a string could not be parsed as a ReportingUnitIdentifier id
#[derive(Debug, Clone, Error)]
#[error("Invalid ReportingUnitIdentifier id: {0}")]
pub struct InvalidReportingUnitIdentifierIdError(String);

impl StringValueData for ReportingUnitIdentifierId {
    type Error = InvalidReportingUnitIdentifierIdError;
    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        if REPORTING_UNIT_IDENTIFIER_ID_RE.is_match(s) {
            Ok(ReportingUnitIdentifierId(s.to_string()))
        } else {
            Err(InvalidReportingUnitIdentifierIdError(s.to_string()))
        }
    }

    fn to_raw_value(&self) -> String {
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
}
