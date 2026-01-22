//! Element definitions common to multiple EML_NL document variants.

mod canonicalization_method;
mod contest_identifier;
mod election_domain;
mod election_tree;
mod locality_name;
mod managing_authority;
mod postal_code;
mod reporting_unit_identifier;

pub use canonicalization_method::*;
pub use contest_identifier::*;
pub use election_domain::*;
pub use election_tree::*;
pub use locality_name::*;
pub use managing_authority::*;
pub use postal_code::*;
pub use reporting_unit_identifier::*;

use std::borrow::Cow;

use crate::{
    NS_EML, NS_KR,
    error::EMLError,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName},
    utils::{StringValue, XsDateOrDateTime, XsDateTime},
};

/// Document transaction id.
///
/// EML_NL documents contain a transaction id, but this is generally not used
/// and set to `1` as a default.
#[derive(Debug, Clone)]
pub struct TransactionId(pub StringValue<u64>);

impl TransactionId {
    /// Create a new TransactionId from a u64 value.
    pub fn new(id: u64) -> Self {
        TransactionId(StringValue::from_value(id))
    }

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
        Ok(TransactionId(elem.string_value()?))
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
    /// Create a new CreationDateTime from a XsDateTime value.
    pub fn new(dt: XsDateTime) -> Self {
        CreationDateTime(StringValue::from_value(dt))
    }

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
        Ok(CreationDateTime(elem.string_value()?))
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
    /// Create a new IssueDate from a `XsDateOrDateTime` value.
    pub fn new(dt: XsDateOrDateTime) -> Self {
        IssueDate(StringValue::from_value(dt))
    }

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
        Ok(IssueDate(elem.string_value()?))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer.text(self.raw().as_ref())?.finish()?;
        Ok(())
    }
}
