use std::borrow::Cow;

use crate::{
    EMLError, NS_KR,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName},
    utils::{StringValue, XsDateTime},
};

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
