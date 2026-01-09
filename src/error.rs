use crate::{OwnedQualifiedName, Span};

/// Different kinds of errors that can occur during EML-NL processing.
#[derive(thiserror::Error, Debug)]
pub enum EMLErrorKind {
    /// An error originanting from the XML parser
    #[error("XML error: {0}")]
    XmlError(#[from] quick_xml::Error),

    /// An input/output error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// An error during escaping/unescaping XML content
    #[error("Escape error: {0}")]
    EscapeError(#[from] quick_xml::escape::EscapeError),

    /// An error related to parsing XML attributes
    #[error("Attribute error: {0}")]
    AttributeError(#[from] quick_xml::events::attributes::AttrError),

    /// An error related to XML encoding/decoding
    #[error("Encoding error: {0}")]
    EncodingError(#[from] quick_xml::encoding::EncodingError),

    /// An error converting from UTF-8
    #[error("UTF-8 conversion error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    /// An end element was found, but it was not the expected one
    #[error("Unexpected end element")]
    UnexpectedEndElement,

    /// The end of the file was reached unexpectedly
    #[error("Unexpected end of file")]
    UnexpectedEof,

    /// An unexpected parsing event was encountered during XML parsing
    #[error("Unexpected parsing event encountered")]
    UnexpectedEvent,

    /// A required element was missing
    #[error("Missing required element: {0}")]
    MissingElement(OwnedQualifiedName),

    /// A required attribute was missing
    #[error("Missing required attribute: {0}")]
    MissingAttribute(OwnedQualifiedName),

    /// A namespace was encountered that is not recognized
    #[error("Unknown namespace: {0}")]
    UnknownNamespace(String),

    /// The root element was not named "EML"
    #[error("Root element must be named EML")]
    InvalidRootElement,

    /// The EML schema version is not supported
    #[error("Schema version '{0}' is not supported, only version '5' is supported")]
    SchemaVersionNotSupported(String),

    /// The document type is not recognized
    #[error("Unknown document type: {0}")]
    UnknownDocumentType(String),

    /// The document type is invalid, a specific type was expected
    #[error("Invalid document type: expected {0}, found {1}")]
    InvalidDocumentType(&'static str, String),

    /// An invalid value was encountered for a specific attribute/element
    #[error("Invalid value for {0}: {1}")]
    InvalidValue(
        &'static str,
        #[source] Box<dyn std::error::Error + Send + Sync>,
    ),

    /// Attributes cannot have the default namespace
    #[error("Attributes cannot have the default namespace")]
    AttributeNamespaceError,

    /// Elements cannot be in no namespace when a default namespace is defined
    #[error("Elements cannot be in no namespace when a default namespace is defined")]
    ElementNamespaceError,
}

/// An error encountered during EML-NL processing.
///
/// The error includes the kind of error as well as an optional span indicating
/// where in the source XML the error approximately occured.
#[derive(thiserror::Error, Debug)]
#[error("{kind} at span {span:?}")]
pub struct EMLError {
    /// The error that occured
    pub kind: EMLErrorKind,
    /// Location in the source XML where the error occured
    pub span: Option<Span>,
}

impl EMLError {
    /// Create a new invalid value error
    pub(crate) fn invalid_value(
        field: &'static str,
        source: impl std::error::Error + Send + Sync + 'static,
        span: Option<Span>,
    ) -> Self {
        EMLError {
            kind: EMLErrorKind::InvalidValue(field, Box::new(source)),
            span,
        }
    }
}

/// Extension trait for Result to add context to EMLError
pub(crate) trait EMLResultExt<T> {
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
