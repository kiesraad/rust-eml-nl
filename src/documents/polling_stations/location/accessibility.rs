use thiserror::Error;

use crate::{
    EMLError, EMLVersion, EMLVersionRange, NS_SB,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName, collect_struct},
    utils::{StringValue, StringValueData},
};

/// Accessibility information about a polling station location
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Accessibility {
    /// Whether the polling station is accessible
    pub accessible: bool,
    /// Accessibility properties of the polling station location
    pub properties: Option<AccessibilityProperties>,
}

impl Accessibility {
    /// Creates a new `Accessibility` with default values.
    pub fn new(accessible: bool) -> Self {
        Self {
            accessible,
            properties: None,
        }
    }

    /// Sets the accessibility status of the polling station.
    pub fn with_accessible(accessible: bool) -> Self {
        Self {
            accessible,
            properties: None,
        }
    }

    /// Sets the accessibility properties of the polling station.
    pub fn with_properties(self, properties: impl Into<AccessibilityProperties>) -> Self {
        Self {
            accessible: self.accessible,
            properties: Some(properties.into()),
        }
    }

    /// Optionally sets the accessibility properties of the polling station.
    pub fn with_properties_option(
        self,
        properties: Option<impl Into<AccessibilityProperties>>,
    ) -> Self {
        Self {
            accessible: self.accessible,
            properties: properties.map(|p| p.into()),
        }
    }
}

impl EMLElement for Accessibility {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("Accessibility", Some(NS_SB));
    const EML_VERSIONS: EMLVersionRange = EMLVersionRange::since(EMLVersion::V1_3);

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, Accessibility {
            accessible: ("Accessible", Some(NS_SB)) => |elem| elem.read_bool()?,
            properties as Option: AccessibilityProperties::EML_NAME => |elem| elem.read_element::<AccessibilityProperties>()?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child(("Accessible", Some(NS_SB)), |writer| {
                writer.bool(self.accessible)
            })?
            .child_elem_option(AccessibilityProperties::EML_NAME, self.properties.as_ref())?
            .finish()
    }
}

/// Accessibility properties of a polling station location
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AccessibilityProperties {
    /// Whether the polling station is easily accessible by public transport
    pub accessible_public_transport: Option<bool>,

    /// Whether the polling station is accessible by a wheelchair
    pub accessible_wheelchair: Option<bool>,

    /// Whether a host is present at the polling station
    pub host_present: Option<bool>,

    /// Accessibility guidelines (i.e. tactile paving) for the polling station
    pub guidelines: Option<StringValue<AccessibleGuidelines>>,

    /// Whether a voting template is available for the polling station
    pub voting_template: Option<bool>,

    /// Whether a braille candidate list is available for the polling station
    pub braille_candidate_list: Option<bool>,

    /// Whether a large-lettered candidate list is available for the polling station
    pub large_lettered_candidate_list: Option<bool>,

    /// Whether a sign language interpreter is present at the polling station
    pub sign_language_interpreter: Option<StringValue<SignLanguageInterpreter>>,

    /// Whether a member of the polling station understands sign language
    pub sign_language_polling_station_member: Option<bool>,

    /// Whether the polling station acoustics are suitable for hearing impaired voters
    pub acoustics_for_hearing_impaired: Option<bool>,

    /// Whether the polling station has a low stimulus environment
    pub low_stimulus_environment: Option<bool>,

    /// Other accessibility properties
    pub others: Vec<String>,
}

impl AccessibilityProperties {
    /// Creates a new `AccessibilityProperties` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the `accessible_public_transport` field.
    pub fn with_accesible_public_transport(self, accessible_public_transport: bool) -> Self {
        Self {
            accessible_public_transport: Some(accessible_public_transport),
            ..self
        }
    }

    /// Optionally sets the `accessible_public_transport` field.
    pub fn with_accessible_public_transport_option(
        self,
        accessible_public_transport: Option<bool>,
    ) -> Self {
        Self {
            accessible_public_transport,
            ..self
        }
    }

    /// Sets the `accessible_wheelchair` field.
    pub fn with_accessible_wheelchair(self, accessible_wheelchair: bool) -> Self {
        Self {
            accessible_wheelchair: Some(accessible_wheelchair),
            ..self
        }
    }

