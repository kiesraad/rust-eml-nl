use crate::{
    EMLError, NS_KR,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName},
};

/// The phase of a count
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// First session ("eerste zitting")
    FirstSession,
    /// Corrigendum ("corrigendum")
    Corrigendum,
}

impl EMLElement for Phase {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Phase", Some(NS_KR));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError>
    where
        Self: Sized,
    {
        todo!()
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        todo!()
    }
}
