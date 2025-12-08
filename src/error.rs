use std::sync::Arc;

use crate::reader::{OwnedQualifiedName, Span};

#[derive(thiserror::Error, Debug)]
pub enum EMLErrorKind {
    #[error("XML error: {0}")]
    XmlError(#[from] quick_xml::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Escape error: {0}")]
    EscapeError(#[from] quick_xml::escape::EscapeError),

    #[error("Attribute error: {0}")]
    AttributeError(#[from] quick_xml::events::attributes::AttrError),

    #[error("Encoding error: {0}")]
    EncodingError(#[from] quick_xml::encoding::EncodingError),

    #[error("Unexpected end element")]
    UnexpectedEndElement,

    #[error("Unexpected end of file")]
    UnexpectedEof,

    #[error("Unexpected event encountered")]
    UnexpectedEvent,

    #[error("Missing required element: {0}")]
    MissingElement(OwnedQualifiedName),

    #[error("Missing required attribute: {0}")]
    MissingAttribute(OwnedQualifiedName),

    #[error("Unknown namespace: {0}")]
    UnknownNamespace(String),

    #[error("Root element must be named EML")]
    InvalidRootElement,

    #[error("Schema version '{0}' is not supported, only version '5' is supported")]
    SchemaVersionNotSupported(String),

    #[error("Unknown document type: {0}")]
    UnknownDocumentType(String),

    #[error("Invalid document type: expected {0}, found {1}")]
    InvalidDocumentType(&'static str, String),

    #[error("Invalid value for {0}: {1}")]
    InvalidValue(&'static str, #[source] Arc<dyn std::error::Error>),

    #[error("Attributes cannot have the default namespace")]
    AttributeNamespaceError,

    #[error("Elements cannot be in no namespace when a default namespace is defined")]
    ElementNamespaceError,
}

#[derive(thiserror::Error, Debug)]
#[error("{kind} at span {span:?}")]
pub struct EMLError {
    pub kind: EMLErrorKind,
    pub span: Option<Span>,
}

impl EMLError {
    pub fn invalid_value(field: &'static str, source: impl std::error::Error + 'static) -> Self {
        EMLError {
            kind: EMLErrorKind::InvalidValue(field, Arc::new(source)),
            span: None,
        }
    }
}

pub trait EMLResultExt<T> {
    fn with_span(self, span: Span) -> Result<T, EMLError>;
    fn without_span(self) -> Result<T, EMLError>;
}

impl<T, I> EMLResultExt<T> for Result<T, I>
where
    I: Into<EMLErrorKind>,
{
    fn with_span(self, span: Span) -> Result<T, EMLError> {
        self.map_err(|kind| EMLError {
            kind: kind.into(),
            span: Some(span),
        })
    }

    fn without_span(self) -> Result<T, EMLError> {
        self.map_err(|kind| EMLError {
            kind: kind.into(),
            span: None,
        })
    }
}
