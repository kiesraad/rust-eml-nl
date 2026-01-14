use thiserror::Error;

use crate::utils::StringValueData;

/// Voting method used in the election.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElectionCategory {
    /// Eerste Kamer
    EK,
    /// Tweede Kamer
    TK,
    /// Europese Parlement
    EP,
    /// Provinciale Staten
    PS,
    /// Waterschapsverkiezingen
    AB,
    /// Gemeenteraad
    GR,
    /// Bestuurscommissie (Amsterdam, unused)
    BC,
    /// Gebiedscommissie (Rotterdam, unused)
    GC,
    /// Eilandsraad
    ER,
    /// Todo: Unknown meaning
    NR,
    /// Todo: Unknown meaning
    PR,
    /// Todo: Unknown meaning
    LR,
    /// Todo: Unknown meaning
    IR,
}

impl ElectionCategory {
    /// Create a VotingMethod from a `&str`, if possible.
    pub fn from_str_value(s: &str) -> Option<Self> {
        match s {
            "EK" => Some(ElectionCategory::EK),
            "TK" => Some(ElectionCategory::TK),
            "EP" => Some(ElectionCategory::EP),
            "PS" => Some(ElectionCategory::PS),
            "AB" => Some(ElectionCategory::AB),
            "GR" => Some(ElectionCategory::GR),
            "BC" => Some(ElectionCategory::BC),
            "GC" => Some(ElectionCategory::GC),
            "ER" => Some(ElectionCategory::ER),
            "NR" => Some(ElectionCategory::NR),
            "PR" => Some(ElectionCategory::PR),
            "LR" => Some(ElectionCategory::LR),
            "IR" => Some(ElectionCategory::IR),
            _ => None,
        }
    }

    /// Get the `&str` representation of this VotingMethod.
    pub fn to_str_value(&self) -> &'static str {
        match self {
            ElectionCategory::EK => "EK",
            ElectionCategory::TK => "TK",
            ElectionCategory::EP => "EP",
            ElectionCategory::PS => "PS",
            ElectionCategory::AB => "AB",
            ElectionCategory::GR => "GR",
            ElectionCategory::BC => "BC",
            ElectionCategory::GC => "GC",
            ElectionCategory::ER => "ER",
            ElectionCategory::NR => "NR",
            ElectionCategory::PR => "PR",
            ElectionCategory::LR => "LR",
            ElectionCategory::IR => "IR",
        }
    }
}

/// Error returned when an unknown election category string is encountered.
#[derive(Debug, Clone, Error)]
#[error("Unknown election category: {0}")]
pub struct UnknownElectionCategory(String);

impl StringValueData for ElectionCategory {
    type Error = UnknownElectionCategory;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Self::from_str_value(s).ok_or(UnknownElectionCategory(s.to_string()))
    }

    fn to_raw_value(&self) -> String {
        self.to_str_value().to_string()
    }
}

/// Subcategory of the election, providing more specific information about the type of election.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElectionSubcategory {
    /// Provinciale Staten (one electoral district)
    PS1,
    /// Provinciale Staten (multiple electoral districts)
    PS2,
    /// Waterschapsverkiezingen (less than 19 seats)
    AB1,
    /// Waterschapsverkiezingen (19 or more seats)
    AB2,
    /// Gemeenteraad (less than 19 seats)
    GR1,
    /// Gemeenteraad (19 or more seats)
    GR2,
    /// Bestuurscommissie (Amsterdam, unused)
    BC,
    /// Gebiedscommissie (Rotterdam, unused)
    GC,
    /// Eilandsraad (less than 19 seats, all eilandraden have this)
    ER1,
    /// Tweede kamer
    TK,
    /// Eerste kamer
    EK,
    /// Europees Parlement
    EP,
    /// Todo: Unknown meaning
    NR,
    /// Todo: Unknown meaning
    PR,
    /// Todo: Unknown meaning
    LR,
    /// Todo: Unknown meaning
    IR,
}

impl ElectionSubcategory {
    /// Create a ElectionSubcategory from a `&str`, if possible.
    pub fn from_str_value(s: &str) -> Option<Self> {
        match s {
            "PS1" => Some(ElectionSubcategory::PS1),
            "PS2" => Some(ElectionSubcategory::PS2),
            "AB1" => Some(ElectionSubcategory::AB1),
            "AB2" => Some(ElectionSubcategory::AB2),
            "GR1" => Some(ElectionSubcategory::GR1),
            "GR2" => Some(ElectionSubcategory::GR2),
            "BC" => Some(ElectionSubcategory::BC),
            "GC" => Some(ElectionSubcategory::GC),
            "ER1" => Some(ElectionSubcategory::ER1),
            "TK" => Some(ElectionSubcategory::TK),
            "EK" => Some(ElectionSubcategory::EK),
            "EP" => Some(ElectionSubcategory::EP),
            "NR" => Some(ElectionSubcategory::NR),
            "PR" => Some(ElectionSubcategory::PR),
            "LR" => Some(ElectionSubcategory::LR),
            "IR" => Some(ElectionSubcategory::IR),
            _ => None,
        }
    }

    /// Get the `&str` representation of this ElectionSubcategory.
    pub fn to_str_value(&self) -> &'static str {
        match self {
            ElectionSubcategory::PS1 => "PS1",
            ElectionSubcategory::PS2 => "PS2",
            ElectionSubcategory::AB1 => "AB1",
            ElectionSubcategory::AB2 => "AB2",
            ElectionSubcategory::GR1 => "GR1",
            ElectionSubcategory::GR2 => "GR2",
            ElectionSubcategory::BC => "BC",
            ElectionSubcategory::GC => "GC",
            ElectionSubcategory::ER1 => "ER1",
            ElectionSubcategory::TK => "TK",
            ElectionSubcategory::EK => "EK",
            ElectionSubcategory::EP => "EP",
            ElectionSubcategory::NR => "NR",
            ElectionSubcategory::PR => "PR",
            ElectionSubcategory::LR => "LR",
            ElectionSubcategory::IR => "IR",
        }
    }
}

/// Error returned when an unknown election subcategory string is encountered.
#[derive(Debug, Clone, Error)]
#[error("Unknown election subcategory: {0}")]
pub struct UnknownElectionSubcategory(String);

impl StringValueData for ElectionSubcategory {
    type Error = UnknownElectionSubcategory;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Self::from_str_value(s).ok_or(UnknownElectionSubcategory(s.to_string()))
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
            ElectionCategory::from_str_value("EK"),
            Some(ElectionCategory::EK)
        );
        assert_eq!(
            ElectionCategory::from_str_value("TK"),
            Some(ElectionCategory::TK)
        );
        assert_eq!(ElectionCategory::from_str_value("UNKNOWN"), None);
    }

    #[test]
    fn test_election_category_to_str() {
        assert_eq!(ElectionCategory::EK.to_str_value(), "EK");
        assert_eq!(ElectionCategory::TK.to_str_value(), "TK");
    }
}
