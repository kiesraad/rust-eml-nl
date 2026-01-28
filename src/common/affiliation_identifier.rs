use crate::{
    NS_EML,
    io::{EMLElement, collect_struct},
    utils::{AffiliationIdType, StringValue},
};

/// An affiliation identifier consisting of an id and a registered name.
#[derive(Debug, Clone)]
pub struct AffiliationIdentifier {
    /// The affiliation id.
    pub id: Option<StringValue<AffiliationIdType>>,
    /// The registered name of the affiliation.
    pub registered_name: Option<String>,
}

impl AffiliationIdentifier {
    /// Create a new AffiliationIdentifier.
    pub fn new(id: Option<AffiliationIdType>, registered_name: Option<impl Into<String>>) -> Self {
        Self {
            id: id.map(StringValue::Parsed),
            registered_name: registered_name.map(|name| name.into()),
        }
    }
}

impl EMLElement for AffiliationIdentifier {
    const EML_NAME: crate::io::QualifiedName<'_, '_> =
        crate::io::QualifiedName::from_static("AffiliationIdentifier", Some(crate::NS_EML));

    fn read_eml(elem: &mut crate::io::EMLElementReader<'_, '_>) -> Result<Self, crate::EMLError> {
        Ok(collect_struct!(
            elem,
            AffiliationIdentifier {
                id: elem.string_value_attr_opt("Id")?,
                registered_name: ("RegisteredName", NS_EML) => |elem| elem.text_without_children_opt()?,
            }
        ))
    }

    fn write_eml(&self, writer: crate::io::EMLElementWriter) -> Result<(), crate::EMLError> {
        writer
            .attr_opt("Id", self.id.as_ref().map(|id| id.raw()))?
            .child(("RegisteredName", NS_EML), |w| {
                if let Some(name) = &self.registered_name {
                    w.text(name)?.finish()
                } else {
                    w.empty()
                }
            })?
            .finish()?;
        Ok(())
    }
}
