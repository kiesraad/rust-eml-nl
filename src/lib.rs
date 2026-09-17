//! EML (Election Markup Language) library written by the
//! [Kiesraad](https://www.kiesraad.nl/) (the Dutch Electoral Council) for
//! parsing and writing EML_NL documents written in safe Rust code only.
//!
//! This library sometimes uses EML and EML_NL interchangeably, but only EML_NL
//! is supported. For details of the EML_NL standard, see the
//! [Kiesraad EML_NL repository](https://github.com/kiesraad/EML_NL/).
//!
//! This crate only parses and writes EML documents in memory, it does not
//! support streaming parsing or writing. This was a design decision to keep
//! the code simple and maintainable, and it is expected that EML documents will
//! generally not be extremely large. Up to a few megabytes were expected, but
//! larger documents will work fine as long as enough memory is available.
//! Expect somewhere between 1.2 and 2.0 times the original document size
//! depending on the contents of the file.
//!
//! ## Getting started
//! The main entrypoints for this crate are the [`EML`](crate::documents::EML)
//! enum for parsing any EML document. You can also use the specific structs
//! for specific EML_NL documents, such as
//! [`ElectionDefinition`](crate::documents::election_definition::ElectionDefinition)
//! for a 110a EML document. The best reference for which documents are supported
//! are the variants in the [`EML`](crate::documents::EML) enum.
//!
//! Reading of EML documents is done through the [`EMLRead`](crate::io::EMLRead)
//! trait, while writing is done through the [`EMLWrite`](crate::io::EMLWrite)
//! trait. So to read an EML document and directly write it back to XML you
//! could do this:
//!
//! ```rust
//! use eml_nl::{documents::EML, io::{EMLRead, EMLWrite}};
//! let xml = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/test-files/polling_stations/eml110b_polling_stations_construction_output.eml.xml"));
//! let eml_doc = EML::parse_eml(xml, eml_nl::io::EMLParsingMode::Strict).unwrap();
//! let xml_output = eml_doc.write_eml_root_str(true, true).unwrap();
//! assert_eq!(xml_output, xml);
//! ```
//!
//! In this example we also see [`EMLParsingMode`](crate::io::EMLParsingMode)
//! being used. This enum defines how strict parsing of several values and elements
//! should be. Take a look at the documenation for the enum for more information.
//!
//! Many times when parsing specific values from EML documents, such as dates or
//! identifiers, depending on the parsing mode, the values may be stored as the
//! raw string or the parsed value. To handle this, we use the
//! [`StringValue`](crate::utils::StringValue) type, which can contain either
//! the raw string or the parsed value. Take a look at the documentation for
//! that type for more information.
//!
//! In general it should not be necessary to use the string values directly
//! unless you are doing something with values that cannot be parsed or where
//! you don't really care about the actual value. In most cases you should let
//! the library handle the parsing of raw strings.
//!
//! ## Error handling
//! During parsing and writing of EML documents, various errors can occur.
//! These are represented by the [`EMLError`] type, which contains an
//! [`EMLErrorKind`] indicating the actual error type.
//!
//! When parsing, the library will attempt to provide location information
//! whenever possible for errors. This is done through the use of the
//! [`Span`](crate::io::Span) type, which indicates the byte range in the
//! original XML document where the error occurred. Other operations (including
//! writing documents) generally will not have this location information
//! available.

// This crate must only use safe Rust code.
#![forbid(unsafe_code)]
// All public items must have some kinds of documentation.
#![forbid(missing_docs)]

pub mod common;
pub mod csv;
pub mod documents;
mod error;
pub mod io;
pub mod utils;

pub use error::*;

use crate::io::QualifiedName;

/// Supported EML schema version
pub(crate) const OASIS_EML_SCHEMA_VERSION: &str = "5";

/// Namespace URI for the EML standard
pub(crate) const NS_EML: &str = "urn:oasis:names:tc:evs:schema:eml";

