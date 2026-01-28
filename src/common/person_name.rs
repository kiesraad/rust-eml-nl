use thiserror::Error;

use crate::{
    EMLError, NS_XNL,
    io::{EMLElement, EMLReadElement, EMLWriteElement, QualifiedName, collect_struct},
};

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

impl EMLReadElement for PersonNameStructure {
    fn read_eml_element<'a, 'b>(
        elem: &mut crate::io::EMLElementReader<'a, 'b>,
    ) -> Result<Self, EMLError> {
        Ok(collect_struct!(
            elem,
            PersonNameStructure {
                person_name: PersonName::EML_NAME => |elem| PersonName::read_eml(elem)?,
                party_type: elem.attribute_value("PartyType")?.map(|s| s.into_owned()),
                code: elem.attribute_value("Code")?.map(|s| s.into_owned()),
            }
        ))
    }
}

impl EMLWriteElement for PersonNameStructure {
    fn write_eml_element(&self, writer: crate::io::EMLElementWriter) -> Result<(), EMLError> {
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
    pub person_name_type: Option<String>,
    /// The Code attribute of the PersonName
    pub code: Option<String>,
    /// The NameDetailsKeyRef attribute of the PersonName
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
    pub fn with_initials(mut self, initials: impl Into<String>) -> Self {
        self.name_line_initials = Some(NameLineInitials::new(initials));
        self
    }

    /// Set the first name of the person.
    pub fn with_first_name(mut self, first_name: impl Into<String>) -> Self {
        self.first_name = Some(FirstName::new(first_name));
        self
    }

    /// Set the prefix of the person's last name.
    pub fn with_name_prefix(mut self, name_prefix: impl Into<String>) -> Self {
        self.name_prefix = Some(NamePrefix::new(name_prefix));
        self
    }
}

impl EMLElement for PersonName {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("PersonName", Some(NS_XNL));

    fn read_eml(elem: &mut crate::io::EMLElementReader<'_, '_>) -> Result<Self, crate::EMLError> {
        Ok(collect_struct!(
            elem,
            PersonName {
                name_line_initials as Option: NameLineInitials::EML_NAME => |elem| {
                    NameLineInitials::read_eml(elem)?
                },
                first_name as Option: FirstName::EML_NAME => |elem| FirstName::read_eml(elem)?,
                name_prefix as Option: NamePrefix::EML_NAME => |elem| NamePrefix::read_eml(elem)?,
                last_name: LastName::EML_NAME => |elem| LastName::read_eml(elem)?,
                person_name_type: elem.attribute_value("Type")?.map(|s| s.into_owned()),
                code: elem.attribute_value("Code")?.map(|s| s.into_owned()),
                name_details_key_ref: elem
                    .attribute_value("NameDetailsKeyRef")?
                    .map(|s| s.into_owned()),
            }
        ))
    }

    fn write_eml(&self, writer: crate::io::EMLElementWriter) -> Result<(), EMLError> {
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
    pub value: String,
    /// The Type attribute of the NameLineInitials
    pub name_line_type: Option<String>,
    /// The Code attribute of the NameLineInitials
    pub code: Option<String>,
}

impl NameLineInitials {
    /// Create a new NameLineInitials.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            name_line_type: None,
            code: None,
        }
    }
}

/// Error indicating that the NameType attribute is not "Initials".
#[derive(Debug, Clone, Error)]
#[error("NameType attribute is not 'Initials'")]
struct NameTypeInitialsError;

impl EMLElement for NameLineInitials {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("NameLine", Some(NS_XNL));

    fn read_eml(elem: &mut crate::io::EMLElementReader<'_, '_>) -> Result<Self, crate::EMLError> {
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
            name_line_type: elem.attribute_value("Type")?.map(|s| s.into_owned()),
            code: elem.attribute_value("Code")?.map(|s| s.into_owned()),
        })
    }

    fn write_eml(&self, writer: crate::io::EMLElementWriter) -> Result<(), crate::EMLError> {
        writer
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
    pub value: String,
    /// The Type attribute of the FirstName
    pub first_name_type: Option<String>,
    /// The NameType attribute of the name
    pub name_type: Option<String>,
    /// The Code attribute of the FirstName
    pub code: Option<String>,
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
}

impl EMLElement for FirstName {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("FirstName", Some(NS_XNL));

    fn read_eml(elem: &mut crate::io::EMLElementReader<'_, '_>) -> Result<Self, crate::EMLError> {
        Ok(FirstName {
            value: elem.text_without_children()?,
            first_name_type: elem.attribute_value("Type")?.map(|s| s.into_owned()),
            name_type: elem.attribute_value("NameType")?.map(|s| s.into_owned()),
            code: elem.attribute_value("Code")?.map(|s| s.into_owned()),
        })
    }

    fn write_eml(&self, writer: crate::io::EMLElementWriter) -> Result<(), crate::EMLError> {
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
    pub value: String,
    /// The Type attribute of the NamePrefix
    pub name_prefix_type: Option<String>,
    /// The NameType attribute of the NamePrefix
    pub name_type: Option<String>,
    /// The Code attribute of the NamePrefix
    pub code: Option<String>,
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
}

impl EMLElement for NamePrefix {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("NamePrefix", Some(NS_XNL));

    fn read_eml(elem: &mut crate::io::EMLElementReader<'_, '_>) -> Result<Self, crate::EMLError> {
        Ok(NamePrefix {
            value: elem.text_without_children()?,
            name_prefix_type: elem.attribute_value("Type")?.map(|s| s.into_owned()),
            name_type: elem.attribute_value("NameType")?.map(|s| s.into_owned()),
            code: elem.attribute_value("Code")?.map(|s| s.into_owned()),
        })
    }

    fn write_eml(&self, writer: crate::io::EMLElementWriter) -> Result<(), crate::EMLError> {
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
    pub value: String,
    /// The Type attribute of the LastName
    pub last_name_type: Option<String>,
    /// The NameType attribute of the LastName
    pub name_type: Option<String>,
    /// The Code attribute of the LastName
    pub code: Option<String>,
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
}

impl EMLElement for LastName {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("LastName", Some(NS_XNL));

    fn read_eml(elem: &mut crate::io::EMLElementReader<'_, '_>) -> Result<Self, crate::EMLError> {
        Ok(LastName {
            value: elem.text_without_children()?,
            last_name_type: elem.attribute_value("Type")?.map(|s| s.into_owned()),
            name_type: elem.attribute_value("NameType")?.map(|s| s.into_owned()),
            code: elem.attribute_value("Code")?.map(|s| s.into_owned()),
        })
    }

    fn write_eml(&self, writer: crate::io::EMLElementWriter) -> Result<(), crate::EMLError> {
        writer
            .attr_opt("Type", self.last_name_type.as_ref())?
            .attr_opt("NameType", self.name_type.as_ref())?
            .attr_opt("Code", self.code.as_ref())?
            .text(&self.value)?
            .finish()?;
        Ok(())
    }
}
