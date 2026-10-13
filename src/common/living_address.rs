use crate::{
    EMLError, NS_KR,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName, collect_struct},
};

/// A living address (kr:LivingAddress).
#[derive(Debug, Clone)]
pub struct LivingAddress {
    /// The locality name.
    pub locality_name: Box<str>,

    /// The country name code, if present.
    pub country_name_code: Option<Box<str>>,
}

impl LivingAddress {
    /// Create a new `LivingAddress`.
    pub fn new(locality_name: impl Into<Box<str>>) -> Self {
        LivingAddress {
            locality_name: locality_name.into(),
            country_name_code: None,
        }
    }

    /// Set the country name code.
    pub fn with_country_name_code(mut self, code: impl Into<Box<str>>) -> Self {
        self.country_name_code = Some(code.into());
        self
    }
}

impl EMLElement for LivingAddress {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("LivingAddress", Some(NS_KR));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, LivingAddress {
            locality_name: ("LocalityName", NS_KR) => |elem| elem.text_without_children()?,
            country_name_code as Option: ("CountryNameCode", NS_KR) => |elem| elem.text_without_children()?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child(("LocalityName", NS_KR), |elem| {
                elem.text(self.locality_name.as_ref())?.finish()
            })?
            .child_option(
                ("CountryNameCode", NS_KR),
                self.country_name_code.as_ref(),
                |elem, value| elem.text(value.as_ref())?.finish(),
            )?
            .finish()
    }
}
