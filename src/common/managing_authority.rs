use crate::{
    EMLError, NS_EML, NS_KR,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName, collect_struct},
    utils::{StringValue, XSBType},
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
    pub id: StringValue<XSBType>,
    /// Name of the managing authority
    pub name: Option<String>,
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
        let id = StringValue::from_maybe_parsed_err(
            elem.attribute_value_req("Id")?.into_owned(),
            elem.strict_value_parsing(),
            "Id",
            Some(elem.span()),
        )?;
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
#[derive(Debug, Clone)]
pub struct AuthorityAddress {}

impl EMLElement for AuthorityAddress {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("AuthorityAddress", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        elem.skip()?;
        Ok(AuthorityAddress {})
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer.finish()?;
        Ok(())
    }
}

/// Address of a managing authority.
#[derive(Debug, Clone)]
pub struct CreatedByAuthority {
    /// Identifier of the managing authority
    pub id: StringValue<XSBType>,
    /// Name of the managing authority
    pub name: Option<String>,
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
        let id = StringValue::from_maybe_parsed_err(
            elem.attribute_value_req("Id")?.into_owned(),
            elem.strict_value_parsing(),
            "Id",
            Some(elem.span()),
        )?;
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
