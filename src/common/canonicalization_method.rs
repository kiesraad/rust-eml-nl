use crate::{
    NS_DS,
    io::{EMLElement, collect_struct},
};

/// XML CanonicalizationMethod element
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalizationMethod {
    algorithm: String,
}

impl EMLElement for CanonicalizationMethod {
    const EML_NAME: crate::io::QualifiedName<'_, '_> =
        crate::io::QualifiedName::from_static("CanonicalizationMethod", Some(NS_DS));

    fn read_eml(elem: &mut crate::io::EMLElementReader<'_, '_>) -> Result<Self, crate::EMLError> {
        Ok(collect_struct!(
            elem,
            CanonicalizationMethod {
                algorithm: elem.attribute_value_req("Algorithm")?.into_owned(),
            }
        ))
    }

    fn write_eml(&self, writer: crate::io::EMLElementWriter) -> Result<(), crate::EMLError> {
        writer.attr("Algorithm", &self.algorithm)?.empty()?;

        Ok(())
    }
}
