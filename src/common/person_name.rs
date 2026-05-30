use thiserror::Error;

use crate::{
    EMLError, NS_XNL,
    io::{
        EMLElement, EMLElementReader, EMLElementWriter, EMLReadElement, EMLWriteElement,
        QualifiedName, collect_struct,
    },
};

/// Container for details of the name of a person.
#[derive(Debug, Clone)]
pub struct PersonNameStructure {
    /// The person's name details.
    pub person_name: PersonName,

    /// The PartyType attribute of the PersonNameStructure
    pub party_type: Option<Box<str>>,

    /// The Code attribute of the PersonNameStructure
    pub code: Option<Box<str>>,
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
    pub fn with_party_type(mut self, party_type: impl Into<Box<str>>) -> Self {
        self.party_type = Some(party_type.into());
        self
    }

    /// Set the `Code` attribute of the `PersonNameStructure`.
    pub fn with_code(mut self, code: impl Into<Box<str>>) -> Self {
        self.code = Some(code.into());
        self
    }
}

impl From<PersonName> for PersonNameStructure {
    fn from(value: PersonName) -> Self {
        PersonNameStructure::new(value)
    }
}

impl EMLReadElement for PersonNameStructure {
    fn read_eml_element(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(
            elem,
            PersonNameStructure {
                person_name: PersonName::EML_NAME => |elem| PersonName::read_eml(elem)?,
                party_type: elem.attribute_value("PartyType")?.map(|s| s.into()),
                code: elem.attribute_value("Code")?.map(|s| s.into()),
            }
        ))
    }
}

impl EMLWriteElement for PersonNameStructure {
    fn write_eml_element(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr_opt("PartyType", self.party_type.as_ref())?
            .attr_opt("Code", self.code.as_ref())?
            .child_elem(PersonName::EML_NAME, &self.person_name)?
            .finish()
    }
}

/// Details of the name of a person.
#[derive(Debug, Clone)]
pub struct PersonName {
    /// The initials of the person.
    pub name_line_initials: Option<NameLineInitials>,

    /// The first name of the person.
    pub first_name: Option<FirstName>,

    /// The prefix of the person's last name.
    pub name_prefix: Option<NamePrefix>,

    /// The last name of the person.
    pub last_name: LastName,

    /// The Type attribute of the PersonName
    pub person_name_type: Option<Box<str>>,

    /// The Code attribute of the PersonName
    pub code: Option<Box<str>>,

    /// The NameDetailsKeyRef attribute of the PersonName
    pub name_details_key_ref: Option<Box<str>>,
}

