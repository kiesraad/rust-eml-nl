use std::fmt;

use instant_xml::{FromXml, ToXml};

use crate::NS_XNL;

/// Container for details of the name of a person.
#[derive(Debug, Clone)]
pub struct PersonNameStructure {
    /// The person's name details.
    pub person_name: PersonName,

    /// The PartyType attribute of the PersonNameStructure
    pub party_type: Option<String>,

    /// The Code attribute of the PersonNameStructure
    pub code: Option<String>,
}

impl<'xml> FromXml<'xml> for PersonNameStructure {
    fn matches(id: instant_xml::Id<'_>, field: Option<instant_xml::Id<'_>>) -> bool {
        // PersonNameStructure uses the parent's field name (CandidateFullName, AgentName, Name, etc.)
        match field {
            Some(field) => id == field,
            None => false,
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut instant_xml::Deserializer<'cx, 'xml>,
    ) -> Result<(), instant_xml::Error> {
        use instant_xml::{Accumulate, Error, de::Node};
        if into.is_some() {
            return Err(Error::DuplicateValue(field));
        }

        let mut person_name = <PersonName as FromXml>::Accumulator::default();
        let mut party_type = None;
        let mut code = None;

        while let Some(node) = deserializer.next() {
            match node? {
                Node::Attribute(attr) => match attr.local {
                    "PartyType" => party_type = Some(attr.value.into_owned()),
                    "Code" => code = Some(attr.value.into_owned()),
                    name => return Err(Error::UnexpectedValue(name.to_owned())),
                },
                Node::Open(element) => {
                    let id = deserializer.element_id(&element)?;
                    if PersonName::matches(id, None) {
                        let mut nested = deserializer.nested(element);
                        PersonName::deserialize(&mut person_name, field, &mut nested)?;
                        nested.ignore()?;
                    } else {
                        let mut nested = deserializer.nested(element);
                        nested.ignore()?;
                    }
                }
                Node::Text(s) if s.trim().is_empty() => continue,
                node => return Err(Error::UnexpectedNode(format!("{node:?}"))),
            }
        }

        *into = Some(PersonNameStructure {
            person_name: person_name.try_done(field)?,
            party_type,
            code,
        });
        Ok(())
    }

    type Accumulator = Option<Self>;
    // Scalar KIND so that the parent's field name/rename is passed via `field` parameter
    const KIND: instant_xml::Kind = instant_xml::Kind::Scalar;
}

impl PersonNameStructure {
    /// Create a new `PersonNameStructure` with the given `PersonName`.
    pub fn new(person_name: PersonName) -> Self {
        PersonNameStructure {
            person_name,
            party_type: None,
            code: None,
        }
    }

    /// Set the `PartyType` attribute of the `PersonNameStructure`.
    pub fn with_party_type(mut self, party_type: impl Into<String>) -> Self {
        self.party_type = Some(party_type.into());
        self
    }

    /// Set the `Code` attribute of the `PersonNameStructure`.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

// Custom: conditionally wraps content in outer element with optional attributes.
impl ToXml for PersonNameStructure {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix = if let Some(field) = field {
            Some(serializer.write_start(
                field.name,
                field.ns,
                None::<instant_xml::ser::Context<0>>,
            )?)
        } else {
            None
        };

        if let Some(pt) = &self.party_type {
            serializer.write_attr("PartyType", "", pt)?;
        }
        if let Some(c) = &self.code {
            serializer.write_attr("Code", "", c)?;
        }

        serializer.end_start()?;
        self.person_name.serialize(None, serializer)?;
        if let Some(prefix) = prefix {
            serializer.write_close(prefix)?;
        }

        Ok(())
    }
}

impl From<PersonName> for PersonNameStructure {
    fn from(value: PersonName) -> Self {
        PersonNameStructure::new(value)
    }
}

/// Details of the name of a person.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(ns(NS_XNL), force_prefix)]
pub struct PersonName {
    /// The initials of the person.
    #[xml(rename = "NameLine")]
    pub name_line_initials: Option<NameLineInitials>,

