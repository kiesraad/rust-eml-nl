use crate::{
    NS_KR,
    io::{EMLElement, QualifiedName},
};

/// Election tree as defined in EML_NL.
#[derive(Debug, Clone)]
pub struct ElectionTree {}

impl EMLElement for ElectionTree {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("ElectionTree", Some(NS_KR));

    fn read_eml(elem: &mut crate::io::EMLElementReader<'_, '_>) -> Result<Self, crate::EMLError>
    where
        Self: Sized,
    {
        // TODO: complete election tree parsing
        elem.skip()?;
        Ok(ElectionTree {})
    }

    fn write_eml(&self, writer: crate::io::EMLElementWriter) -> Result<(), crate::EMLError> {
        // TODO: complete election tree writing
        writer.empty()
    }
}
