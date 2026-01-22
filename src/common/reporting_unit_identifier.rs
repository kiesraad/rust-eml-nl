use crate::{
    io::EMLElement,
    utils::{ReportingUnitIdentifierId, StringValue},
};

/// Identifier for the reporting unit.
#[derive(Debug, Clone)]
pub struct ReportingUnitIdentifier {
    /// Id of the reporting unit.
    pub id: StringValue<ReportingUnitIdentifierId>,
    /// Name of the reporting unit.
    pub name: String,
}

impl EMLElement for ReportingUnitIdentifier {
    const EML_NAME: crate::io::QualifiedName<'_, '_> =
        crate::io::QualifiedName::from_static("ReportingUnitIdentifier", Some(crate::NS_EML));

    fn read_eml(elem: &mut crate::io::EMLElementReader<'_, '_>) -> Result<Self, crate::EMLError> {
        let name = elem.text_without_children()?;
        let id = elem.string_value_attr("Id", None)?;
        Ok(ReportingUnitIdentifier { id, name })
    }

    fn write_eml(&self, writer: crate::io::EMLElementWriter) -> Result<(), crate::EMLError> {
        writer
            .attr("Id", self.id.raw().as_ref())?
            .text(self.name.as_ref())?
            .finish()
    }
}
