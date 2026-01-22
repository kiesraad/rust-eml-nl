use std::{borrow::Cow, convert::Infallible, num::NonZeroU64};

use crate::{
    EMLError,
    io::{QualifiedName, Span},
};

/// Trait for data types that can be used with [`StringValue`], defines how to parse and serialize the value.
pub trait StringValueData: Clone {
    /// The error type returned when parsing the value from a string fails.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Parse the value from a string.
    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized;

    /// Convert the value to its raw string representation.
    fn to_raw_value(&self) -> String;
}

/// A string value that can either be stored as a raw unparsed string or as a parsed value of type `T`.
///
/// The type `T` must implement the [`StringValueData`] trait, which defines how to parse and
/// serialize the value. This type is used whenever an EML_NL document element or attribute
/// contains a string value that could be parsed, but where strict parsing is not always desired.
#[derive(Debug, Clone)]
pub enum StringValue<T: StringValueData> {
    /// A raw unparsed string value that potentially can be parsed into a value of type `T`.
    Raw(String),
    /// Parsed value of type `T`.
    Parsed(T),
}

impl<T: StringValueData> StringValue<T> {
    /// Try to create a [`StringValue`] from the given raw string by parsing it.
    pub fn from_raw_parsed(s: impl AsRef<str>) -> Result<Self, T::Error> {
        let v = T::parse_from_str(s.as_ref())?;
        Ok(StringValue::Parsed(v))
    }

    /// Create a [`StringValue`] from a raw string.
    pub fn from_raw(s: impl Into<String>) -> Self {
        StringValue::Raw(s.into())
    }

    /// Create a [`StringValue`] from a parsed value.
    pub fn from_value(v: T) -> Self {
        StringValue::Parsed(v)
    }

    /// Get the raw string value.
    pub fn raw(&self) -> Cow<'_, str> {
        match self {
            StringValue::Raw(s) => Cow::Borrowed(s),
            StringValue::Parsed(v) => Cow::Owned(v.to_raw_value()),
        }
    }

    /// Get the parsed value, returning any possible parsing errors.
    pub fn value(&self) -> Result<Cow<'_, T>, T::Error> {
        match self {
            StringValue::Raw(s) => {
                let v = T::parse_from_str(s)?;
                Ok(Cow::Owned(v))
            }
            StringValue::Parsed(v) => Ok(Cow::Borrowed(v)),
        }
    }

    /// Get the parsed value, returning any possible parsing errors as an [`EMLError`].
    ///
    /// The `element_name` and `span` parameters are used to provide context in the error
    /// if parsing fails.
    pub fn value_err<'a, 'b>(
        &self,
        element_name: impl Into<QualifiedName<'a, 'b>>,
        span: Option<Span>,
    ) -> Result<Cow<'_, T>, EMLError> {
        self.value()
            .map_err(|e| EMLError::invalid_value(element_name.into().as_owned(), e, span))
    }
}

impl StringValueData for String {
    type Error = Infallible;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(s.to_string())
    }

    fn to_raw_value(&self) -> String {
        self.clone()
    }
}

impl StringValueData for u64 {
    type Error = std::num::ParseIntError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        s.parse::<u64>()
    }

    fn to_raw_value(&self) -> String {
        self.to_string()
    }
}

impl StringValueData for NonZeroU64 {
    type Error = std::num::ParseIntError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        s.parse()
    }

    fn to_raw_value(&self) -> String {
        self.get().to_string()
    }
}