    /// The first name of the person.
    #[xml(rename = "FirstName")]
    pub first_name: Option<FirstName>,

    /// The prefix of the person's last name.
    #[xml(rename = "NamePrefix")]
    pub name_prefix: Option<NamePrefix>,

    /// The last name of the person.
    #[xml(rename = "LastName")]
    pub last_name: LastName,

    /// The Type attribute of the PersonName
    #[xml(attribute, rename = "Type")]
    pub person_name_type: Option<String>,

    /// The Code attribute of the PersonName
    #[xml(attribute, rename = "Code")]
    pub code: Option<String>,

    /// The NameDetailsKeyRef attribute of the PersonName
    #[xml(attribute, rename = "NameDetailsKeyRef")]
    pub name_details_key_ref: Option<String>,
}

impl PersonName {
    /// Create a new PersonName.
    pub fn new(last_name: impl Into<String>) -> Self {
        Self {
            name_line_initials: None,
            first_name: None,
            name_prefix: None,
            last_name: LastName::new(last_name),
            person_name_type: None,
            code: None,
            name_details_key_ref: None,
        }
    }

    /// Set the initials of the person.
    pub fn with_initials(self, initials: impl Into<String>) -> Self {
        self.with_initials_option(Some(initials))
    }

    /// Set the initials of the person, if present.
    pub fn with_initials_option(mut self, initials: Option<impl Into<String>>) -> Self {
        self.name_line_initials = initials.map(|i| NameLineInitials::new(i));
        self
    }

    /// Set the first name of the person.
    pub fn with_first_name(self, first_name: impl Into<String>) -> Self {
        self.with_first_name_option(Some(first_name))
    }

    /// Set the first name of the person, if present.
    pub fn with_first_name_option(mut self, first_name: Option<impl Into<String>>) -> Self {
        self.first_name = first_name.map(|f| FirstName::new(f));
        self
    }

    /// Set the prefix of the person's last name.
    pub fn with_name_prefix(self, name_prefix: impl Into<String>) -> Self {
        self.with_name_prefix_option(Some(name_prefix))
    }

    /// Set the prefix of the person's last name, if present.
    pub fn with_name_prefix_option(mut self, name_prefix: Option<impl Into<String>>) -> Self {
        self.name_prefix = name_prefix.map(|p| NamePrefix::new(p));
        self
    }

    /// Set the `Type` attribute of the PersonName.
    pub fn with_type(mut self, person_name_type: impl Into<String>) -> Self {
        self.person_name_type = Some(person_name_type.into());
        self
    }

    /// Set the `Code` attribute of the PersonName.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Set the `NameDetailsKeyRef` attribute of the PersonName.
    pub fn with_name_details_key_ref(mut self, name_details_key_ref: impl Into<String>) -> Self {
        self.name_details_key_ref = Some(name_details_key_ref.into());
        self
    }

    /// Get the last name of the person.
    pub fn get_last_name(&self) -> &str {
        &self.last_name.value
    }

    /// Get the initials of the person, if present.
    pub fn get_initials(&self) -> Option<&str> {
        self.name_line_initials
            .as_ref()
            .map(|initials| initials.value.as_str())
    }

    /// Get the first name of the person, if present.
    pub fn get_first_name(&self) -> Option<&str> {
        self.first_name
            .as_ref()
            .map(|first_name| first_name.value.as_str())
    }

    /// Get the prefix of the person's last name, if present.
    pub fn get_name_prefix(&self) -> Option<&str> {
        self.name_prefix
            .as_ref()
            .map(|prefix| prefix.value.as_str())
    }
}

/// Details of the initials line of a person's name.
#[derive(Debug, Clone, FromXml)]
#[xml(rename = "NameLine", ns(NS_XNL), force_prefix)]
pub struct NameLineInitials {
    /// The NameType attribute (expected to be "Initials")
    #[xml(attribute, rename = "NameType")]
    pub name_type_attr: Option<String>,

