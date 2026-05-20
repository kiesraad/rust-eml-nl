use crate::{
    MultipleEMLErrors,
    error::{EMLError, EMLErrorKind},
};

/// Reading EML documents from a string slice.
pub trait EMLRead {
    /// Parse an EML document from the given string slice.
    ///
    /// The `parsing_mode` parameter indicates whether strict parsing of values
    /// (e.g. dates, numbers) should be performed. If set to Strict, any parsing
    /// error will fail immediately. If set to StrictFallback, parsing errors
    /// will be collected and the raw string value will be used instead. If set
    /// to Loose, no parsing will be performed and all values will be stored as
    /// raw strings.
    fn parse_eml(input: &str, parsing_mode: EMLParsingMode) -> EMLReadResult<Self>
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
    fn parse_eml(input: &str, _parsing_mode: EMLParsingMode) -> EMLReadResult<Self>
    where
        Self: Sized,
    {
        match instant_xml::from_str(input) {
            Ok(doc) => EMLReadResult::Ok(doc, vec![]),
            Err(e) => EMLReadResult::Err(EMLErrorKind::InstantXmlError(e).without_span()),
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

/// The mode to use when parsing values in EML files.
///
/// This enum defines how strict the library handles parsing of several values
/// and known issues in EML files. In strict mode any issue will immediately
/// cause a parsing error and parsing will fail right away. With fallback,
/// whenever we encounter an issue that is recoverable we continue parsing.
/// With loose mode many parsing operations aren't even attempted and many
/// values are just stored as raw strings. Also take a look at the documentation
/// for [`StringValue`] for more information on how these string/parsed values
/// are handled in the different modes and how to use them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EMLParsingMode {
    /// Require strict parsing of all stringly values to their respective types
    Strict,

    /// Try to parse stringly values, but fall back to raw strings on failure.
    ///
    /// This mode will collect errors to allow reporting them later.
    StrictFallback,

    /// Do not attempt to parse stringly values, always store raw strings.
    Loose,
}

impl EMLParsingMode {
    /// Returns whether the parsing mode is `Strict`.
    pub fn is_strict(&self) -> bool {
        matches!(self, EMLParsingMode::Strict)
    }
}
