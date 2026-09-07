//! Reading and writing EML_NL documents.

mod qualified_name;
mod reader;
mod writer;

pub use qualified_name::*;
pub use reader::*;
pub use writer::*;

use crate::{EMLError, EMLVersion};

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

/// Represents a full EML document.
pub trait EMLDocument {
    /// Get the EML_NL document version for the document.
    fn document_version(&self) -> EMLVersion;
}

#[cfg(test)]
pub(crate) fn test_xml_fragment(input: &str) -> String {
    let mut data_lines = input
        .split('\n')
        .skip_while(|l| l.trim().is_empty())
        .peekable();
    let indent = data_lines
        .peek()
        .map(|v| v.chars().take_while(|c| c.is_ascii_whitespace()).count())
        .unwrap_or(0);
    let mut result = data_lines
        .map(|line| {
            let cut = line
                .char_indices()
                .take_while(|&(_, c)| c.is_ascii_whitespace())
                .take(indent)
                .last()
                .map(|(i, c)| i + c.len_utf8())
                .unwrap_or(0);

            let mut line = line[cut..].to_owned();
            line.push('\n');
            line
        })
        .collect::<String>();
    result.truncate(result.trim_end().len());
    result.push('\n');

    result
}