    /// The Type attribute of the NameLineInitials
    #[xml(attribute, rename = "Type")]
    pub name_line_type: Option<String>,

    /// The Code attribute of the NameLineInitials
    #[xml(attribute, rename = "Code")]
    pub code: Option<String>,

    /// The initials value.
    #[xml(direct)]
    pub value: String,
}

impl NameLineInitials {
    /// Create a new NameLineInitials.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            name_type_attr: None,
            name_line_type: None,
            code: None,
            value: value.into(),
        }
    }

    /// Set the `Type` attribute of the element.
    pub fn with_type(mut self, name_line_type: impl Into<String>) -> Self {
        self.name_line_type = Some(name_line_type.into());
        self
    }

    /// Set the `Code` attribute of the element.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

// Custom: hardcodes a constant `NameType="Initials"` attribute not mapped to a field.
impl ToXml for NameLineInitials {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let mut cx = instant_xml::ser::Context::<0>::default();
        cx.default_ns = NS_XNL;
        cx.force_prefix = true;
        let prefix = serializer.write_start("NameLine", NS_XNL, Some(cx))?;

        serializer.write_attr("NameType", "", "Initials")?;
        if let Some(t) = &self.name_line_type {
            serializer.write_attr("Type", "", t)?;
        }
        if let Some(c) = &self.code {
            serializer.write_attr("Code", "", c)?;
        }
        serializer.end_start()?;

        serializer.write_str(&self.value)?;
        serializer.write_close(prefix)
    }
}

/// Details of the first name of a person.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(ns(NS_XNL), force_prefix)]
pub struct FirstName {
    /// The Type attribute of the FirstName
    #[xml(attribute, rename = "Type")]
    pub first_name_type: Option<String>,

    /// The NameType attribute of the name
    #[xml(attribute, rename = "NameType")]
    pub name_type: Option<String>,

    /// The Code attribute of the FirstName
    #[xml(attribute, rename = "Code")]
    pub code: Option<String>,

    /// The first name value.
    #[xml(direct)]
    pub value: String,
}

impl FirstName {
    /// Create a new FirstName.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            first_name_type: None,
            name_type: None,
            code: None,
        }
    }

    /// Set the `Type` attribute of the element.
    pub fn with_type(mut self, first_name_type: impl Into<String>) -> Self {
        self.first_name_type = Some(first_name_type.into());
        self
    }

    /// Set the `NameType` attribute of the element.
    pub fn with_name_type(mut self, name_type: impl Into<String>) -> Self {
        self.name_type = Some(name_type.into());
        self
    }

    /// Set the `Code` attribute of the element.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

/// Details of the prefix of a person's last name.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(ns(NS_XNL), force_prefix)]
pub struct NamePrefix {
    /// The Type attribute of the NamePrefix
    #[xml(attribute, rename = "Type")]
    pub name_prefix_type: Option<String>,

    /// The NameType attribute of the NamePrefix
    #[xml(attribute, rename = "NameType")]
    pub name_type: Option<String>,

    /// The Code attribute of the NamePrefix
    #[xml(attribute, rename = "Code")]
    pub code: Option<String>,

    /// The prefix value.
    #[xml(direct)]
    pub value: String,
}

impl NamePrefix {
    /// Create a new NamePrefix.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            name_prefix_type: None,
            name_type: None,
            code: None,
        }
    }

    /// Set the `Type` attribute of the element.
    pub fn with_type(mut self, name_prefix_type: impl Into<String>) -> Self {
        self.name_prefix_type = Some(name_prefix_type.into());
        self
    }

    /// Set the `NameType` attribute of the element.
    pub fn with_name_type(mut self, name_type: impl Into<String>) -> Self {
        self.name_type = Some(name_type.into());
        self
    }

    /// Set the `Code` attribute of the element.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

