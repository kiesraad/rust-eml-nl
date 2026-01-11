//! Element definitions common to multiple EML_NL document variants.

use std::borrow::Cow;

use crate::{
    error::EMLError,
    io::{EMLElement, EMLElementWriter, EMLReadElement, EMLWriteElement},
    utils::{StringValue, XsDateOrDateTime, XsDateTime},
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
        Ok(self.0.value_err("TransactionId", None)?.into_owned())
    }
}

impl EMLReadElement for TransactionId {
    fn read_eml_element(elem: &mut EMLElement<'_, '_>) -> Result<Self, EMLError> {
        let text = elem.text_without_children()?;

        Ok(TransactionId(StringValue::from_maybe_parsed_err(
            text,
            elem.strict_value_parsing(),
            "TransactionId",
            Some(elem.inner_span()),
        )?))
    }
}

impl EMLWriteElement for TransactionId {
    fn write_eml_element(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
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
        Ok(self.0.value_err("CreationDateTime", None)?.into_owned())
    }
}

impl EMLReadElement for CreationDateTime {
    fn read_eml_element(elem: &mut EMLElement<'_, '_>) -> Result<Self, EMLError> {
        let text = elem.text_without_children()?;

        Ok(CreationDateTime(StringValue::from_maybe_parsed_err(
            text,
            elem.strict_value_parsing(),
            "CreationDateTime",
            Some(elem.inner_span()),
        )?))
    }
}

impl EMLWriteElement for CreationDateTime {
    fn write_eml_element(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
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
        Ok(self.0.value_err("IssueDate", None)?.into_owned())
    }
}

impl EMLReadElement for IssueDate {
    fn read_eml_element(elem: &mut EMLElement<'_, '_>) -> Result<Self, EMLError> {
        let text = elem.text_without_children()?;

        Ok(IssueDate(StringValue::from_maybe_parsed_err(
            text,
            elem.strict_value_parsing(),
            "IssueDate",
            Some(elem.inner_span()),
        )?))
    }
}

impl EMLWriteElement for IssueDate {
    fn write_eml_element(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer.text(self.raw().as_ref())?.finish()?;
        Ok(())
    }
}
