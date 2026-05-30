use crate::{
    EMLError, NS_EML, NS_KR,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName, collect_struct},
    utils::{AuthorityId, StringValue},
};

/// Managing authority of an election.
#[derive(Debug, Clone)]
pub struct ManagingAuthority {
    /// Identifier of the managing authority
    pub authority_identifier: AuthorityIdentifier,
    /// Address of the managing authority
    pub authority_address: AuthorityAddress,
    /// Instance which created a data set on behalf of another (only if different!)
    pub created_by_authority: Option<CreatedByAuthority>,
}

impl ManagingAuthority {
    /// Creates a new `ManagingAuthority` with the given identifier and default values for the other fields.
    pub fn new(authority_identifier: impl Into<AuthorityIdentifier>) -> Self {
        ManagingAuthority {
            authority_identifier: authority_identifier.into(),
            authority_address: AuthorityAddress {},
            created_by_authority: None,
        }
    }

    /// Sets the authority that created this authority and returns the modified `ManagingAuthority`.
    pub fn with_created_by_authority(
        mut self,
        created_by_authority: impl Into<CreatedByAuthority>,
    ) -> Self {
        self.created_by_authority = Some(created_by_authority.into());
        self
    }
}

impl From<AuthorityIdentifier> for ManagingAuthority {
    fn from(value: AuthorityIdentifier) -> Self {
        ManagingAuthority::new(value)
    }
}

impl From<AuthorityId> for ManagingAuthority {
    fn from(value: AuthorityId) -> Self {
        ManagingAuthority::new(AuthorityIdentifier::new(value))
    }
}

impl EMLElement for ManagingAuthority {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ManagingAuthority", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, ManagingAuthority {
            authority_identifier: AuthorityIdentifier::EML_NAME => |elem| AuthorityIdentifier::read_eml(elem)?,
            authority_address: AuthorityAddress::EML_NAME => |elem| AuthorityAddress::read_eml(elem)?,
            created_by_authority as Option: CreatedByAuthority::EML_NAME => |elem| CreatedByAuthority::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(AuthorityIdentifier::EML_NAME, &self.authority_identifier)?
            .child_elem(AuthorityAddress::EML_NAME, &self.authority_address)?
            .child_elem_option(
                CreatedByAuthority::EML_NAME,
                self.created_by_authority.as_ref(),
            )?
            .finish()?;

        Ok(())
    }
}

/// Identifier of a managing authority.
#[derive(Debug, Clone)]
pub struct AuthorityIdentifier {
    /// Identifier of the managing authority
    pub id: StringValue<AuthorityId>,
    /// Name of the managing authority
    pub name: Option<Box<str>>,
}

impl AuthorityIdentifier {
    /// Creates a new `AuthorityIdentifier` with the given ID and no name.
    pub fn new(id: AuthorityId) -> Self {
        AuthorityIdentifier {
            id: StringValue::from_value(id),
            name: None,
        }
    }

    /// Sets the name of the managing authority and returns the modified `AuthorityIdentifier`.
    pub fn with_name(mut self, name: impl Into<Box<str>>) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl From<AuthorityId> for AuthorityIdentifier {
    fn from(value: AuthorityId) -> Self {
        AuthorityIdentifier::new(value)
    }
}

impl EMLElement for AuthorityIdentifier {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("AuthorityIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let name = if elem.is_empty() {
            None
        } else {
            Some(elem.text_without_children()?)
        };
        let id = elem.string_value_attr("Id", None)?;
        Ok(AuthorityIdentifier { id, name })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        let writer = writer.attr("Id", self.id.raw().as_ref())?;
        if let Some(name) = &self.name {
            writer.text(name.as_ref())?.finish()?;
        } else {
            writer.empty()?;
        }
        Ok(())
    }
}

/// Address of a managing authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorityAddress {}

impl EMLElement for AuthorityAddress {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("AuthorityAddress", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, AuthorityAddress {}))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer.empty()
    }
}

/// Authority that created the authority.
#[derive(Debug, Clone)]
pub struct CreatedByAuthority {
    /// Identifier of the managing authority
    pub id: StringValue<AuthorityId>,
    /// Name of the managing authority
    pub name: Option<Box<str>>,
}

impl CreatedByAuthority {
    /// Creates a new `CreatedByAuthority` with the given ID and no name.
    pub fn new(id: AuthorityId) -> Self {
        CreatedByAuthority {
            id: StringValue::from_value(id),
            name: None,
        }
    }

    /// Sets the name of the managing authority and returns the modified `CreatedByAuthority`.
    pub fn with_name(mut self, name: impl Into<Box<str>>) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl EMLElement for CreatedByAuthority {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("CreatedByAuthority", Some(NS_KR));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let name = if elem.is_empty() {
            None
        } else {
            Some(elem.text_without_children()?)
        };

        let id = elem.string_value_attr("Id", None)?;
        Ok(CreatedByAuthority { id, name })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        let writer = writer.attr("Id", self.id.raw().as_ref())?;
        if let Some(name) = &self.name {
            writer.text(name.as_ref())?.finish()?;
        } else {
            writer.empty()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{EMLParsingMode, EMLRead, test_write_eml_element, test_xml_fragment};

    #[test]
    fn test_managing_authority_construction() {
        let id =
            AuthorityIdentifier::new(AuthorityId::new("1234").unwrap()).with_name("Authority 1");
        let created_by = CreatedByAuthority::new(AuthorityId::new("4321").unwrap())
            .with_name("Creator Authority");
        let m = ManagingAuthority::new(id).with_created_by_authority(created_by);
        assert_eq!(m.authority_identifier.id.raw(), "1234");
        assert_eq!(m.authority_identifier.name.as_deref(), Some("Authority 1"));
        assert_eq!(m.authority_address, AuthorityAddress {});
        let cba = m.created_by_authority.as_ref().unwrap();
        assert_eq!(cba.id.raw(), "4321");
        assert_eq!(cba.name.as_deref(), Some("Creator Authority"));
    }

    #[test]
    fn test_managing_authority_parsing() {
        let xml = test_xml_fragment(
            r#"
            <ManagingAuthority xmlns="urn:oasis:names:tc:evs:schema:eml" xmlns:kr="http://www.kiesraad.nl/extensions">
                <AuthorityIdentifier Id="1234">Authority 1</AuthorityIdentifier>
                <AuthorityAddress/>
                <kr:CreatedByAuthority Id="4321">Creator Authority</kr:CreatedByAuthority>
            </ManagingAuthority>
            "#,
        );
        let ma = ManagingAuthority::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(ma.authority_identifier.id.raw(), "1234");
        assert_eq!(ma.authority_identifier.name.as_deref(), Some("Authority 1"));
        assert_eq!(ma.authority_address, AuthorityAddress {});
        let cba = ma.created_by_authority.as_ref().unwrap();
        assert_eq!(cba.id.raw(), "4321");
        assert_eq!(cba.name.as_deref(), Some("Creator Authority"));

        let xml_output = test_write_eml_element(&ma, &[NS_EML, NS_KR]).unwrap();
        assert_eq!(xml_output, xml);
    }
}