/// Namespace URI for the Kiesraad expansions on the EML standard
pub(crate) const NS_KR: &str = "http://www.kiesraad.nl/extensions";

/// Namespace URI for the eXtensible Address Language (xAL)
pub(crate) const NS_XAL: &str = "urn:oasis:names:tc:ciq:xsdschema:xAL:2.0";

/// Namespace URI for the eXtensible Name Language (xNL)
pub(crate) const NS_XNL: &str = "urn:oasis:names:tc:ciq:xsdschema:xNL:2.0";

/// Namespace URI for XML Digital Signatures
pub(crate) const NS_DS: &str = "http://www.w3.org/2000/09/xmldsig#";

/// Describes the EML_NL version of the EML standard.
///
/// Since EML 1.3, EML_NL adds a `kr:Schema` element within the top level
/// element to indicate the specific EML_NL schema version in use. If this
/// element is not present, the legacy unversioned EML_NL schema is assumed,
/// in practice this means that this document should follow the EML_NL 1.2.2
/// schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum EMLVersion {
    /// A legacy unversioned EML_NL document, assumed to follow EML_NL 1.2.2
    V1_2_2,

    /// An EML_NL version 1.3 document
    #[default]
    V1_3,
}

impl EMLVersion {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Schema", Some(NS_KR));

    /// Returns the string representation of this EML version, if one is known.
    ///
    /// Returns `None` for [`EMLVersion::V1_2_2`].
    pub fn to_str(&self) -> Option<&'static str> {
        match self {
            EMLVersion::V1_2_2 => None,
            EMLVersion::V1_3 => Some("1.3"),
        }
    }

    /// Returns `true` if this is an EML_NL version 1.3 document.
    pub fn is_v1_3(&self) -> bool {
        matches!(self, EMLVersion::V1_3)
    }

    /// Returns `true` if this is a legacy unversioned EML_NL document.
    pub fn is_v1_2_2(&self) -> bool {
        matches!(self, EMLVersion::V1_2_2)
    }
}

impl std::str::FromStr for EMLVersion {
    type Err = UnsupportedEMLVersion;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1.3" => Ok(EMLVersion::V1_3),
            _ => Err(UnsupportedEMLVersion(s.to_owned())),
        }
    }
}

/// EML_NL version is not known to this version of the library.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Unsupported EML_NL version: {0}")]
pub struct UnsupportedEMLVersion(String);

impl From<UnsupportedEMLVersion> for EMLError {
    fn from(value: UnsupportedEMLVersion) -> Self {
        EMLErrorKind::UnsupportedEMLVersion(value.0).without_span()
    }
}

// /// Namespace URI for XML Schema
// pub(crate) const NS_XMLNS: &str = "http://www.w3.org/2000/xmlns/";
// /// Namespace URI for XML
// pub(crate) const NS_XML: &str = "http://www.w3.org/XML/1998/namespace";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eml_version_to_str() {
        assert_eq!(EMLVersion::V1_3.to_str(), Some("1.3"));
        assert_eq!(EMLVersion::V1_2_2.to_str(), None);
    }

    #[test]
    fn test_eml_version_from_str() {
        assert_eq!("1.3".parse::<EMLVersion>(), Ok(EMLVersion::V1_3));
        assert!("1.2".parse::<EMLVersion>().is_err());
    }

    #[test]
    fn test_comparison_of_eml_versions() {
        assert!(EMLVersion::V1_3 > EMLVersion::V1_2_2);
        assert!(EMLVersion::V1_3 >= EMLVersion::V1_3);
        assert!(EMLVersion::V1_2_2 <= EMLVersion::V1_3);
        assert!(EMLVersion::V1_2_2 < EMLVersion::V1_3);
        assert!(EMLVersion::V1_3 != EMLVersion::V1_2_2);
        assert!(EMLVersion::V1_3 == EMLVersion::V1_3);
    }
}