    /// Optionally sets the `accessible_wheelchair` field.
    pub fn with_accessible_wheelchair_option(self, accessible_wheelchair: Option<bool>) -> Self {
        Self {
            accessible_wheelchair,
            ..self
        }
    }

    /// Sets the `host_present` field.
    pub fn with_host_present(self, host_present: bool) -> Self {
        Self {
            host_present: Some(host_present),
            ..self
        }
    }

    /// Optionally sets the `host_present` field.
    pub fn with_host_present_option(self, host_present: Option<bool>) -> Self {
        Self {
            host_present,
            ..self
        }
    }

    /// Optionally sets the `guidelines` field.
    pub fn with_guidelines(self, guidelines: impl Into<AccessibleGuidelines>) -> Self {
        Self {
            guidelines: Some(StringValue::from_value(guidelines.into())),
            ..self
        }
    }

    /// Optionally sets the `guidelines` field.
    pub fn with_guidelines_option(
        self,
        guidelines: Option<impl Into<AccessibleGuidelines>>,
    ) -> Self {
        Self {
            guidelines: guidelines.map(|g| StringValue::from_value(g.into())),
            ..self
        }
    }

    /// Set the `voting_template` field.
    pub fn with_voting_template(self, voting_template: bool) -> Self {
        Self {
            voting_template: Some(voting_template),
            ..self
        }
    }

    /// Optionally sets the `voting_template` field.
    pub fn with_voting_template_option(self, voting_template: Option<bool>) -> Self {
        Self {
            voting_template,
            ..self
        }
    }

    /// Set the `braille_candidate_list` field.
    pub fn with_braille_candidate_list(self, braille_candidate_list: bool) -> Self {
        Self {
            braille_candidate_list: Some(braille_candidate_list),
            ..self
        }
    }

    /// Optionally sets the `braille_candidate_list` field.
    pub fn with_braille_candidate_list_option(self, braille_candidate_list: Option<bool>) -> Self {
        Self {
            braille_candidate_list,
            ..self
        }
    }

    /// Set the `large_lettered_candidate_list` field.
    pub fn with_large_lettered_candidate_list(self, large_lettered_candidate_list: bool) -> Self {
        Self {
            large_lettered_candidate_list: Some(large_lettered_candidate_list),
            ..self
        }
    }

    /// Optionally sets the `large_lettered_candidate_list` field.
    pub fn with_large_lettered_candidate_list_option(
        self,
        large_lettered_candidate_list: Option<bool>,
    ) -> Self {
        Self {
            large_lettered_candidate_list,
            ..self
        }
    }

    /// Set the `sign_language_interpreter` field.
    pub fn with_sign_language_interpreter(
        self,
        sign_language_interpreter: impl Into<SignLanguageInterpreter>,
    ) -> Self {
        Self {
            sign_language_interpreter: Some(StringValue::from_value(
                sign_language_interpreter.into(),
            )),
            ..self
        }
    }

    /// Optionally sets the `sign_language_interpreter` field.
    pub fn with_sign_language_interpreter_option(
        self,
        sign_language_interpreter: Option<impl Into<SignLanguageInterpreter>>,
    ) -> Self {
        Self {
            sign_language_interpreter: sign_language_interpreter
                .map(|i| StringValue::from_value(i.into())),
            ..self
        }
    }

    /// Set the `sign_language_polling_station_member` field.
    pub fn with_sign_language_polling_station_member(
        self,
        sign_language_polling_station_member: bool,
    ) -> Self {
        Self {
            sign_language_polling_station_member: Some(sign_language_polling_station_member),
            ..self
        }
    }

    /// Optionally sets the `sign_language_polling_station_member` field.
    pub fn with_sign_language_polling_station_member_option(
        self,
        sign_language_polling_station_member: Option<bool>,
    ) -> Self {
        Self {
            sign_language_polling_station_member,
            ..self
        }
    }

    /// Set the `acoustics_for_hearing_impaired` field.
    pub fn with_acoustics_for_hearing_impaired(self, acoustics_for_hearing_impaired: bool) -> Self {
        Self {
            acoustics_for_hearing_impaired: Some(acoustics_for_hearing_impaired),
            ..self
        }
    }

