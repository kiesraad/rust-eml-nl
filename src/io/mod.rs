//! Reading and writing EML_NL documents.

mod qualified_name;
mod reader;
mod writer;

pub use qualified_name::*;
pub use reader::*;
pub use writer::*;

#[cfg(test)]
pub(crate) fn test_xml_fragment(input: &str) -> String {
    input
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect()
}
