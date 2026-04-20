use thiserror::Error;

use crate::{EMLError, EMLValueResultExt as _, utils::StringValueData};

/// Job title used for a proposer in a nomination document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NominationJobTitle {
    /// inleveraar
    Submitter,
    /// plaatsvervanger van de inleveraar
    DeputySubmitter,
    /// gemachtigde voor het aangaan van lijstencombinaties
    CombinationRepresentative,
    /// plaatsvervanger voor het aangaan van lijstencombinaties
    DeputyCombinationRepresentative,
}

impl NominationJobTitle {
    /// Create a new NominationJobTitle from a string, validating its format.
    pub fn new(s: impl AsRef<str>) -> Result<Self, EMLError> {
        Self::from_eml_value(s).wrap_value_error()
    }

    /// Create a [`NominationJobTitle`] from a `&str`, if possible.
    pub fn from_eml_value(s: impl AsRef<str>) -> Result<Self, UnknownNominationJobTitleError> {
        let data = s.as_ref();
        match data {
            "inleveraar" => Ok(NominationJobTitle::Submitter),
            "plaatsvervanger van de inleveraar" => Ok(NominationJobTitle::DeputySubmitter),
            "gemachtigde voor het aangaan van lijstencombinaties" => {
                Ok(NominationJobTitle::CombinationRepresentative)
            }
            "plaatsvervanger voor het aangaan van lijstencombinaties" => {
                Ok(NominationJobTitle::DeputyCombinationRepresentative)
            }
            _ => Err(UnknownNominationJobTitleError(data.to_string())),
        }
    }

    /// Get the `&str` representation of this [`NominationJobTitle`].
    pub fn to_eml_value(&self) -> &'static str {
        match self {
            NominationJobTitle::Submitter => "inleveraar",
            NominationJobTitle::DeputySubmitter => "plaatsvervanger van de inleveraar",
            NominationJobTitle::CombinationRepresentative => {
                "gemachtigde voor het aangaan van lijstencombinaties"
            }
            NominationJobTitle::DeputyCombinationRepresentative => {
                "plaatsvervanger voor het aangaan van lijstencombinaties"
            }
        }
    }
}

/// Error returned when an unknown nomination job title string is encountered.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("Unknown nomination job title: {0}")]
pub struct UnknownNominationJobTitleError(String);

impl StringValueData for NominationJobTitle {
    type Error = UnknownNominationJobTitleError;

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
    fn test_nomination_job_title_from_str() {
        assert_eq!(
            NominationJobTitle::from_eml_value("inleveraar"),
            Ok(NominationJobTitle::Submitter)
        );
        assert_eq!(
            NominationJobTitle::from_eml_value("plaatsvervanger van de inleveraar"),
            Ok(NominationJobTitle::DeputySubmitter)
        );
        assert_eq!(
            NominationJobTitle::from_eml_value(
                "gemachtigde voor het aangaan van lijstencombinaties"
            ),
            Ok(NominationJobTitle::CombinationRepresentative)
        );
        assert_eq!(
            NominationJobTitle::from_eml_value(
                "plaatsvervanger voor het aangaan van lijstencombinaties"
            ),
            Ok(NominationJobTitle::DeputyCombinationRepresentative)
        );
        assert_eq!(
            NominationJobTitle::from_eml_value("UNKNOWN"),
            Err(UnknownNominationJobTitleError("UNKNOWN".to_string()))
        );
    }

    #[test]
    fn test_nomination_job_title_to_str() {
        assert_eq!(NominationJobTitle::Submitter.to_eml_value(), "inleveraar");
        assert_eq!(
            NominationJobTitle::DeputySubmitter.to_eml_value(),
            "plaatsvervanger van de inleveraar"
        );
        assert_eq!(
            NominationJobTitle::CombinationRepresentative.to_eml_value(),
            "gemachtigde voor het aangaan van lijstencombinaties"
        );
        assert_eq!(
            NominationJobTitle::DeputyCombinationRepresentative.to_eml_value(),
            "plaatsvervanger voor het aangaan van lijstencombinaties"
        );
    }
}
