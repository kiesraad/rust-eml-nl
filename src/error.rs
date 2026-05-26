use crate::io::{OwnedQualifiedName, Span};

/// Different kinds of errors that can occur during EML_NL processing.
#[derive(thiserror::Error, Debug)]
pub enum EMLErrorKind {
    /// An input/output error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// An error from instant-xml serialization/deserialization
    #[error("instant-xml error: {0}")]
    XmlError(#[from] instant_xml::Error),

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

    /// An element existed, but it was empty or had no text content
    #[error("Missing value for element: {0}")]
    MissingElementValue(OwnedQualifiedName),

    /// None of the choice elements were found
    #[error("Missing any of these elements: {0:?}")]
    MissingChoiceElements(Vec<OwnedQualifiedName>),

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

    /// An error occurred while converting a value to the parsed type
    #[error("Error converting value: {0}")]
    ValueConversionError(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

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

    /// The RejectedVotes element with ReasonCode "blanco" is missing
    #[error("The RejectedVotes element with ReasonCode 'blanco' is missing")]
    MissingRejectedVotesBlank,

    /// The RejectedVotes element with ReasonCode "ongeldig" is missing
    #[error("The RejectedVotes element with ReasonCode 'ongeldig' is missing")]
    MissingRejectedVotesInvalid,

    /// A Selection is missing a selection type (i.e. Candidate, AffiliationIdentifier or ReferendumOptionIdentifier)
    #[error(
        "A Selection is missing a selection type (i.e. Candidate, AffiliationIdentifier or ReferendumOptionIdentifier)"
    )]
    MissingSelectionType,

    /// A field that is required for building a struct is missing.
    #[error("A required property '{0}' is missing for building this struct")]
    MissingBuildProperty(&'static str),

    /// The NominationDate is before the ElectionDate, which is not allowed.
    #[error("The NominationDate is before the ElectionDate, which is not allowed")]
    NominationDateNotBeforeElectionDate,

    /// The ElectionSubcategory is not valid for the ElectionCategory.
    #[error("The ElectionSubcategory is not valid for the ElectionCategory")]
    InvalidElectionSubcategory,

    /// The voting method specified in the document is not supported.
    #[error("The voting method specified in the document is not supported, only SPV is supported")]
    UnsupportedVotingMethod,

    /// The preference threshold specified in the document is not valid.
    #[error("The preference threshold specified does not match the election identifier")]
    InvalidPreferenceThreshold,

    /// The number of seats specified in the document is not valid.
    #[error("The number of seats specified does not match the subcategory")]
    InvalidNumberOfSeats,

    /// A referendum option was found where none was expected.
    #[error("A referendum option was found where none was expected")]
    UnexpectedReferendumOptionSelection,

    /// A candidate was found without an affiliation, which is not allowed.
    #[error("A candidate without affiliation was found")]
    CandidateWithoutAffiliationFound,

    /// A custom error with something that can be displayed
    #[error("Custom error: {0}")]
    Custom(Box<dyn CustomError>),
}

/// Custom error type that can be used in EMLErrorKind::Custom
pub trait CustomError: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static {}

impl<T> CustomError for T where T: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static {}

impl EMLErrorKind {
    /// Converts the error kind to an error without span information.
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

    /// Create a new custom error.
    pub fn custom(source: impl CustomError) -> Self {
        EMLErrorKind::Custom(Box::new(source)).without_span()
    }

