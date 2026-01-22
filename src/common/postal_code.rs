use crate::{
    NS_XAL,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName, collect_struct},
};

/// Postal code element
#[derive(Debug, Clone)]
pub struct PostalCode {
    /// Postal code number
    pub number: PostalCodeNumber,
}

impl EMLElement for PostalCode {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("PostalCode", Some(NS_XAL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, crate::EMLError> {
        Ok(collect_struct!(elem, PostalCode {
            number: PostalCodeNumber::EML_NAME => |elem| PostalCodeNumber::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), crate::EMLError> {
        writer
            .child_elem(PostalCodeNumber::EML_NAME, &self.number)?
            .finish()
    }
}

/// Postal code number element
#[derive(Debug, Clone)]
pub struct PostalCodeNumber {
    /// Type attribute of the postal code number
    pub number_type: Option<String>,
    /// Code attribute of the postal code number
    pub code: Option<String>,
    /// The postal code value
    pub number: Option<String>,
}

impl EMLElement for PostalCodeNumber {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("PostalCodeNumber", Some(NS_XAL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, crate::EMLError> {
        let number_type = elem.attribute_value("Type")?.map(|s| s.into_owned());
        let code = elem.attribute_value("Code")?.map(|s| s.into_owned());
        let number = elem.text_without_children_opt()?;
        Ok(PostalCodeNumber {
            number_type,
            code,
            number,
        })
    }

    fn write_eml(&self, mut writer: EMLElementWriter) -> Result<(), crate::EMLError> {
        if let Some(number_type) = &self.number_type {
            writer = writer.attr("Type", number_type.as_ref())?
        }
        if let Some(code) = &self.code {
            writer = writer.attr("Code", code.as_ref())?
        }
        if let Some(number) = &self.number {
            writer.text(number.as_ref())?.finish()
        } else {
            writer.empty()
        }
    }
}
