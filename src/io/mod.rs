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
}

impl<T> EMLReadElement for T
where
    T: EMLElement,
{
    fn read_eml_element(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        T::read_eml(elem)
    }
}

#[cfg(test)]
pub(crate) fn test_xml_fragment(input: &str) -> String {
    input
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect()
}
