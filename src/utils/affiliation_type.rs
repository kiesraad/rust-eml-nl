use thiserror::Error;

use crate::utils::StringValueData;

/// Voting method used in the election.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AffiliationType {
    /// lijstengroep
    GroupOfLists,
    /// stel gelijkluidende lijsten
    SetOfEqualLists,
    /// op zichzelf staande lijst
    StandAloneList,
}

impl AffiliationType {
    /// Create a VotingMethod from a `&str`, if possible.
    pub fn from_str_value(s: &str) -> Option<Self> {
        match s {
            "lijstengroep" => Some(AffiliationType::GroupOfLists),
            "stel gelijkluidende lijsten" => Some(AffiliationType::SetOfEqualLists),
            "op zichzelf staande lijst" => Some(AffiliationType::StandAloneList),
            _ => None,
        }
    }

    /// Get the `&str` representation of this VotingMethod.
    pub fn to_str_value(&self) -> &'static str {
        match self {
            AffiliationType::GroupOfLists => "lijstengroep",
            AffiliationType::SetOfEqualLists => "stel gelijkluidende lijsten",
            AffiliationType::StandAloneList => "op zichzelf staande lijst",
        }
    }
}

/// Error returned when an unknown election category string is encountered.
#[derive(Debug, Clone, Error)]
#[error("Unknown affiliation type: {0}")]
pub struct UnknownAffiliationType(String);

impl StringValueData for AffiliationType {
    type Error = UnknownAffiliationType;
    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Self::from_str_value(s).ok_or(UnknownAffiliationType(s.to_string()))
    }

    fn to_raw_value(&self) -> String {
        self.to_str_value().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_election_category_from_str() {
        assert_eq!(
            AffiliationType::from_str_value("lijstengroep"),
            Some(AffiliationType::GroupOfLists)
        );
        assert_eq!(
            AffiliationType::from_str_value("stel gelijkluidende lijsten"),
            Some(AffiliationType::SetOfEqualLists)
        );
        assert_eq!(
            AffiliationType::from_str_value("op zichzelf staande lijst"),
            Some(AffiliationType::StandAloneList)
        );
        assert_eq!(AffiliationType::from_str_value("UNKNOWN"), None);
    }

    #[test]
    fn test_election_category_to_str() {
        assert_eq!(AffiliationType::GroupOfLists.to_str_value(), "lijstengroep");
        assert_eq!(
            AffiliationType::SetOfEqualLists.to_str_value(),
            "stel gelijkluidende lijsten"
        );
        assert_eq!(
            AffiliationType::StandAloneList.to_str_value(),
            "op zichzelf staande lijst"
        );
    }
}
