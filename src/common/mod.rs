//! Element definitions common to multiple EML_NL document variants.

mod managing_authority;

pub use managing_authority::*;
use thiserror::Error;

use std::borrow::Cow;

use crate::{
    NS_EML, NS_KR,
    error::EMLError,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName},
    utils::{StringValue, StringValueData, XsDateOrDateTime, XsDateTime},
};

/// Document transaction id.
///
/// EML_NL documents contain a transaction id, but this is generally not used
/// and set to `1` as a default.
#[derive(Debug, Clone)]
pub struct TransactionId(pub StringValue<u64>);

impl TransactionId {
    /// Get the raw string value of the transaction id.
    pub fn raw(&self) -> Cow<'_, str> {
        self.0.raw()
    }

    /// Get the parsed u64 value of the transaction id.
    pub fn value(&self) -> Result<u64, EMLError> {
        Ok(self
            .0
            .value_err(("TransactionId", NS_EML), None)?
            .into_owned())
    }
}

impl EMLElement for TransactionId {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("TransactionId", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let text = elem.text_without_children()?;

        Ok(TransactionId(StringValue::from_maybe_parsed_err(
            text,
            elem.strict_value_parsing(),
            ("TransactionId", NS_EML),
            Some(elem.inner_span()),
        )?))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer.text(self.raw().as_ref())?.finish()?;
        Ok(())
    }
}

/// Document creation date time.
#[derive(Debug, Clone)]
pub struct CreationDateTime(pub StringValue<XsDateTime>);

impl CreationDateTime {
    /// Get the raw string value of the creation date time.
    pub fn raw(&self) -> Cow<'_, str> {
        self.0.raw()
    }

    /// Get the parsed XsDateTime value of the creation date time.
    pub fn value(&self) -> Result<XsDateTime, EMLError> {
        Ok(self
            .0
            .value_err(("CreationDateTime", NS_KR), None)?
            .into_owned())
    }
}

impl EMLElement for CreationDateTime {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("CreationDateTime", Some(NS_KR));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let text = elem.text_without_children()?;

        Ok(CreationDateTime(StringValue::from_maybe_parsed_err(
            text,
            elem.strict_value_parsing(),
            ("CreationDateTime", NS_KR),
            Some(elem.inner_span()),
        )?))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer.text(self.raw().as_ref())?.finish()?;
        Ok(())
    }
}

/// Document issue date.
///
/// Can be either a date or a date with time.
#[derive(Debug, Clone)]
pub struct IssueDate(pub StringValue<XsDateOrDateTime>);

impl IssueDate {
    /// Get the raw string value of the issue date.
    pub fn raw(&self) -> Cow<'_, str> {
        self.0.raw()
    }

    /// Get the parsed XsDateOrDateTime value of the issue date.
    pub fn value(&self) -> Result<XsDateOrDateTime, EMLError> {
        Ok(self.0.value_err(IssueDate::EML_NAME, None)?.into_owned())
    }
}

impl EMLElement for IssueDate {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("IssueDate", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let text = elem.text_without_children()?;

        Ok(IssueDate(StringValue::from_maybe_parsed_err(
            text,
            elem.strict_value_parsing(),
            IssueDate::EML_NAME,
            Some(elem.inner_span()),
        )?))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer.text(self.raw().as_ref())?.finish()?;
        Ok(())
    }
}

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
