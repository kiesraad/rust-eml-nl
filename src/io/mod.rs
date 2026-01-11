//! Reading and writing EML_NL documents.

mod qualified_name;
mod reader;
mod writer;

pub use qualified_name::*;
pub use reader::*;
pub use writer::*;

use crate::EMLError;

pub(crate) trait EMLElement {
    const EML_NAME: QualifiedName<'static, 'static>;

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError>
    where
        Self: Sized;

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError>;
}

impl<T> EMLWriteElement for T
where
    T: EMLElement,
{
    fn write_eml_element(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        self.write_eml(writer)
    }
}

impl<T> EMLReadElement for T
where
    T: EMLElement,
{
    fn read_eml_element(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        T::read_eml(elem)
    }
}