/// Details of the last name of a person.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(ns(NS_XNL), force_prefix)]
pub struct LastName {
    /// The Type attribute of the LastName
    #[xml(attribute, rename = "Type")]
    pub last_name_type: Option<String>,

    /// The NameType attribute of the LastName
    #[xml(attribute, rename = "NameType")]
    pub name_type: Option<String>,

    /// The Code attribute of the LastName
    #[xml(attribute, rename = "Code")]
    pub code: Option<String>,

    /// The last name value.
    #[xml(direct)]
    pub value: String,
}

impl LastName {
    /// Create a new LastName.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            last_name_type: None,
            name_type: None,
            code: None,
        }
    }

    /// Set the `Type` attribute of the element.
    pub fn with_type(mut self, last_name_type: impl Into<String>) -> Self {
        self.last_name_type = Some(last_name_type.into());
        self
    }

    /// Set the `NameType` attribute of the element.
    pub fn with_name_type(mut self, name_type: impl Into<String>) -> Self {
        self.name_type = Some(name_type.into());
        self
    }

    /// Set the `Code` attribute of the element.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{EMLParsingMode, EMLRead as _, test_xml_fragment};

    #[test]
    fn test_person_name_construction() {
        let person_name = PersonName::new("Test")
            .with_initials("J.")
            .with_first_name("Jan")
            .with_name_prefix("van")
            .with_type("TestType")
            .with_code("TestCode")
            .with_name_details_key_ref("TestKeyRef");

        assert_eq!(person_name.last_name.value, "Test");
        assert_eq!(person_name.name_line_initials.as_ref().unwrap().value, "J.");
        assert_eq!(person_name.first_name.as_ref().unwrap().value, "Jan");
        assert_eq!(person_name.name_prefix.as_ref().unwrap().value, "van");
        assert_eq!(person_name.person_name_type.as_deref(), Some("TestType"));
        assert_eq!(person_name.code.as_deref(), Some("TestCode"));
        assert_eq!(
            person_name.name_details_key_ref.as_deref(),
            Some("TestKeyRef")
        );
    }

    #[test]
    fn test_person_name_structure_construction() {
        let person_name = PersonName::new("Test");
        let person_name_structure = PersonNameStructure::new(person_name)
            .with_party_type("TestPartyType")
            .with_code("TestCode");

        assert_eq!(person_name_structure.person_name.last_name.value, "Test");
        assert_eq!(
            person_name_structure.party_type.as_deref(),
            Some("TestPartyType")
        );
        assert_eq!(person_name_structure.code.as_deref(), Some("TestCode"));
    }

    #[test]
    fn test_person_name_structure_construction_without_optional_fields() {
        let person_name = PersonName::new("Test");
        let person_name_structure = PersonNameStructure::new(person_name);

        assert_eq!(person_name_structure.person_name.last_name.value, "Test");
        assert!(person_name_structure.party_type.is_none());
        assert!(person_name_structure.code.is_none());
    }

    #[test]
    fn test_name_line_initials_construction() {
        let initials = NameLineInitials::new("J.")
            .with_type("Initials")
            .with_code("TestCode");
        assert_eq!(initials.value, "J.");
        assert_eq!(initials.name_line_type.as_deref(), Some("Initials"));
        assert_eq!(initials.code.as_deref(), Some("TestCode"));
    }

    #[test]
    fn test_first_name_construction() {
        let first_name = FirstName::new("Jan")
            .with_type("FirstNameType")
            .with_name_type("FirstName")
            .with_code("TestCode");
        assert_eq!(first_name.value, "Jan");
        assert_eq!(first_name.first_name_type.as_deref(), Some("FirstNameType"));
        assert_eq!(first_name.name_type.as_deref(), Some("FirstName"));
        assert_eq!(first_name.code.as_deref(), Some("TestCode"));
    }

    #[test]
    fn test_name_prefix_construction() {
        let name_prefix = NamePrefix::new("van")
            .with_type("NamePrefixType")
            .with_name_type("NamePrefix")
            .with_code("TestCode");
        assert_eq!(name_prefix.value, "van");
        assert_eq!(
            name_prefix.name_prefix_type.as_deref(),
            Some("NamePrefixType")
        );
        assert_eq!(name_prefix.name_type.as_deref(), Some("NamePrefix"));
        assert_eq!(name_prefix.code.as_deref(), Some("TestCode"));
    }

    #[test]
    fn test_last_name_construction() {
        let last_name = LastName::new("Test")
            .with_type("LastNameType")
            .with_name_type("LastName")
            .with_code("TestCode");
        assert_eq!(last_name.value, "Test");
        assert_eq!(last_name.last_name_type.as_deref(), Some("LastNameType"));
        assert_eq!(last_name.name_type.as_deref(), Some("LastName"));
        assert_eq!(last_name.code.as_deref(), Some("TestCode"));
    }

    #[test]
    fn test_person_name_parse() {
        let xml = test_xml_fragment(
            r#"
            <xnl:PersonName xmlns:xnl="urn:oasis:names:tc:ciq:xsdschema:xNL:2.0" Type="TestType" Code="TestCode" NameDetailsKeyRef="TestKeyRef">
                <xnl:NameLine NameType="Initials" Type="InitialsTest" Code="TestCode">J.</xnl:NameLine>
                <xnl:FirstName Type="FirstNameType" NameType="FirstName" Code="TestCode">Jan</xnl:FirstName>
                <xnl:NamePrefix Type="NamePrefixType" NameType="NamePrefix" Code="TestCode">van</xnl:NamePrefix>
                <xnl:LastName Type="LastNameType" NameType="LastName" Code="TestCode">Test</xnl:LastName>
            </xnl:PersonName>
            "#,
        );

        let person_name = PersonName::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(person_name.last_name.value, "Test");
        assert_eq!(person_name.name_line_initials.as_ref().unwrap().value, "J.");
        assert_eq!(person_name.first_name.as_ref().unwrap().value, "Jan");
        assert_eq!(person_name.name_prefix.as_ref().unwrap().value, "van");
        assert_eq!(person_name.person_name_type.as_deref(), Some("TestType"));
        assert_eq!(person_name.code.as_deref(), Some("TestCode"));
        assert_eq!(
            person_name.name_details_key_ref.as_deref(),
            Some("TestKeyRef")
        );
        assert_eq!(
            person_name
                .name_line_initials
                .as_ref()
                .unwrap()
                .name_line_type
                .as_deref(),
            Some("InitialsTest")
        );
        assert_eq!(
            person_name
                .name_line_initials
                .as_ref()
                .unwrap()
                .code
                .as_deref(),
            Some("TestCode")
        );
        assert_eq!(
            person_name
                .first_name
                .as_ref()
                .unwrap()
                .first_name_type
                .as_deref(),
            Some("FirstNameType")
        );
        assert_eq!(
            person_name
                .first_name
                .as_ref()
                .unwrap()
                .name_type
                .as_deref(),
            Some("FirstName")
        );
        assert_eq!(
            person_name.first_name.as_ref().unwrap().code.as_deref(),
            Some("TestCode")
        );
        assert_eq!(
            person_name
                .name_prefix
                .as_ref()
                .unwrap()
                .name_prefix_type
                .as_deref(),
            Some("NamePrefixType")
        );
        assert_eq!(
            person_name
                .name_prefix
                .as_ref()
                .unwrap()
                .name_type
                .as_deref(),
            Some("NamePrefix")
        );
        assert_eq!(
            person_name.name_prefix.as_ref().unwrap().code.as_deref(),
            Some("TestCode")
        );
        assert_eq!(
            person_name.last_name.last_name_type.as_deref(),
            Some("LastNameType")
        );
        assert_eq!(person_name.last_name.name_type.as_deref(), Some("LastName"));
        assert_eq!(person_name.last_name.code.as_deref(), Some("TestCode"));
    }
}
