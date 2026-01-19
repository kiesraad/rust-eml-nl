use crate::{
    io::EMLElement,
    utils::{ElectionDomainIdType, StringValue},
};

/// Election domain of an election.
///
/// The (top level) region where the election takes place. Only needed if the
/// ElectionDomain is part of the election name, e.g. election of the council of
/// a municipality or province. Not needed e.g. for Tweede Kamer or European
/// Parliament.
#[derive(Debug, Clone)]
pub struct ElectionDomain {
    /// Identifier of the election domain
    pub id: StringValue<ElectionDomainIdType>,
    /// Name of the election domain
    pub name: String,
}

impl ElectionDomain {
    /// Create a new ElectionDomain
    pub fn new(id: ElectionDomainIdType, name: String) -> Self {
        ElectionDomain {
            id: StringValue::from_value(id),
            name,
        }
    }
}

impl EMLElement for ElectionDomain {
    const EML_NAME: crate::io::QualifiedName<'_, '_> =
        crate::io::QualifiedName::from_static("ElectionDomain", Some(crate::NS_KR));

    fn read_eml(elem: &mut crate::io::EMLElementReader<'_, '_>) -> Result<Self, crate::EMLError> {
        let id = elem.string_value_attr("Id", None)?;
        let name = elem.text_without_children()?;

        Ok(ElectionDomain { id, name })
    }

    fn write_eml(&self, writer: crate::io::EMLElementWriter) -> Result<(), crate::EMLError> {
        writer
            .attr("Id", self.id.raw().as_ref())?
            .text(self.name.as_ref())?
            .finish()
    }
}