    /// Create an EMLError from a list of errors.
    pub(crate) fn from_vec(errors: Vec<EMLError>) -> Self {
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

    /// Returns the kind of this error as an EMLErrorKind, consuming the error.
    ///
    /// When this error consists of multiple errors, the kind of the last error is returned.
    pub fn into_kind(self) -> EMLErrorKind {
        match self {
            EMLError::Positioned { kind, .. } => kind,
            EMLError::UnknownPosition { kind } => kind,
            EMLError::Multiple(MultipleEMLErrors { errors }) => errors
                .into_iter()
                .last()
                .map(|e| e.into_kind())
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
}

/// Extension trait for Result to add context to EMLError
pub(crate) trait EMLResultExt<T> {
    /// Converts the error kind to an error without span information.
    fn without_span(self) -> Result<T, EMLError>;
}

impl<T, I> EMLResultExt<T> for Result<T, I>
where
    I: Into<EMLErrorKind>,
{
    fn without_span(self) -> Result<T, EMLError> {
        self.map_err(|kind| EMLError::UnknownPosition { kind: kind.into() })
    }
}

/// Extension trait for Result to add context to EMLError for errors that can be
/// converted into EMLErrorKind::InvalidValue.
pub(crate) trait EMLValueResultExt<T> {
    /// Convert the error into an EMLError with context about the field that caused the error.
    fn wrap_field_value_error(
        self,
        element_name: impl Into<OwnedQualifiedName>,
    ) -> Result<T, EMLError>;

    /// Convert the error into an EMLError that the value is invalid.
    fn wrap_value_error(self) -> Result<T, EMLError>;
}

impl<T, I> EMLValueResultExt<T> for Result<T, I>
where
    I: std::error::Error + Send + Sync + 'static,
{
    fn wrap_field_value_error(
        self,
        element_name: impl Into<OwnedQualifiedName>,
    ) -> Result<T, EMLError> {
        self.map_err(|e| EMLError::invalid_value(element_name.into(), Box::new(e), None))
    }

    fn wrap_value_error(self) -> Result<T, EMLError> {
        self.map_err(|e| EMLErrorKind::ValueConversionError(Box::new(e)).without_span())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NS_EML;

    #[test]
    fn test_creating_invalid_value_error() {
        let error = EMLError::invalid_value(
            OwnedQualifiedName::from_static("Test", Some(NS_EML)),
            std::io::Error::other("error"),
            None,
        );

        assert!(matches!(
            error,
            EMLError::UnknownPosition {
                kind: EMLErrorKind::InvalidValue(_, _)
            }
        ));

        let error_with_span = EMLError::invalid_value(
            OwnedQualifiedName::from_static("Test", Some(NS_EML)),
            std::io::Error::other("error"),
            Some(Span { start: 10, end: 20 }),
        );

        assert!(matches!(
            error_with_span,
            EMLError::Positioned {
                kind: EMLErrorKind::InvalidValue(_, _),
                span: Span { start: 10, end: 20 }
            }
        ));
    }

    #[test]
    fn test_creating_multiple_errors() {
        let err1 = EMLError::Positioned {
            kind: EMLErrorKind::UnexpectedEndElement,
            span: Span { start: 0, end: 5 },
        };
        let err2 = EMLError::Positioned {
            kind: EMLErrorKind::MissingElement(OwnedQualifiedName::from_static(
                "Test",
                Some(NS_EML),
            )),
            span: Span { start: 10, end: 15 },
        };

        let multiple_error = EMLError::from_vec(vec![err1, err2]);
        assert!(matches!(multiple_error, EMLError::Multiple(_)));

        let err3 = EMLError::Positioned {
            kind: EMLErrorKind::UnexpectedEof,
            span: Span { start: 0, end: 10 },
        };
        let single_error = EMLError::from_vec(vec![err3]);
        assert!(matches!(
            single_error,
            EMLError::Positioned {
                kind: EMLErrorKind::UnexpectedEof,
                span: Span { start: 0, end: 10 }
            }
        ));
    }

    #[test]
    fn get_data_from_error() {
        let err = EMLError::Positioned {
            kind: EMLErrorKind::UnexpectedEof,
            span: Span { start: 0, end: 10 },
        };
        assert!(matches!(err.kind(), &EMLErrorKind::UnexpectedEof));
        assert_eq!(err.span(), Some(Span { start: 0, end: 10 }));

        let err2 = EMLError::UnknownPosition {
            kind: EMLErrorKind::UnexpectedElement(
                OwnedQualifiedName::from_static("Test", None),
                OwnedQualifiedName::from_static("Test", None),
            ),
        };

        assert!(matches!(
            err2.kind(),
            &EMLErrorKind::UnexpectedElement(_, _)
        ));
        assert_eq!(err2.span(), None);

        let err3 = EMLError::Multiple(MultipleEMLErrors {
            errors: vec![
                EMLError::Positioned {
                    kind: EMLErrorKind::UnexpectedElement(
                        OwnedQualifiedName::from_static("Test", None),
                        OwnedQualifiedName::from_static("Test", None),
                    ),
                    span: Span { start: 0, end: 5 },
                },
                EMLError::UnknownPosition {
                    kind: EMLErrorKind::MissingElement(OwnedQualifiedName::from_static(
                        "Test", None,
                    )),
                },
            ],
        });
        assert!(matches!(err3.kind(), &EMLErrorKind::MissingElement(_)));
        assert_eq!(err3.span(), None);
    }
}
