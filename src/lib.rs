//! EML (Election Markup Language) library written by the
//! [Kiesraad](https://www.kiesraad.nl/) (the Dutch Electoral Council) for
//! parsing and writing EML_NL documents written in safe Rust code only.
//!
//! This library sometimes uses EML and EML_NL interchangeably, but only EML_NL
//! is supported. For details of the EML_NL standard, see the
//! [Kiesraad EML_NL repository](https://github.com/kiesraad/EML_NL/).
//!
//! The main entrypoints for this crate are the [`EML`](crate::documents::EML)
//! enum for parsing any EML document. You can also use the specific structs
//! for specific EML_NL documents, such as
//! [`ElectionDefinition`](crate::documents::election_definition::ElectionDefinition)
//! for a 110a EML document. The best reference for which documents are supported
//! are the variants in the [`EML`](crate::documents::EML) enum.
//!
//! Reading of EML documents is done through the [`EMLRead`](crate::io::EMLRead)
//! trait, while writing is done through the [`EMLWrite`](crate::io::EMLWrite)
//! trait.
//!
//! This crate only parses and writes EML documents in memory, it does not
//! support streaming parsing or writing. This was a design decision to keep
//! the code simple and maintainable, and it is expected that EML documents will
//! generally not be extremely large. Up to a few megabytes were expected, but
//! larger documents will work fine as long as enough memory is available.
//! Expect somewhere between 1.2 and 2.0 times the original document size
//! depending on the contents of the file.

// This crate must only use safe Rust code.
#![forbid(unsafe_code)]
// All public items must have some kinds of documentation.
#![forbid(missing_docs)]

pub mod common;
pub mod documents;
mod error;
pub mod io;
pub mod utils;

pub use error::*;

/// Supported EML schema version
pub(crate) const EML_SCHEMA_VERSION: &str = "5";

/// Namespace URI for the EML standard
pub(crate) const NS_EML: &str = "urn:oasis:names:tc:evs:schema:eml";

/// Namespace URI for the Kiesraad expansions on the EML standard
pub(crate) const NS_KR: &str = "http://www.kiesraad.nl/extensions";

/// Namespace URI for the eXtensible Address Language (xAL)
pub(crate) const NS_XAL: &str = "urn:oasis:names:tc:ciq:xsdschema:xAL:2.0";

/// Namespace URI for the eXtensible Name Language (xNL)
pub(crate) const NS_XNL: &str = "urn:oasis:names:tc:ciq:xsdschema:xNL:2.0";

// /// Namespace URI for XML Digital Signatures
// pub(crate) const NS_DS: &str = "http://www.w3.org/2000/09/xmldsig#";
// /// Namespace URI for XML Schema
// pub(crate) const NS_XMLNS: &str = "http://www.w3.org/2000/xmlns/";
// /// Namespace URI for XML
// pub(crate) const NS_XML: &str = "http://www.w3.org/XML/1998/namespace";
