use crate::{EMLError, EMLResultExt};

/// Writing EML documents to a [`String`] or [`Vec<u8>`].
///
/// The errors generated during writing do not contain location information, as
/// there is no document to refer to yet. Most of the time errors generated
/// during writing are underlying errors or logic errors in your implementation,
/// so location information would be of limited use anyway.
pub trait EMLWrite {
    /// Writes an EML document with an EML root element to a byte vector.
    fn write_eml_root(&self, include_declaration: bool) -> Result<Vec<u8>, EMLError>;

    /// Writes an EML document with an EML root element to a string.
    fn write_eml_root_str(&self, include_declaration: bool) -> Result<String, EMLError>;
}

impl<T> EMLWrite for T
where
    T: instant_xml::ToXml,
{
    fn write_eml_root(&self, include_declaration: bool) -> Result<Vec<u8>, EMLError> {
        Ok(self.write_eml_root_str(include_declaration)?.into_bytes())
    }

    fn write_eml_root_str(&self, include_declaration: bool) -> Result<String, EMLError> {
        let mut xml = String::new();
        if include_declaration {
            xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
        }
        instant_xml::to_writer(self, &mut xml).without_span()?;
        Ok(xml)
    }
}
