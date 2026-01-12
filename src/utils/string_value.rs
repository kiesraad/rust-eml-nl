use std::{borrow::Cow, convert::Infallible};

use crate::{
    EMLError,
    io::{EMLElementReader, QualifiedName, Span},
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

    /// Try to create a [`StringValue`] from the given string. If the
    /// `strict_value_parsing` parameter is true, parsing of the string will be
    /// attempted, otherwise the string will be stored unparsed as a raw value.
    ///
    /// During parsing of EML document it might be useful to use the
    /// [`StringValue::from_maybe_parsed_err`] method instead, which returns
    /// parsing errors as [`EMLError`]s with context provided.
    pub fn from_maybe_parsed(s: String, strict_value_parsing: bool) -> Result<Self, T::Error> {
        if strict_value_parsing {
            Self::from_raw_parsed(s)
        } else {
            Ok(StringValue::Raw(s))
        }
    }

    /// Try to create a [`StringValue`] from the given string. If the
    /// `strict_value_parsing` parameter is true, parsing of the string will be
    /// attempted, otherwise the string will be stored unparsed as a raw value.
    ///
    /// In case of parsing errors an [`EMLError`] is returned. The `element_name`
    /// and `span` parameters are used to provide context in the error if parsing
    /// fails in strict mode.
    pub fn from_maybe_parsed_err<'a, 'b>(
        text: String,
        strict_value_parsing: bool,
        element_name: impl Into<QualifiedName<'a, 'b>>,
        span: Option<Span>,
    ) -> Result<Self, EMLError> {
        Self::from_maybe_parsed(text, strict_value_parsing)
            .map_err(|e| EMLError::invalid_value(element_name.into().as_owned(), e, span))
    }

    /// Given an [`EMLElementReader`], read all text from the element and put it
    /// in a StringValue, parsing it if strict parsing is enabled.
    ///
    /// In case of parsing errors an [`EMLError`] is returned. The `element_name`
    /// parameter is used to provide context in the error if parsing fails in
    /// strict mode.
    pub(crate) fn from_maybe_read_parsed_err<'a, 'b>(
        elem: &mut EMLElementReader<'a, 'b>,
        element_name: impl Into<QualifiedName<'a, 'b>>,
    ) -> Result<Self, EMLError> {
        let text = elem.text_without_children()?;
        Self::from_maybe_parsed_err(
            text,
            elem.strict_value_parsing(),
            element_name,
            Some(elem.inner_span()),
        )
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
