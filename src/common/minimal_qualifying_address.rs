use crate::{
    EMLError, EMLErrorKind, NS_EML, NS_XAL,
    common::{CountryNameCode, LocalityName},
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName, collect_struct},
};

/// The minimal details for a qualifying address
#[derive(Debug, Clone)]
pub enum MinimalQualifyingAddress {
    /// This qualifying address is a locality
    Locality(MinimalQualifyingAddressLocality),

    /// This qualifying address is a country
    Country(MinimalQualifyingAddressCountry),
}

impl MinimalQualifyingAddress {
    /// Creates a new `MinimalQualifyingAddress` as a locality.
    pub fn new_locality(locality_name: impl Into<LocalityName>) -> Self {
        MinimalQualifyingAddress::Locality(MinimalQualifyingAddressLocality::new(
            locality_name.into(),
        ))
    }

    /// Creates a new `MinimalQualifyingAddress` as a country.
    pub fn new_country(
        country_name_code: impl Into<CountryNameCode>,
        locality_name: impl Into<LocalityName>,
    ) -> Self {
        MinimalQualifyingAddress::Country(MinimalQualifyingAddressCountry::new(
            country_name_code.into(),
            MinimalQualifyingAddressLocality::new(locality_name.into()),
        ))
    }
}

impl EMLElement for MinimalQualifyingAddress {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("QualifyingAddress", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let parent_name = elem.name()?.as_owned();
        let mut found_child = None;
        while let Some(mut child) = elem.next_child()? {
            match child.name()? {
                n if n == MinimalQualifyingAddressLocality::EML_NAME && found_child.is_none() => {
                    found_child = Some(MinimalQualifyingAddress::Locality(
                        MinimalQualifyingAddressLocality::read_eml(&mut child)?,
                    ));
                }
                n if n == MinimalQualifyingAddressCountry::EML_NAME && found_child.is_none() => {
                    found_child = Some(MinimalQualifyingAddress::Country(
                        MinimalQualifyingAddressCountry::read_eml(&mut child)?,
                    ));
                }
                n => {
                    let span = child.span();
                    let name = n.as_owned();

                    let err =
                        EMLErrorKind::UnexpectedElement(name, parent_name.clone()).with_span(span);
                    if child.parsing_mode().is_strict() {
                        return Err(err);
                    } else {
                        child.push_err(err);
                    }
                }
            }
        }
        let Some(result) = found_child else {
            return Err(EMLErrorKind::MissingChoiceElements(vec![
                MinimalQualifyingAddressCountry::EML_NAME.as_owned(),
                MinimalQualifyingAddressLocality::EML_NAME.as_owned(),
            ])
            .with_span(elem.full_span()));
        };
        Ok(result)
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        let mut writer = writer.content()?;
        match self {
            MinimalQualifyingAddress::Locality(locality) => {
                writer = writer.child_elem(MinimalQualifyingAddressLocality::EML_NAME, locality)?
            }
            MinimalQualifyingAddress::Country(country) => {
                writer = writer.child_elem(MinimalQualifyingAddressCountry::EML_NAME, country)?
            }
        }
        writer.finish()
    }
}

/// The minimal details for locality in a qualifying address
#[derive(Debug, Clone)]
pub struct MinimalQualifyingAddressLocality {
    /// Name of the locality
    pub locality_name: LocalityName,
}

impl MinimalQualifyingAddressLocality {
    /// Creates a new `MinimalQualifyingAddressLocality` with the given locality name.
    pub fn new(locality_name: LocalityName) -> Self {
        MinimalQualifyingAddressLocality { locality_name }
    }
}

impl EMLElement for MinimalQualifyingAddressLocality {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Locality", Some(NS_XAL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(
            elem,
            MinimalQualifyingAddressLocality {
                locality_name: LocalityName::EML_NAME => |elem| LocalityName::read_eml(elem)?,
            }
        ))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(LocalityName::EML_NAME, &self.locality_name)?
            .finish()
    }
}

