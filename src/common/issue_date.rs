use std::borrow::Cow;

use crate::{
    NS_EML,
    error::EMLError,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName},
    utils::{StringValue, XsDateOrDateTime},
};

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
