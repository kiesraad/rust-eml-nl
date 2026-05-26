use instant_xml::{FromXml, ToXml};

use crate::{
    NS_EML, NS_KR,
    utils::{AuthorityId, StringValue},
};

/// Managing authority of an election.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(ns(NS_EML, kr = NS_KR))]
pub struct ManagingAuthority {
    /// Identifier of the managing authority
    #[xml(rename = "AuthorityIdentifier")]
    pub authority_identifier: AuthorityIdentifier,
    /// Address of the managing authority
    #[xml(rename = "AuthorityAddress")]
    pub authority_address: AuthorityAddress,
    /// Instance which created a data set on behalf of another (only if different!)
    #[xml(rename = "CreatedByAuthority")]
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

/// Identifier of a managing authority.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename_all = "PascalCase", ns(NS_EML))]
pub struct AuthorityIdentifier {
    /// Identifier of the managing authority
    #[xml(attribute)]
    pub id: StringValue<AuthorityId>,
    /// Name of the managing authority
    #[xml(direct)]
    pub name: Option<String>,
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
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl From<AuthorityId> for AuthorityIdentifier {
    fn from(value: AuthorityId) -> Self {
        AuthorityIdentifier::new(value)
    }
}

/// Address of a managing authority.
#[derive(Debug, Clone, PartialEq, Eq, FromXml, ToXml)]
#[xml(ns(NS_EML))]
pub struct AuthorityAddress {}

/// Authority that created the authority.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename_all = "PascalCase", ns(NS_KR), force_prefix)]
pub struct CreatedByAuthority {
    /// Identifier of the managing authority
    #[xml(attribute)]
    pub id: StringValue<AuthorityId>,
    /// Name of the managing authority
    #[xml(direct)]
    pub name: Option<String>,
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
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{EMLRead, test_xml_fragment};

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
        let ma = ManagingAuthority::parse_eml(&xml).unwrap();
        assert_eq!(ma.authority_identifier.id.raw(), "1234");
        assert_eq!(ma.authority_identifier.name.as_deref(), Some("Authority 1"));
        assert_eq!(ma.authority_address, AuthorityAddress {});
        let cba = ma.created_by_authority.as_ref().unwrap();
        assert_eq!(cba.id.raw(), "4321");
        assert_eq!(cba.name.as_deref(), Some("Creator Authority"));
    }
}
