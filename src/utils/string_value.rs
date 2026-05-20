use std::{borrow::Cow, convert::Infallible, fmt, num::NonZeroU64};

use instant_xml::{Deserializer, FromXml, Id, Kind, Serializer, ToXml};
use thiserror::Error;

use crate::{EMLError, EMLValueResultExt as _};

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
///
/// Depending on the parsing mode used when parsing a document, [`StringValue`]
/// instances may either contain the raw string or the parsed value. When
/// [`EMLParsingMode::Strict`](crate::io::EMLParsingMode::Strict) is used, all
/// [`StringValue`] instances will contain the parsed value. When
/// [`EMLParsingMode::StrictFallback`](crate::io::EMLParsingMode::StrictFallback)
/// is used, the library will attempt to parse values but will fall back to
/// storing the raw string if parsing fails. When
/// [`EMLParsingMode::Loose`](crate::io::EMLParsingMode::Loose) is used, the
/// library will not even attempt to parse values and will store all values as
/// raw strings.
///
/// In most cases you will want to use the parsed value by using one of the
/// value retrieving methods:
/// - [`StringValue::value`]: Get a [`Cow`] containing either a borrowed
///   reference to the parsed value or an owned parsed value if the StringValue
///   contains a raw string. If the StringValue contains a raw string and
///   parsing fails, the error from the parsing attempt will be returned as an
///   error.
/// - [`StringValue::copied_value`]: Same as `value`, but only available for
///   types that implement [`Copy`] and returns a copy of the value instead.
/// - [`StringValue::cloned_value`]: Same as `value`, but returns a cloned copy
///   of the value instead of a reference. Only available for types that
///   implement [`Clone`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    pub fn value_err(&self) -> Result<Cow<'_, T>, T::Error> {
        match self {
            StringValue::Raw(s) => {
                let v = T::parse_from_str(s)?;
                Ok(Cow::Owned(v))
            }
            StringValue::Parsed(v) => Ok(Cow::Borrowed(v)),
        }
    }

    /// Get the parsed value, returning a cloned copy of it.
    ///
    /// If there is an error, the original parsing error will be returned.
    pub fn cloned_value_err(&self) -> Result<T, T::Error> {
        self.value_err().map(|v| v.into_owned())
    }

    /// Get the parsed value, returning a cloned copy of it.
    ///
    /// If there is an error, it will be wrapped in an [`EMLError`].
    pub fn cloned_value(&self) -> Result<T, EMLError> {
        self.cloned_value_err().wrap_value_error()
    }

    /// Get the parsed value, returning any possible parsing errors as an [`EMLError`].
    ///
    /// The `element_name` and `span` parameters are used to provide context in the error
    /// if parsing fails.
    pub fn value(&self) -> Result<Cow<'_, T>, EMLError> {
        self.value_err().wrap_value_error()
    }
}

impl<T: StringValueData + Copy> StringValue<T> {
    /// Get the parsed value, returning a copy of it.
    ///
    /// This is only available for types that implement [`Copy`].
    pub fn copied_value_err(&self) -> Result<T, T::Error> {
        Ok(*self.value_err()?.as_ref())
    }

    /// Get the parsed value, returning a copy of it.
    ///
    /// If there is an error, it will be wrapped in an [`EMLError`].
    /// This is only available for types that implement [`Copy`].
    pub fn copied_value(&self) -> Result<T, EMLError> {
        self.copied_value_err().wrap_value_error()
    }
}

impl<'xml, T: StringValueData> FromXml<'xml> for StringValue<T> {
    #[inline]
    fn matches(id: Id<'_>, field: Option<Id<'_>>) -> bool {
        match field {
            Some(field) => id == field,
            None => false,
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut Deserializer<'cx, 'xml>,
    ) -> Result<(), instant_xml::Error> {
        if into.is_some() {
            return Err(instant_xml::Error::DuplicateValue(field));
        }

        let text = deserializer.take_str()?;
        let raw = text.as_deref().unwrap_or("");
        let value = match T::parse_from_str(raw) {
            Ok(v) => StringValue::Parsed(v),
            Err(_) => StringValue::Raw(raw.to_owned()),
        };

        *into = Some(value);
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: Kind = Kind::Scalar;
}

impl<T: StringValueData> ToXml for StringValue<T> {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        field: Option<Id<'_>>,
        serializer: &mut Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        self.raw().serialize(field, serializer)
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

/// Error returned when parsing a boolean value fails.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("Failed to parse boolean value")]
pub struct ParseBoolError;

impl StringValueData for bool {
    type Error = ParseBoolError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(match s {
            "0" | "false" => false,
            "1" | "true" => true,
            _ => return Err(ParseBoolError),
        })
    }

    fn to_raw_value(&self) -> String {
        // Note: We use "true" and "false" as the raw string representation for
        // boolean values, as this is more human-readable than "1" and "0".
        self.to_string()
    }
}