    /// Optionally sets the `acoustics_for_hearing_impaired` field.
    pub fn with_acoustics_for_hearing_impaired_option(
        self,
        acoustics_for_hearing_impaired: Option<bool>,
    ) -> Self {
        Self {
            acoustics_for_hearing_impaired,
            ..self
        }
    }

    /// Set the `low_stimulus_environment` field.
    pub fn with_low_stimulus_environment(self, low_stimulus_environment: bool) -> Self {
        Self {
            low_stimulus_environment: Some(low_stimulus_environment),
            ..self
        }
    }

    /// Optionally sets the `low_stimulus_environment` field.
    pub fn with_low_stimulus_environment_option(
        self,
        low_stimulus_environment: Option<bool>,
    ) -> Self {
        Self {
            low_stimulus_environment,
            ..self
        }
    }

    /// Adds an `other` value to the `others` field.
    pub fn with_other(mut self, other: impl Into<String>) -> Self {
        self.others.push(other.into());
        self
    }

    /// Sets the `others` field to the given value.
    ///
    /// This replaces any previously set `others` value.
    pub fn with_others(self, others: impl Into<Vec<String>>) -> Self {
        Self {
            others: others.into(),
            ..self
        }
    }
}

impl EMLElement for AccessibilityProperties {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("AccessibilityProperties", Some(NS_SB));
    const EML_VERSIONS: EMLVersionRange = EMLVersionRange::since(EMLVersion::V1_3);

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, AccessibilityProperties {
            accessible_public_transport as Option: ("AccessiblePublicTransport", Some(NS_SB)) => |elem| elem.read_bool()?,
            accessible_wheelchair as Option: ("AccessibleWheelchair", Some(NS_SB)) => |elem| elem.read_bool()?,
            host_present as Option: ("HostPresent", Some(NS_SB)) => |elem| elem.read_bool()?,
            guidelines as Option: ("Guidelines", Some(NS_SB)) => |elem| elem.string_value()?,
            voting_template as Option: ("VotingTemplate", Some(NS_SB)) => |elem| elem.read_bool()?,
            braille_candidate_list as Option: ("BrailleCandidateList", Some(NS_SB)) => |elem| elem.read_bool()?,
            large_lettered_candidate_list as Option: ("LargeLetteredCandidateList", Some(NS_SB)) => |elem| elem.read_bool()?,
            sign_language_interpreter as Option: ("SignLanguageInterpreter", Some(NS_SB)) => |elem| elem.string_value()?,
            sign_language_polling_station_member as Option: ("SignLanguagePollingStationMember", Some(NS_SB)) => |elem| elem.read_bool()?,
            acoustics_for_hearing_impaired as Option: ("AcousticsForHearingImpaired", Some(NS_SB)) => |elem| elem.read_bool()?,
            low_stimulus_environment as Option: ("LowStimulusEnvironment", Some(NS_SB)) => |elem| elem.read_bool()?,
            others as Vec: ("Other", Some(NS_SB)) => |elem| elem.text_without_children()?.to_string(),
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_option(
                ("AccessiblePublicTransport", Some(NS_SB)),
                self.accessible_public_transport,
                |writer, value| writer.bool(value),
            )?
            .child_option(
                ("AccessibleWheelchair", Some(NS_SB)),
                self.accessible_wheelchair,
                |writer, value| writer.bool(value),
            )?
            .child_option(
                ("HostPresent", Some(NS_SB)),
                self.host_present,
                |writer, value| writer.bool(value),
            )?
            .child_option(
                ("Guidelines", Some(NS_SB)),
                self.guidelines.as_ref(),
                |writer, value| writer.text(value.raw().as_ref())?.finish(),
            )?
            .child_option(
                ("VotingTemplate", Some(NS_SB)),
                self.voting_template,
                |writer, value| writer.bool(value),
            )?
            .child_option(
                ("BrailleCandidateList", Some(NS_SB)),
                self.braille_candidate_list,
                |writer, value| writer.bool(value),
            )?
            .child_option(
                ("LargeLetteredCandidateList", Some(NS_SB)),
                self.large_lettered_candidate_list,
                |writer, value| writer.bool(value),
            )?
            .child_option(
                ("SignLanguageInterpreter", Some(NS_SB)),
                self.sign_language_interpreter.as_ref(),
                |writer, value| writer.text(value.raw().as_ref())?.finish(),
            )?
            .child_option(
                ("SignLanguagePollingStationMember", Some(NS_SB)),
                self.sign_language_polling_station_member,
                |writer, value| writer.bool(value),
            )?
            .child_option(
                ("AcousticsForHearingImpaired", Some(NS_SB)),
                self.acoustics_for_hearing_impaired,
                |writer, value| writer.bool(value),
            )?
            .child_option(
                ("LowStimulusEnvironment", Some(NS_SB)),
                self.low_stimulus_environment,
                |writer, value| writer.bool(value),
            )?
            .child_elems_map(
                ("Other", Some(NS_SB)),
                self.others.as_slice(),
                |writer, value| writer.text(value)?.finish(),
            )?
            .finish()
    }
}