impl PersonName {
    /// Create a new PersonName.
    pub fn new(last_name: impl Into<Box<str>>) -> Self {
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
    pub fn with_initials(self, initials: impl Into<Box<str>>) -> Self {
        self.with_initials_option(Some(initials))
    }

    /// Set the initials of the person, if present.
    pub fn with_initials_option(mut self, initials: Option<impl Into<Box<str>>>) -> Self {
        self.name_line_initials = initials.map(|i| NameLineInitials::new(i));
        self
    }

    /// Set the first name of the person.
    pub fn with_first_name(self, first_name: impl Into<Box<str>>) -> Self {
        self.with_first_name_option(Some(first_name))
    }

    /// Set the first name of the person, if present.
    pub fn with_first_name_option(mut self, first_name: Option<impl Into<Box<str>>>) -> Self {
        self.first_name = first_name.map(|f| FirstName::new(f));
        self
    }

    /// Set the prefix of the person's last name.
    pub fn with_name_prefix(self, name_prefix: impl Into<Box<str>>) -> Self {
        self.with_name_prefix_option(Some(name_prefix))
    }

    /// Set the prefix of the person's last name, if present.
    pub fn with_name_prefix_option(mut self, name_prefix: Option<impl Into<Box<str>>>) -> Self {
        self.name_prefix = name_prefix.map(|p| NamePrefix::new(p));
        self
    }

    /// Set the `Type` attribute of the PersonName.
    pub fn with_type(mut self, person_name_type: impl Into<Box<str>>) -> Self {
        self.person_name_type = Some(person_name_type.into());
        self
    }

    /// Set the `Code` attribute of the PersonName.
    pub fn with_code(mut self, code: impl Into<Box<str>>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Set the `NameDetailsKeyRef` attribute of the PersonName.
    pub fn with_name_details_key_ref(mut self, name_details_key_ref: impl Into<Box<str>>) -> Self {
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
            .map(|initials| initials.value.as_ref())
    }

    /// Get the first name of the person, if present.
    pub fn get_first_name(&self) -> Option<&str> {
        self.first_name
            .as_ref()
            .map(|first_name| first_name.value.as_ref())
    }

    /// Get the prefix of the person's last name, if present.
    pub fn get_name_prefix(&self) -> Option<&str> {
        self.name_prefix
            .as_ref()
            .map(|prefix| prefix.value.as_ref())
    }
}

impl EMLElement for PersonName {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("PersonName", Some(NS_XNL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(
            elem,
            PersonName {
                name_line_initials as Option: NameLineInitials::EML_NAME => |elem| {
                    NameLineInitials::read_eml(elem)?
                },
                first_name as Option: FirstName::EML_NAME => |elem| FirstName::read_eml(elem)?,
                name_prefix as Option: NamePrefix::EML_NAME => |elem| NamePrefix::read_eml(elem)?,
                last_name: LastName::EML_NAME => |elem| LastName::read_eml(elem)?,
                person_name_type: elem.attribute_value("Type")?.map(|s| s.into()),
                code: elem.attribute_value("Code")?.map(|s| s.into()),
                name_details_key_ref: elem
                    .attribute_value("NameDetailsKeyRef")?
                    .map(|s| s.into()),
            }
        ))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr_opt("Type", self.person_name_type.as_ref())?
            .attr_opt("Code", self.code.as_ref())?
            .attr_opt("NameDetailsKeyRef", self.name_details_key_ref.as_ref())?
            .child_elem_option(NameLineInitials::EML_NAME, self.name_line_initials.as_ref())?
            .child_elem_option(FirstName::EML_NAME, self.first_name.as_ref())?
            .child_elem_option(NamePrefix::EML_NAME, self.name_prefix.as_ref())?
            .child_elem(LastName::EML_NAME, &self.last_name)?
            .finish()
    }
}

/// Details of the initials line of a person's name.
#[derive(Debug, Clone)]
pub struct NameLineInitials {
    /// The initials value.
    pub value: Box<str>,

    /// The Type attribute of the NameLineInitials
    pub name_line_type: Option<Box<str>>,

    /// The Code attribute of the NameLineInitials
    pub code: Option<Box<str>>,
}

impl NameLineInitials {
    /// Create a new NameLineInitials.
    pub fn new(value: impl Into<Box<str>>) -> Self {
        Self {
            value: value.into(),
            name_line_type: None,
            code: None,
        }
    }

    /// Set the `Type` attribute of the element.
    pub fn with_type(mut self, name_line_type: impl Into<Box<str>>) -> Self {
        self.name_line_type = Some(name_line_type.into());
        self
    }

    /// Set the `Code` attribute of the element.
    pub fn with_code(mut self, code: impl Into<Box<str>>) -> Self {
        self.code = Some(code.into());
        self
    }
}

/// Error indicating that the NameType attribute is not "Initials".
#[derive(Debug, Clone, Error)]
#[error("NameType attribute is not 'Initials'")]
struct NameTypeInitialsError;

impl EMLElement for NameLineInitials {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("NameLine", Some(NS_XNL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let name_type = elem.attribute_value_req("NameType")?;
        if name_type.as_ref() != "Initials" {
            let err = EMLError::invalid_value(
                elem.name()?.as_owned(),
                NameTypeInitialsError,
                Some(elem.span()),
            );
            if elem.parsing_mode().is_strict() {
                return Err(err);
            } else {
                elem.push_err(err);
            }
        }

        Ok(NameLineInitials {
            value: elem.text_without_children()?,
            name_line_type: elem.attribute_value("Type")?.map(|s| s.into()),
            code: elem.attribute_value("Code")?.map(|s| s.into()),
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr("NameType", "Initials")?
            .attr_opt("Type", self.name_line_type.as_ref())?
            .attr_opt("Code", self.code.as_ref())?
            .text(&self.value)?
            .finish()?;
        Ok(())
    }
}

/// Details of the first name of a person.
#[derive(Debug, Clone)]
pub struct FirstName {
    /// The first name value.
    pub value: Box<str>,

    /// The Type attribute of the FirstName
    pub first_name_type: Option<Box<str>>,

    /// The NameType attribute of the name
    pub name_type: Option<Box<str>>,

    /// The Code attribute of the FirstName
    pub code: Option<Box<str>>,
}

impl FirstName {
    /// Create a new FirstName.
    pub fn new(value: impl Into<Box<str>>) -> Self {
        Self {
            value: value.into(),
            first_name_type: None,
            name_type: None,
            code: None,
        }
    }

    /// Set the `Type` attribute of the element.
    pub fn with_type(mut self, first_name_type: impl Into<Box<str>>) -> Self {
        self.first_name_type = Some(first_name_type.into());
        self
    }

    /// Set the `NameType` attribute of the element.
    pub fn with_name_type(mut self, name_type: impl Into<Box<str>>) -> Self {
        self.name_type = Some(name_type.into());
        self
    }

    /// Set the `Code` attribute of the element.
    pub fn with_code(mut self, code: impl Into<Box<str>>) -> Self {
        self.code = Some(code.into());
        self
    }
}

impl EMLElement for FirstName {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("FirstName", Some(NS_XNL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(FirstName {
            value: elem.text_without_children()?,
            first_name_type: elem.attribute_value("Type")?.map(|s| s.into()),
            name_type: elem.attribute_value("NameType")?.map(|s| s.into()),
            code: elem.attribute_value("Code")?.map(|s| s.into()),
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr_opt("Type", self.first_name_type.as_ref())?
            .attr_opt("NameType", self.name_type.as_ref())?
            .attr_opt("Code", self.code.as_ref())?
            .text(&self.value)?
            .finish()?;
        Ok(())
    }
}

/// Details of the prefix of a person's last name.
#[derive(Debug, Clone)]
pub struct NamePrefix {
    /// The prefix value.
    pub value: Box<str>,

    /// The Type attribute of the NamePrefix
    pub name_prefix_type: Option<Box<str>>,

    /// The NameType attribute of the NamePrefix
    pub name_type: Option<Box<str>>,

    /// The Code attribute of the NamePrefix
    pub code: Option<Box<str>>,
}

impl NamePrefix {
    /// Create a new NamePrefix.
    pub fn new(value: impl Into<Box<str>>) -> Self {
        Self {
            value: value.into(),
            name_prefix_type: None,
            name_type: None,
            code: None,
        }
    }

    /// Set the `Type` attribute of the element.
    pub fn with_type(mut self, name_prefix_type: impl Into<Box<str>>) -> Self {
        self.name_prefix_type = Some(name_prefix_type.into());
        self
    }

    /// Set the `NameType` attribute of the element.
    pub fn with_name_type(mut self, name_type: impl Into<Box<str>>) -> Self {
        self.name_type = Some(name_type.into());
        self
    }

    /// Set the `Code` attribute of the element.
    pub fn with_code(mut self, code: impl Into<Box<str>>) -> Self {
        self.code = Some(code.into());
        self
    }
}

impl EMLElement for NamePrefix {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("NamePrefix", Some(NS_XNL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(NamePrefix {
            value: elem.text_without_children()?,
            name_prefix_type: elem.attribute_value("Type")?.map(|s| s.into()),
            name_type: elem.attribute_value("NameType")?.map(|s| s.into()),
            code: elem.attribute_value("Code")?.map(|s| s.into()),
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr_opt("Type", self.name_prefix_type.as_ref())?
            .attr_opt("NameType", self.name_type.as_ref())?
            .attr_opt("Code", self.code.as_ref())?
            .text(&self.value)?
            .finish()?;
        Ok(())
    }
}

/// Details of the last name of a person.
#[derive(Debug, Clone)]
pub struct LastName {
    /// The last name value.
    pub value: Box<str>,

    /// The Type attribute of the LastName
    pub last_name_type: Option<Box<str>>,

    /// The NameType attribute of the LastName
    pub name_type: Option<Box<str>>,

    /// The Code attribute of the LastName
    pub code: Option<Box<str>>,
}

impl LastName {
    /// Create a new LastName.
    pub fn new(value: impl Into<Box<str>>) -> Self {
        Self {
            value: value.into(),
            last_name_type: None,
            name_type: None,
            code: None,
        }
    }

    /// Set the `Type` attribute of the element.
    pub fn with_type(mut self, last_name_type: impl Into<Box<str>>) -> Self {
        self.last_name_type = Some(last_name_type.into());
        self
    }

    /// Set the `NameType` attribute of the element.
    pub fn with_name_type(mut self, name_type: impl Into<Box<str>>) -> Self {
        self.name_type = Some(name_type.into());
        self
    }

    /// Set the `Code` attribute of the element.
    pub fn with_code(mut self, code: impl Into<Box<str>>) -> Self {
        self.code = Some(code.into());
        self
    }
}

impl EMLElement for LastName {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("LastName", Some(NS_XNL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(LastName {
            value: elem.text_without_children()?,
            last_name_type: elem.attribute_value("Type")?.map(|s| s.into()),
            name_type: elem.attribute_value("NameType")?.map(|s| s.into()),
            code: elem.attribute_value("Code")?.map(|s| s.into()),
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr_opt("Type", self.last_name_type.as_ref())?
            .attr_opt("NameType", self.name_type.as_ref())?
            .attr_opt("Code", self.code.as_ref())?
            .text(&self.value)?
            .finish()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{EMLParsingMode, EMLRead as _, test_write_eml_element, test_xml_fragment};

    #[test]
    fn test_person_name_construction() {
        let person_name = PersonName::new("Test")
            .with_initials("J.")
            .with_first_name("Jan")
            .with_name_prefix("van")
            .with_type("TestType")
            .with_code("TestCode")
            .with_name_details_key_ref("TestKeyRef");

        assert_eq!(person_name.last_name.value.as_ref(), "Test");
        assert_eq!(
            person_name
                .name_line_initials
                .as_ref()
                .unwrap()
                .value
                .as_ref(),
            "J."
        );
        assert_eq!(
            person_name.first_name.as_ref().unwrap().value.as_ref(),
            "Jan"
        );
        assert_eq!(
            person_name.name_prefix.as_ref().unwrap().value.as_ref(),
            "van"
        );
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

        assert_eq!(
            person_name_structure.person_name.last_name.value.as_ref(),
            "Test"
        );
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

        assert_eq!(
            person_name_structure.person_name.last_name.value.as_ref(),
            "Test"
        );
        assert!(person_name_structure.party_type.is_none());
        assert!(person_name_structure.code.is_none());
    }

    #[test]
    fn test_name_line_initials_construction() {
        let initials = NameLineInitials::new("J.")
            .with_type("Initials")
            .with_code("TestCode");
        assert_eq!(initials.value.as_ref(), "J.");
        assert_eq!(initials.name_line_type.as_deref(), Some("Initials"));
        assert_eq!(initials.code.as_deref(), Some("TestCode"));
    }

    #[test]
    fn test_first_name_construction() {
        let first_name = FirstName::new("Jan")
            .with_type("FirstNameType")
            .with_name_type("FirstName")
            .with_code("TestCode");
        assert_eq!(first_name.value.as_ref(), "Jan");
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
        assert_eq!(name_prefix.value.as_ref(), "van");
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
        assert_eq!(last_name.value.as_ref(), "Test");
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
        assert_eq!(person_name.last_name.value.as_ref(), "Test");
        assert_eq!(
            person_name
                .name_line_initials
                .as_ref()
                .unwrap()
                .value
                .as_ref(),
            "J."
        );
        assert_eq!(
            person_name.first_name.as_ref().unwrap().value.as_ref(),
            "Jan"
        );
        assert_eq!(
            person_name.name_prefix.as_ref().unwrap().value.as_ref(),
            "van"
        );
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

        let xml_output = test_write_eml_element(&person_name, &[NS_XNL]).unwrap();
        assert_eq!(xml_output, xml);
    }
}
