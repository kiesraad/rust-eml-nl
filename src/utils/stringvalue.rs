use std::{borrow::Cow, convert::Infallible};

pub trait StringValueData: Clone {
    type Error: std::error::Error;

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
    pub fn from_raw_parsed(s: impl AsRef<String>) -> Result<Self, T::Error> {
        let v = T::parse_from_str(s.as_ref())?;
        Ok(StringValue::Parsed(v))
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
