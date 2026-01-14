use thiserror::Error;

use crate::utils::StringValueData;

/// Voting method used in the election.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Supporterlist,
    /// Partisan Voting
    Partisan,
    /// Supplementary Vote
    Supplementaryvote,
    /// Other Voting Method
    Other,
}

impl VotingMethod {
    /// Create a VotingMethod from a `&str`, if possible.
    pub fn from_str_value(s: &str) -> Option<Self> {
        match s {
            "AMS" => Some(VotingMethod::AMS),
            "FPP" => Some(VotingMethod::FPP),
            "IRV" => Some(VotingMethod::IRV),
            "NOR" => Some(VotingMethod::NOR),
            "OPV" => Some(VotingMethod::OPV),
            "RCV" => Some(VotingMethod::RCV),
            "SPV" => Some(VotingMethod::SPV),
            "STV" => Some(VotingMethod::STV),
            "cumulative" => Some(VotingMethod::Cumulative),
            "approval" => Some(VotingMethod::Approval),
            "block" => Some(VotingMethod::Block),
            "supporterlist" => Some(VotingMethod::Supporterlist),
            "partisan" => Some(VotingMethod::Partisan),
            "supplementaryvote" => Some(VotingMethod::Supplementaryvote),
            "other" => Some(VotingMethod::Other),
            _ => None,
        }
    }

    /// Get the `&str` representation of this VotingMethod.
    pub fn to_str_value(&self) -> &'static str {
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
            VotingMethod::Supporterlist => "supporterlist",
            VotingMethod::Partisan => "partisan",
            VotingMethod::Supplementaryvote => "supplementaryvote",
            VotingMethod::Other => "other",
        }
    }
}

/// Error returned when an unknown voting method string is encountered.
#[derive(Debug, Clone, Error)]
#[error("Unknown voting method: {0}")]
pub struct UnknownVotingMethodError(String);

impl StringValueData for VotingMethod {
    type Error = UnknownVotingMethodError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Self::from_str_value(s).ok_or(UnknownVotingMethodError(s.to_string()))
    }

    fn to_raw_value(&self) -> String {
        self.to_str_value().to_string()
    }
}
