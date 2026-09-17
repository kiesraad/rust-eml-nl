use thiserror::Error;

use crate::{
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName},
    utils::{StringValue, StringValueData},
    EMLError, EMLValueResultExt as _, EMLVersion, EMLVersionRange, NS_KR,
};

/// Represents the counting method used for counting votes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CountingMethod(StringValue<CountingMethodCode>);

impl CountingMethod {
    /// Creates a new [`CountingMethod`] from the given [`CountingMethodCode`].
    pub fn from(code: impl Into<CountingMethodCode>) -> Self {
        Self(StringValue::Parsed(code.into()))
    }

    /// Returns the parsed value of the counting method.
    pub fn copied_value(&self) -> Result<CountingMethodCode, EMLError> {
        self.0.copied_value()
    }

    /// Returns a reference to the underlying [`StringValue`] of the counting method.
    pub fn value(&self) -> &StringValue<CountingMethodCode> {
        &self.0
    }
}

impl From<CountingMethodCode> for CountingMethod {
    fn from(code: CountingMethodCode) -> Self {
        Self(StringValue::Parsed(code))
    }
}

impl EMLElement for CountingMethod {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("CountingMethod", Some(NS_KR));
    const EML_VERSIONS: EMLVersionRange = EMLVersionRange::since(EMLVersion::V1_3);

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let code = elem.string_value_attr("MethodCode", None)?;
        Ok(Self(code))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer.attr("MethodCode", self.0.raw().as_ref())?.empty()
    }
}

/// Represents the counting method used for counting votes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CountingMethodCode {
    /// Centrally counted ('centrale stemopneming')
    CSO,
    /// Decentrally counted ('decentrale stemopneming')
    DSO,
}

impl CountingMethodCode {
    /// Create a new CountingMethodCode from a string, validating its format.
    pub fn new(s: impl AsRef<str>) -> Result<Self, EMLError> {
        Self::from_eml_value(s).wrap_value_error()
    }

    /// Create a CountingMethodCode from a `&str`, if possible.
    pub fn from_eml_value(s: impl AsRef<str>) -> Result<Self, UnknownCountingMethodCodeError> {
        let data = s.as_ref();
        match data {
            "centrale stemopneming" => Ok(CountingMethodCode::CSO),
            "decentrale stemopneming" => Ok(CountingMethodCode::DSO),
            _ => Err(UnknownCountingMethodCodeError(data.to_string())),
        }
    }

    /// Get the `&str` representation of this CountingMethodCode.
    pub fn to_eml_value(&self) -> &'static str {
        match self {
            CountingMethodCode::CSO => "centrale stemopneming",
            CountingMethodCode::DSO => "decentrale stemopneming",
        }
    }
}

/// Error returned when an unknown counting method string is encountered.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("Unknown counting method: {0}")]
pub struct UnknownCountingMethodCodeError(String);

impl From<UnknownCountingMethodCodeError> for EMLError {
    fn from(err: UnknownCountingMethodCodeError) -> Self {
        EMLError::value_conversion(err)
    }
}

impl StringValueData for CountingMethodCode {
    type Error = UnknownCountingMethodCodeError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Self::from_eml_value(s)
    }

    fn to_raw_value(&self) -> Box<str> {
        self.to_eml_value().into()
    }
}