/// Accessibility guidelines (i.e. tactile paving) for the polling station
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessibleGuidelines {
    /// Guidelines are available inside and outside the polling station
    InsideAndOutside,
    /// Guidelines are available outside the polling station
    Outside,
    /// Guidelines are available inside the polling station
    Inside,
    /// No guidelines are available for the polling station
    NotPresent,
}

impl AccessibleGuidelines {
    /// Create an AccessibleGuidelines from a `&str`, if possible.
    pub fn from_eml_value(s: &str) -> Result<Self, InvalidAccessibleGuidelines> {
        match s {
            "inside and outside" => Ok(Self::InsideAndOutside),
            "outside" => Ok(Self::Outside),
            "inside" => Ok(Self::Inside),
            "not present" => Ok(Self::NotPresent),
            _ => Err(InvalidAccessibleGuidelines(s.to_owned())),
        }
    }

    /// Get the `&str` representation of this accessible guidelines value.
    pub fn to_eml_value(&self) -> &str {
        match self {
            Self::InsideAndOutside => "inside and outside",
            Self::Outside => "outside",
            Self::Inside => "inside",
            Self::NotPresent => "not present",
        }
    }
}

/// Represents an invalid accessibility guidelines value
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid accessible guidelines: {0}")]
pub struct InvalidAccessibleGuidelines(String);

impl From<InvalidAccessibleGuidelines> for EMLError {
    fn from(err: InvalidAccessibleGuidelines) -> Self {
        EMLError::value_conversion(err)
    }
}

impl StringValueData for AccessibleGuidelines {
    type Error = InvalidAccessibleGuidelines;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error> {
        Self::from_eml_value(s)
    }

    fn to_raw_value(&self) -> Box<str> {
        self.to_eml_value().into()
    }
}

/// Whether a sign language interpreter is present at the polling station
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignLanguageInterpreter {
    /// The sign language interpreter is present at the polling station
    AtLocation,
    /// The sign language interpreter is available remotely
    Remote,
    /// No sign language interpreter is present
    NotPresent,
}

impl SignLanguageInterpreter {
    /// Create a SignLanguageInterpreter from a `&str`, if possible.
    pub fn from_eml_value(s: impl AsRef<str>) -> Result<Self, InvalidSignLanguageInterpreter> {
        let data = s.as_ref();
        match data {
            "at location" => Ok(Self::AtLocation),
            "remote" => Ok(Self::Remote),
            "not present" => Ok(Self::NotPresent),
            _ => Err(InvalidSignLanguageInterpreter(data.to_string())),
        }
    }

    /// Get the `&str` representation of this sign language interpreter value.
    pub fn to_eml_value(&self) -> &'static str {
        match self {
            Self::AtLocation => "at location",
            Self::Remote => "remote",
            Self::NotPresent => "not present",
        }
    }
}

/// Represents an invalid sign language interpreter value
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid sign language interpreter: {0}")]
pub struct InvalidSignLanguageInterpreter(String);

impl From<InvalidSignLanguageInterpreter> for EMLError {
    fn from(err: InvalidSignLanguageInterpreter) -> Self {
        EMLError::value_conversion(err)
    }
}

impl StringValueData for SignLanguageInterpreter {
    type Error = InvalidSignLanguageInterpreter;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error> {
        Self::from_eml_value(s)
    }

    fn to_raw_value(&self) -> Box<str> {
        self.to_eml_value().into()
    }
}
