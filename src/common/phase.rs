use thiserror::Error;

use crate::{
    EMLError, EMLValueResultExt as _, EMLVersion, EMLVersionRange, NS_KR,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName},
    utils::{StringValue, StringValueData},
};

/// The 'phase' of a count. PhaseCodes correspond to the names of the proces-verbaal
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Phase(StringValue<PhaseCode>);

impl Phase {
    /// Creates a new [`Phase`] from the given [`PhaseCode`].
    pub fn new(code: impl Into<PhaseCode>) -> Self {
        Self(StringValue::Parsed(code.into()))
    }

    /// Returns the parsed value of the phase
    pub fn copied_value(&self) -> Result<PhaseCode, EMLError> {
        self.0.copied_value()
    }

    /// Returns a reference to the underlying [`StringValue`] of the phase
    pub fn value(&self) -> &StringValue<PhaseCode> {
        &self.0
    }
}

/// The phase of a count
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhaseCode {
    /// First session ("eerste zitting")
    FirstSession,
    /// Corrigendum ("corrigendum")
    Corrigendum,
}

impl PhaseCode {
    /// Create a new PhaseCode from a string, validating its format.
    pub fn new(s: impl AsRef<str>) -> Result<Self, EMLError> {
        Self::from_eml_value(s).wrap_value_error()
    }

    /// Create a PhaseCode from a `&str`, if possible.
    pub fn from_eml_value(s: impl AsRef<str>) -> Result<Self, UnknownPhaseCodeError> {
        let data = s.as_ref();
        match data {
            "eerste zitting" => Ok(PhaseCode::FirstSession),
            "corrigendum" => Ok(PhaseCode::Corrigendum),
            _ => Err(UnknownPhaseCodeError(data.to_string())),
        }
    }

    /// Get the `&str` representation of this PhaseCode.
    pub fn to_eml_value(&self) -> &'static str {
        match self {
            PhaseCode::FirstSession => "eerste zitting",
            PhaseCode::Corrigendum => "corrigendum",
        }
    }
}

impl From<PhaseCode> for Phase {
    fn from(code: PhaseCode) -> Self {
        Self(StringValue::Parsed(code))
    }
}

impl EMLElement for Phase {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Phase", Some(NS_KR));
    const EML_VERSIONS: EMLVersionRange = EMLVersionRange::since(EMLVersion::V1_3);

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let code = elem.string_value_attr("PhaseCode", None)?;
        Ok(Self(code))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer.attr("PhaseCode", self.0.raw().as_ref())?.empty()
    }
}

/// Error returned when an unknown phase string is encountered.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("Unknown phase: {0}")]
pub struct UnknownPhaseCodeError(String);

impl From<UnknownPhaseCodeError> for EMLError {
    fn from(err: UnknownPhaseCodeError) -> Self {
        EMLError::value_conversion(err)
    }
}

impl StringValueData for PhaseCode {
    type Error = UnknownPhaseCodeError;

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
