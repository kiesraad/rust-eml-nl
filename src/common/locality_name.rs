use crate::{
    NS_XAL,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName},
};

/// Name of a locality
#[derive(Debug, Clone)]
pub struct LocalityName {
    /// Name of the locality
    pub name: String,
    /// Type of the locality, if any
    pub locality_type: Option<String>,
    /// Associated code for the locality, if any
    pub code: Option<String>,
}

impl EMLElement for LocalityName {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("LocalityName", Some(NS_XAL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, crate::EMLError> {
        Ok(LocalityName {
            name: elem.text_without_children()?,
            locality_type: elem.attribute_value("Type")?.map(|s| s.into_owned()),
            code: elem.attribute_value("Code")?.map(|s| s.into_owned()),
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), crate::EMLError> {
        let mut writer = writer;
        if let Some(ref locality_type) = self.locality_type {
            writer = writer.attr("Type", locality_type)?;
        }
        if let Some(ref code) = self.code {
            writer = writer.attr("Code", code)?;
        }
        writer.text(&self.name)?.finish()
    }
}
