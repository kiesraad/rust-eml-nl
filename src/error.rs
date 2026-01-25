use crate::io::{OwnedQualifiedName, Span};

/// Different kinds of errors that can occur during EML_NL processing.
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

    /// An unexpected element was found
    #[error("Unexpected element: {0} inside of {1}")]
    UnexpectedElement(OwnedQualifiedName, OwnedQualifiedName),

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
        OwnedQualifiedName,
        #[source] Box<dyn std::error::Error + Send + Sync + 'static>,
    ),

    /// Attributes cannot have the default namespace
    #[error("Attributes cannot have the default namespace")]
    AttributeNamespaceError,

    /// Elements cannot be in no namespace when a default namespace is defined
    #[error("Elements cannot be in no namespace when a default namespace is defined")]
    ElementNamespaceError,

    /// The ContestIdentifier element is missing
    #[error("Missing the ContestIdentifier element")]
    MissingContenstIdentifier,

    /// The ElectionDate element is used without using the kiesraad namespace
    #[error("Used ElectionDate element without using the kiesraad namespace")]
    InvalidElectionDateNamespace,
}

impl EMLErrorKind {
    /// Adds span information to the error.
    pub(crate) fn add_span(self, span: Span) -> EMLError {
        EMLError::Positioned { kind: self, span }
    }

    /// Converts the error kind to an error without span information.
    #[expect(unused)]
    pub(crate) fn without_span(self) -> EMLError {
        EMLError::UnknownPosition { kind: self }
    }
}

/// An error encountered during EML_NL processing.
///
/// The error includes the kind of error as well as an optional span indicating
/// where in the source XML the error approximately occured.
#[derive(thiserror::Error, Debug)]
pub enum EMLError {
    /// An error with position information in a document
    #[error("Error in EML: {kind} at position {span:?}")]
    Positioned {
        /// The kind of error that occured
        kind: EMLErrorKind,
        /// The span (position) in the document where the error occured
        span: Span,
    },
    /// An error without position information
    #[error("Error in EML: {kind}")]
    UnknownPosition {
        /// The kind of error that occured
        kind: EMLErrorKind,
    },
    /// A list of multiple errors
    #[error("Multiple errors in EML: {0}")]
    Multiple(MultipleEMLErrors),
}

/// An error containing multiple EMLErrors
#[derive(Debug)]
pub struct MultipleEMLErrors {
    /// The list of errors
    pub errors: Vec<EMLError>,
}

impl std::fmt::Display for MultipleEMLErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} non-fatal error(s) and then: {}",
            self.errors.len() - 1,
            self.errors.last().unwrap()
        )
    }
}

impl EMLError {
    /// Create a new invalid value error
    pub(crate) fn invalid_value(
        field: OwnedQualifiedName,
        source: impl std::error::Error + Send + Sync + 'static,
        span: Option<Span>,
    ) -> Self {
        let kind = EMLErrorKind::InvalidValue(field, Box::new(source));
        if let Some(span) = span {
            EMLError::Positioned { kind, span }
        } else {
            EMLError::UnknownPosition { kind }
        }
    }

    /// Create an EMLError from a vector of errors.
    pub(crate) fn from_vec_with_additional(mut errors: Vec<EMLError>, error: EMLError) -> Self {
        errors.push(error);
        if errors.len() == 1 {
            errors
                .into_iter()
                .next()
                .expect("Vec must have one element")
        } else {
            EMLError::Multiple(MultipleEMLErrors { errors })
        }
    }

    /// Returns the kind of this error.
    ///
    /// When this error consists of multiple errors, None is returned.
    ///
    /// Note: when multiple errors are present, the kind of the last error is returned.
    pub fn kind(&self) -> &EMLErrorKind {
        match self {
            EMLError::Positioned { kind, .. } => kind,
            EMLError::UnknownPosition { kind } => kind,
            EMLError::Multiple(MultipleEMLErrors { errors }) => errors
                .last()
                .map(|e| e.kind())
                .expect("Errors vec cannot be empty"),
        }
    }

    /// Returns the span of this error, if available.
    ///
    /// Note: when multiple errors are present, the span of the last error is returned.
    pub fn span(&self) -> Option<Span> {
        match self {
            EMLError::Positioned { span, .. } => Some(*span),
            EMLError::UnknownPosition { .. } => None,
            EMLError::Multiple(MultipleEMLErrors { errors }) => {
                errors.last().and_then(|e| e.span())
            }
        }
    }

    /// Returns whether this error is considered fatal.
    ///
    /// Note: when multiple errors are present, this only checks the last error.
    pub fn is_fatal(&self) -> bool {
        !matches!(
            self.kind(),
            EMLErrorKind::UnexpectedElement(_, _) | EMLErrorKind::InvalidValue(_, _)
        )
    }
}

/// Extension trait for Result to add context to EMLError
pub(crate) trait EMLResultExt<T> {
    /// Adds span information to the error if it occurs.
    fn with_span(self, span: Span) -> Result<T, EMLError>;
    /// Converts the error kind to an error without span information.
    fn without_span(self) -> Result<T, EMLError>;
}

impl<T, I> EMLResultExt<T> for Result<T, I>
where
    I: Into<EMLErrorKind>,
{
    fn with_span(self, span: Span) -> Result<T, EMLError> {
        self.map_err(|kind| EMLError::Positioned {
            kind: kind.into(),
            span,
        })
    }

    fn without_span(self) -> Result<T, EMLError> {
        self.map_err(|kind| EMLError::UnknownPosition { kind: kind.into() })
    }
}
