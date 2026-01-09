use std::{borrow::Cow, convert::Infallible};

use chrono::{DateTime, Utc};

use crate::{EMLError, Span};

pub trait StringValueData: Clone {
    type Error: std::error::Error + Send + Sync + 'static;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized;

    fn to_raw_value(&self) -> String;
}

#[derive(Debug, Clone)]
pub enum StringValue<T: StringValueData> {
    Raw(String),
    Parsed(T),
}

impl<T: StringValueData> StringValue<T> {
    pub fn from_raw_parsed(s: impl AsRef<str>) -> Result<Self, T::Error> {
        let v = T::parse_from_str(s.as_ref())?;
        Ok(StringValue::Parsed(v))
    }

    pub fn from_maybe_parsed(s: String, strict_value_parsing: bool) -> Result<Self, T::Error> {
        if strict_value_parsing {
            Self::from_raw_parsed(s)
        } else {
            Ok(StringValue::Raw(s))
        }
    }

    pub fn from_maybe_parsed_err(
        text: String,
        strict_value_parsing: bool,
        element_name: &'static str,
        span: Option<Span>,
    ) -> Result<Self, EMLError> {
        Self::from_maybe_parsed(text, strict_value_parsing)
            .map_err(|e| EMLError::invalid_value(element_name, e, span))
    }

    pub fn from_raw(s: impl Into<String>) -> Self {
        StringValue::Raw(s.into())
    }

    pub fn from_value(v: T) -> Self {
        StringValue::Parsed(v)
    }

    pub fn raw(&self) -> Cow<'_, str> {
        match self {
            StringValue::Raw(s) => Cow::Borrowed(s),
            StringValue::Parsed(v) => Cow::Owned(v.to_raw_value()),
        }
    }

    pub fn value(&self) -> Result<Cow<'_, T>, T::Error> {
        match self {
            StringValue::Raw(s) => {
                let v = T::parse_from_str(s)?;
                Ok(Cow::Owned(v))
            }
            StringValue::Parsed(v) => Ok(Cow::Borrowed(v)),
        }
    }

    pub fn value_err(
        &self,
        element_name: &'static str,
        span: Option<Span>,
    ) -> Result<Cow<'_, T>, EMLError> {
        self.value()
            .map_err(|e| EMLError::invalid_value(element_name, e, span))
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

impl StringValueData for DateTime<Utc> {
    type Error = chrono::ParseError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            Ok(dt.with_timezone(&Utc))
        } else {
            // Fallback to parsing without timezone info, assuming UTC
            let naive_dt = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f")?;
            Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc))
        }
    }

    fn to_raw_value(&self) -> String {
        self.to_rfc3339()
    }
}
