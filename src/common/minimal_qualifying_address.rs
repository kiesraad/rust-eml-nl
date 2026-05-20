use std::fmt;

use instant_xml::{Accumulate, Deserializer, FromXml, Kind, ToXml};

use crate::{NS_EML, NS_XAL};

use super::{CountryNameCode, LocalityName};

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

impl<'xml> FromXml<'xml> for MinimalQualifyingAddress {
    fn matches(id: instant_xml::Id<'_>, field: Option<instant_xml::Id<'_>>) -> bool {
        match field {
            Some(field) => id == field,
            None => {
                id == instant_xml::Id {
                    ns: NS_EML,
                    name: "QualifyingAddress",
                }
            }
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut Deserializer<'cx, 'xml>,
    ) -> Result<(), instant_xml::Error> {
        use instant_xml::{Error, de::Node};

        if into.is_some() {
            return Err(Error::DuplicateValue(field));
        }

        while let Some(node) = deserializer.next() {
            let element = match node? {
                Node::Open(element) => element,
                Node::Text(s) if s.trim().is_empty() => continue,
                node => return Err(Error::UnexpectedNode(format!("{node:?}"))),
            };

            let id = deserializer.element_id(&element)?;
            if MinimalQualifyingAddressLocality::matches(id, None) {
                let mut acc =
                    <MinimalQualifyingAddressLocality as FromXml<'xml>>::Accumulator::default();
                let mut nested = deserializer.nested(element);
                MinimalQualifyingAddressLocality::deserialize(&mut acc, field, &mut nested)?;
                nested.ignore()?;
                *into = Some(MinimalQualifyingAddress::Locality(acc.try_done(field)?));
            } else if MinimalQualifyingAddressCountry::matches(id, None) {
                let mut acc =
                    <MinimalQualifyingAddressCountry as FromXml<'xml>>::Accumulator::default();
                let mut nested = deserializer.nested(element);
                MinimalQualifyingAddressCountry::deserialize(&mut acc, field, &mut nested)?;
                nested.ignore()?;
                *into = Some(MinimalQualifyingAddress::Country(acc.try_done(field)?));
            } else {
                let mut nested = deserializer.nested(element);
                nested.ignore()?;
            }
        }

        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: Kind = Kind::Element;
}

// Custom: enum dispatch (Locality/Country variants) inside a `<QualifyingAddress>` element.
impl ToXml for MinimalQualifyingAddress {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix = serializer.write_start(
            "QualifyingAddress",
            NS_EML,
            None::<instant_xml::ser::Context<0>>,
        )?;

        serializer.end_start()?;
        match self {
            MinimalQualifyingAddress::Locality(locality) => locality.serialize(None, serializer)?,
            MinimalQualifyingAddress::Country(country) => country.serialize(None, serializer)?,
        }

        serializer.write_close(prefix)
    }
}

/// The minimal details for locality in a qualifying address
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "Locality", ns(NS_XAL), force_prefix)]
pub struct MinimalQualifyingAddressLocality {
    /// Name of the locality
    #[xml(rename = "LocalityName")]
    pub locality_name: LocalityName,
}

impl MinimalQualifyingAddressLocality {
    /// Creates a new `MinimalQualifyingAddressLocality` with the given locality name.
    pub fn new(locality_name: LocalityName) -> Self {
        MinimalQualifyingAddressLocality { locality_name }
    }
}

/// The minimal details for country in a qualifying address
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "Country", ns(NS_XAL), force_prefix)]
pub struct MinimalQualifyingAddressCountry {
    /// The country name code, if present.
    #[xml(rename = "CountryNameCode")]
    pub country_name_code: CountryNameCode,
    /// The locality within the country.
    #[xml(rename = "Locality")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{EMLParsingMode, EMLRead as _, test_xml_fragment};

    #[test]
    fn test_minimal_qualifying_address_construction() {
        let locality_name = LocalityName::new("Amsterdam");
        let locality =
            MinimalQualifyingAddress::new_country(CountryNameCode::new("NL"), locality_name);

        if let MinimalQualifyingAddress::Country(country) = locality {
            assert_eq!(country.country_name_code.value, "NL");
            assert_eq!(country.locality.locality_name.name, "Amsterdam");
        } else {
            panic!("Expected a country qualifying address");
        }

        let locality_name = LocalityName::new("Rotterdam");
        let locality = MinimalQualifyingAddress::new_locality(locality_name);
        if let MinimalQualifyingAddress::Locality(locality) = locality {
            assert_eq!(locality.locality_name.name, "Rotterdam");
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
            assert_eq!(country.country_name_code.value, "NL");
            assert_eq!(country.locality.locality_name.name, "Amsterdam");
        } else {
            panic!("Expected a country qualifying address");
        }
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
            assert_eq!(locality.locality_name.name, "Amsterdam");
        } else {
            panic!("Expected a locality qualifying address");
        }
    }
}
