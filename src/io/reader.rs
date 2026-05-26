use crate::{
    MultipleEMLErrors,
    error::{EMLError, EMLErrorKind},
};

/// Reading EML documents from a string slice.
pub trait EMLRead {
    /// Parse an EML document from the given string slice.
    fn parse_eml(input: &str) -> EMLReadResult<Self>
    where
        Self: Sized;
}

/// The result of reading an EML document, which may include non-fatal errors.
#[must_use]
pub enum EMLReadResult<T> {
    /// The document was parsed successfully, with optional non-fatal errors.
    Ok(T, Vec<EMLError>),
    /// The document could not be parsed due to fatal errors.
    Err(EMLError),
}

impl<T> EMLReadResult<T> {
    /// Returns the list of errors (fatal and non-fatal)
    pub fn errors(&self) -> &[EMLError] {
        match self {
            EMLReadResult::Ok(_, errors) => errors,
            EMLReadResult::Err(EMLError::Multiple(MultipleEMLErrors { errors })) => errors,
            EMLReadResult::Err(err) => std::slice::from_ref(err),
        }
    }

    /// Converts this result into a standard Result, returning the value if
    /// successful, or the error(s) if not.
    pub fn ok(self) -> Result<T, EMLError> {
        self.into()
    }

    /// Converts this result into a standard Result, returning the value and
    /// the list of non-fatal errors if successful, or the error(s) if not.
    pub fn ok_with_errors(self) -> Result<(T, Vec<EMLError>), EMLError> {
        self.into()
    }

    /// Unwraps the value if successful, or panics if not.
    #[track_caller]
    pub fn unwrap(self) -> T {
        self.ok().unwrap()
    }

    /// Unwraps the value if successful, or panics with the given message if not.
    #[track_caller]
    pub fn expect(self, msg: &str) -> T {
        self.ok().expect(msg)
    }
}

impl<T> From<EMLReadResult<T>> for Result<T, EMLError> {
    fn from(value: EMLReadResult<T>) -> Self {
        match value {
            EMLReadResult::Ok(doc, _) => Ok(doc),
            EMLReadResult::Err(e) => Err(e),
        }
    }
}

impl<T> From<EMLReadResult<T>> for Result<(T, Vec<EMLError>), EMLError> {
    fn from(value: EMLReadResult<T>) -> Self {
        match value {
            EMLReadResult::Ok(doc, errors) => Ok((doc, errors)),
            EMLReadResult::Err(e) => Err(e),
        }
    }
}

impl<T> EMLRead for T
where
    T: for<'xml> instant_xml::FromXml<'xml>,
{
    fn parse_eml(input: &str) -> EMLReadResult<Self>
    where
        Self: Sized,
    {
        match instant_xml::from_str(input) {
            Ok(doc) => EMLReadResult::Ok(doc, vec![]),
            Err(e) => EMLReadResult::Err(EMLErrorKind::XmlError(e).without_span()),
        }
    }
}

/// A span in the input data, represented as byte offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Start byte offset of the span (inclusive).
    pub start: u64,
    /// End byte offset of the span (exclusive).
    pub end: u64,
}

impl Span {
    /// Create a new span from the given start and end byte offsets.
    pub fn new(start: u64, end: u64) -> Span {
        Span { start, end }
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} until {}", self.start, self.end)
    }
}
