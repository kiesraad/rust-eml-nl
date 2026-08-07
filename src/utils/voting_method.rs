use thiserror::Error;

use crate::{EMLError, EMLValueResultExt as _, utils::StringValueData};

/// Voting method used in the election.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VotingMethod {
    /// Additional Member System
    AMS,
    /// First Past the Post
    FPP,
    /// Instant Runoff Voting
    IRV,
    /// Norwegian Voting
    NOR,
    /// Optional Preferential Voting
    OPV,
    /// Ranked Choice Voting
    RCV,
    /// Single Preferential Vote
    SPV,
    /// Single Transferable Vote
    STV,
    /// Cumulative Voting
    Cumulative,
    /// Approval Voting
    Approval,
    /// Block Voting
    Block,
    /// Supporter List Voting
    SupporterList,
    /// Partisan Voting
    Partisan,
    /// Supplementary Vote
    SupplementaryVote,
    /// Other Voting Method
    Other,
}

impl VotingMethod {
    /// Create a new VotingMethod from a string, validating its format.
    pub fn new(s: impl AsRef<str>) -> Result<Self, EMLError> {
        Self::from_eml_value(s).wrap_value_error()
    }

    /// Create a VotingMethod from a `&str`, if possible.
    pub fn from_eml_value(s: impl AsRef<str>) -> Result<Self, UnknownVotingMethodError> {
        let data = s.as_ref();
        match data {
            "AMS" => Ok(VotingMethod::AMS),
            "FPP" => Ok(VotingMethod::FPP),
            "IRV" => Ok(VotingMethod::IRV),
            "NOR" => Ok(VotingMethod::NOR),
            "OPV" => Ok(VotingMethod::OPV),
            "RCV" => Ok(VotingMethod::RCV),
            "SPV" => Ok(VotingMethod::SPV),
            "STV" => Ok(VotingMethod::STV),
            "cumulative" => Ok(VotingMethod::Cumulative),
            "approval" => Ok(VotingMethod::Approval),
            "block" => Ok(VotingMethod::Block),
            "supporterlist" => Ok(VotingMethod::SupporterList),
            "partisan" => Ok(VotingMethod::Partisan),
            "supplementaryvote" => Ok(VotingMethod::SupplementaryVote),
            "other" => Ok(VotingMethod::Other),
            _ => Err(UnknownVotingMethodError(data.to_string())),
        }
    }

    /// Get the `&str` representation of this VotingMethod.
    pub fn to_eml_value(&self) -> &'static str {
        match self {
            VotingMethod::AMS => "AMS",
            VotingMethod::FPP => "FPP",
            VotingMethod::IRV => "IRV",
            VotingMethod::NOR => "NOR",
            VotingMethod::OPV => "OPV",
            VotingMethod::RCV => "RCV",
            VotingMethod::SPV => "SPV",
            VotingMethod::STV => "STV",
            VotingMethod::Cumulative => "cumulative",
            VotingMethod::Approval => "approval",
            VotingMethod::Block => "block",
            VotingMethod::SupporterList => "supporterlist",
            VotingMethod::Partisan => "partisan",
            VotingMethod::SupplementaryVote => "supplementaryvote",
            VotingMethod::Other => "other",
        }
    }
}

/// Error returned when an unknown voting method string is encountered.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("Unknown voting method: {0}")]
pub struct UnknownVotingMethodError(String);

impl From<UnknownVotingMethodError> for EMLError {
    fn from(err: UnknownVotingMethodError) -> Self {
        EMLError::value_conversion(err)
    }
}

impl StringValueData for VotingMethod {
    type Error = UnknownVotingMethodError;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voting_method_from_str() {
        assert_eq!(VotingMethod::from_eml_value("AMS"), Ok(VotingMethod::AMS));
        assert_eq!(VotingMethod::from_eml_value("FPP"), Ok(VotingMethod::FPP));
        assert_eq!(VotingMethod::from_eml_value("SPV"), Ok(VotingMethod::SPV));
        assert_eq!(
            VotingMethod::from_eml_value("UNKNOWN"),
            Err(UnknownVotingMethodError("UNKNOWN".to_string()))
        );
    }

    #[test]
    fn test_voting_method_to_str() {
        assert_eq!(VotingMethod::AMS.to_eml_value(), "AMS");
        assert_eq!(VotingMethod::FPP.to_eml_value(), "FPP");
        assert_eq!(VotingMethod::SPV.to_eml_value(), "SPV");
    }
}
