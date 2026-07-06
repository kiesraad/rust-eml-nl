use thiserror::Error;

use crate::{EMLError, EMLValueResultExt as _, utils::StringValueData};

/// Voting channel used in the election.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VotingChannelType {
    /// A physical polling station
    Polling,
    /// Votes by mail
    Postal,
}

impl VotingChannelType {
    /// Create a new VotingChannelType from a string, validating its format.
    pub fn new(s: impl AsRef<str>) -> Result<Self, EMLError> {
        Self::from_eml_value(s).wrap_value_error()
    }

    /// Create a [`VotingChannelType`] from a `&str`, if possible.
    pub fn from_eml_value(s: impl AsRef<str>) -> Result<Self, UnknownVotingChannelError> {
        let data = s.as_ref();
        match data {
            "polling" => Ok(VotingChannelType::Polling),
            "postal" => Ok(VotingChannelType::Postal),
            _ => Err(UnknownVotingChannelError(data.to_string())),
        }
    }

    /// Get the `&str` representation of this [`VotingChannelType`].
    pub fn to_eml_value(&self) -> &'static str {
        match self {
            VotingChannelType::Polling => "polling",
            VotingChannelType::Postal => "postal",
        }
    }
}

/// Error returned when an unknown voting channel string is encountered.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("Unknown voting channel: {0}")]
pub struct UnknownVotingChannelError(String);

impl StringValueData for VotingChannelType {
    type Error = UnknownVotingChannelError;

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
    fn test_valid_voting_channel_types() {
        let valid_channels = ["polling", "postal"];
        for channel in valid_channels {
            assert!(
                VotingChannelType::from_eml_value(channel).is_ok(),
                "VotingChannelType should accept valid channel: {}",
                channel
            );
        }
    }

    #[test]
    fn test_invalid_voting_channel_types() {
        let invalid_channels = ["", "test", "abc"];
        for channel in invalid_channels {
            assert!(
                VotingChannelType::from_eml_value(channel).is_err(),
                "VotingChannelType should reject invalid channel: {}",
                channel
            );
        }
    }
}