/// The minimal details for country in a qualifying address
#[derive(Debug, Clone)]
pub struct MinimalQualifyingAddressCountry {
    /// The country name code, if present.
    pub country_name_code: CountryNameCode,
    /// The locality within the country.
    pub locality: MinimalQualifyingAddressLocality,
}

impl MinimalQualifyingAddressCountry {
    /// Creates a new `MinimalQualifyingAddressCountry` with the given country name code and locality.
    pub fn new(
        country_name_code: CountryNameCode,
        locality: MinimalQualifyingAddressLocality,
    ) -> Self {
        MinimalQualifyingAddressCountry {
            country_name_code,
            locality,
        }
    }
}

impl EMLElement for MinimalQualifyingAddressCountry {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Country", Some(NS_XAL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, MinimalQualifyingAddressCountry {
            country_name_code: CountryNameCode::EML_NAME => |elem| CountryNameCode::read_eml(elem)?,
            locality: MinimalQualifyingAddressLocality::EML_NAME => |elem| MinimalQualifyingAddressLocality::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(CountryNameCode::EML_NAME, &self.country_name_code)?
            .child_elem(MinimalQualifyingAddressLocality::EML_NAME, &self.locality)?
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{EMLParsingMode, EMLRead as _, test_write_eml_element, test_xml_fragment};

    #[test]
    fn test_minimal_qualifying_address_construction() {
        let locality_name = LocalityName::new("Amsterdam");
        let locality =
            MinimalQualifyingAddress::new_country(CountryNameCode::new("NL"), locality_name);

        if let MinimalQualifyingAddress::Country(country) = locality {
            assert_eq!(country.country_name_code.value.as_ref(), "NL");
            assert_eq!(country.locality.locality_name.name.as_ref(), "Amsterdam");
        } else {
            panic!("Expected a country qualifying address");
        }

        let locality_name = LocalityName::new("Rotterdam");
        let locality = MinimalQualifyingAddress::new_locality(locality_name);
        if let MinimalQualifyingAddress::Locality(locality) = locality {
            assert_eq!(locality.locality_name.name.as_ref(), "Rotterdam");
        } else {
            panic!("Expected a locality qualifying address");
        }
    }

    #[test]
    fn test_minimal_qualifying_address_country_parsing() {
        let xml = test_xml_fragment(
            r#"
            <QualifyingAddress xmlns="urn:oasis:names:tc:evs:schema:eml" xmlns:xal="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0">
                <xal:Country>
                    <xal:CountryNameCode>NL</xal:CountryNameCode>
                    <xal:Locality>
                        <xal:LocalityName>Amsterdam</xal:LocalityName>
                    </xal:Locality>
                </xal:Country>
            </QualifyingAddress>
            "#,
        );

        let address = MinimalQualifyingAddress::parse_eml(&xml, EMLParsingMode::Strict).unwrap();

        if let MinimalQualifyingAddress::Country(country) = &address {
            assert_eq!(country.country_name_code.value.as_ref(), "NL");
            assert_eq!(country.locality.locality_name.name.as_ref(), "Amsterdam");
        } else {
            panic!("Expected a country qualifying address");
        }

        let xml_output = test_write_eml_element(&address, &[NS_EML, NS_XAL]).unwrap();
        assert_eq!(xml_output, xml);
    }

    #[test]
    fn test_minimal_qualifying_address_locality_parsing() {
        let xml = test_xml_fragment(
            r#"
            <QualifyingAddress xmlns="urn:oasis:names:tc:evs:schema:eml" xmlns:xal="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0">
                <xal:Locality>
                    <xal:LocalityName>Amsterdam</xal:LocalityName>
                </xal:Locality>
            </QualifyingAddress>
            "#,
        );

        let address = MinimalQualifyingAddress::parse_eml(&xml, EMLParsingMode::Strict).unwrap();

        if let MinimalQualifyingAddress::Locality(locality) = &address {
            assert_eq!(locality.locality_name.name.as_ref(), "Amsterdam");
        } else {
            panic!("Expected a locality qualifying address");
        }

        let xml_output = test_write_eml_element(&address, &[NS_EML, NS_XAL]).unwrap();
        assert_eq!(xml_output, xml);
    }
}
