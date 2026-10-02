use crate::{
    EMLError, EMLErrorKind, NS_EML, NS_XAL,
    common::{CountryNameCode, LocalityName, PostalCode},
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName, collect_struct},
};

/// The qualifying address of a candidate.
#[derive(Debug, Clone)]
pub enum QualifyingAddress {
    /// Qualifying address is a locality only.
    Locality(QualifyingAddressLocality),

    /// Qualifying address is a locality in a specific country.
    Country(QualifyingAddressCountry),
}

impl QualifyingAddress {
    /// Create a new qualifying address with locality information and an optional country.
    pub fn new(
        locality: impl Into<QualifyingAddressLocality>,
        country_name_code: Option<impl Into<CountryNameCode>>,
    ) -> Self {
        match country_name_code {
            Some(code) => QualifyingAddress::Country(QualifyingAddressCountry {
                locality: locality.into(),
                country_name_code: Some(code.into()),
            }),
            None => QualifyingAddress::Locality(locality.into()),
        }
    }

    /// Get the locality information for the qualifying address.
    pub fn locality(&self) -> &QualifyingAddressLocality {
        match self {
            QualifyingAddress::Locality(locality) => locality,
            QualifyingAddress::Country(country) => &country.locality,
        }
    }

    /// Get the country information for the qualifying address, if present.
    pub fn country_name_code(&self) -> Option<&CountryNameCode> {
        match self {
            QualifyingAddress::Locality(_) => None,
            QualifyingAddress::Country(country) => country.country_name_code.as_ref(),
        }
    }
}

impl From<QualifyingAddressLocality> for QualifyingAddress {
    fn from(locality: QualifyingAddressLocality) -> Self {
        QualifyingAddress::Locality(locality)
    }
}

impl From<QualifyingAddressCountry> for QualifyingAddress {
    fn from(country: QualifyingAddressCountry) -> Self {
        QualifyingAddress::Country(country)
    }
}

impl EMLElement for QualifyingAddress {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("QualifyingAddress", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let parent_name = elem.name()?.as_owned();
        let mut found_value = None;
        while let Some(mut next_child) = elem.next_child()? {
            let name = next_child.name()?;
            if found_value.is_some()
                || name != QualifyingAddressLocality::EML_NAME
                    && name != QualifyingAddressCountry::EML_NAME
            {
                let err = EMLErrorKind::UnexpectedElement(name.as_owned(), parent_name.clone())
                    .with_span(next_child.span());
                if next_child.parsing_mode().is_strict() {
                    return Err(err);
                } else {
                    next_child.push_err(err);
                    next_child.skip()?;
                }
            } else {
                match name {
                    name if name == QualifyingAddressLocality::EML_NAME => {
                        let locality = QualifyingAddressLocality::read_eml(&mut next_child)?;
                        found_value = Some(QualifyingAddress::Locality(locality));
                    }
                    name if name == QualifyingAddressCountry::EML_NAME => {
                        let country = QualifyingAddressCountry::read_eml(&mut next_child)?;
                        found_value = Some(QualifyingAddress::Country(country));
                    }
                    _ => unreachable!(),
                }
            }
        }
        let Some(value) = found_value else {
            return Err(EMLErrorKind::MissingChoiceElements(vec![
                QualifyingAddressLocality::EML_NAME.as_owned(),
                QualifyingAddressCountry::EML_NAME.as_owned(),
            ])
            .with_span(elem.span()));
        };
        Ok(value)
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        match self {
            QualifyingAddress::Locality(locality) => {
                writer.child_elem(QualifyingAddressLocality::EML_NAME, locality)?
            }
            QualifyingAddress::Country(country) => {
                writer.child_elem(QualifyingAddressCountry::EML_NAME, country)?
            }
        }
        .finish()
    }
}

/// Qualifying address locality.
#[derive(Debug, Clone)]
pub struct QualifyingAddressLocality {
    /// The address line, if present.
    pub address_line: Option<AddressLine>,

    /// The locality name.
    pub locality_name: LocalityName,

    /// The postal code, if present.
    pub postal_code: Option<PostalCode>,

    /// The Type attribute, if present.
    pub locality_type: Option<Box<str>>,

    /// The UsageType attribute, if present.
    pub usage_type: Option<Box<str>>,

    /// The Indicator attribute, if present.
    pub indicator: Option<Box<str>>,
}

impl QualifyingAddressLocality {
    /// Create a new QualifyingAddressLocality.
    pub fn new(locality_name: impl Into<Box<str>>) -> Self {
        QualifyingAddressLocality {
            address_line: None,
            locality_name: LocalityName::new(locality_name),
            postal_code: None,
            locality_type: None,
            usage_type: None,
            indicator: None,
        }
    }

    /// Get the locality name for the qualifying address locality.
    pub fn locality_name(&self) -> &str {
        &self.locality_name.name
    }

    /// Set the address line for the locality.
    pub fn with_address_line(self, address_line: impl Into<AddressLine>) -> Self {
        self.with_address_line_option(Some(address_line))
    }

    /// Set the address line for the locality, if present.
    pub fn with_address_line_option(
        mut self,
        address_line: Option<impl Into<AddressLine>>,
    ) -> Self {
        self.address_line = address_line.map(Into::into);
        self
    }

    /// Set the postal code for the locality.
    pub fn with_postal_code(self, postal_code: impl Into<PostalCode>) -> Self {
        self.with_postal_code_option(Some(postal_code))
    }

    /// Set the postal code for the locality, if present.
    pub fn with_postal_code_option(mut self, postal_code: Option<impl Into<PostalCode>>) -> Self {
        self.postal_code = postal_code.map(Into::into);
        self
    }

    /// Set the Type attribute for the locality.
    pub fn with_locality_type(self, locality_type: impl Into<Box<str>>) -> Self {
        self.with_locality_type_option(Some(locality_type))
    }

    /// Set the Type attribute for the locality, if present.
    pub fn with_locality_type_option(mut self, locality_type: Option<impl Into<Box<str>>>) -> Self {
        self.locality_type = locality_type.map(Into::into);
        self
    }

    /// Set the UsageType attribute for the locality.
    pub fn with_usage_type(self, usage_type: impl Into<Box<str>>) -> Self {
        self.with_usage_type_option(Some(usage_type))
    }

    /// Set the UsageType attribute for the locality, if present.
    pub fn with_usage_type_option(mut self, usage_type: Option<impl Into<Box<str>>>) -> Self {
        self.usage_type = usage_type.map(Into::into);
        self
    }

    /// Set the Indicator attribute for the locality.
    pub fn with_indicator(self, indicator: impl Into<Box<str>>) -> Self {
        self.with_indicator_option(Some(indicator))
    }

    /// Set the Indicator attribute for the locality, if present.
    pub fn with_indicator_option(mut self, indicator: Option<impl Into<Box<str>>>) -> Self {
        self.indicator = indicator.map(Into::into);
        self
    }
}

impl From<&str> for QualifyingAddressLocality {
    fn from(value: &str) -> Self {
        QualifyingAddressLocality::new(value)
    }
}

impl From<String> for QualifyingAddressLocality {
    fn from(value: String) -> Self {
        QualifyingAddressLocality::new(value)
    }
}

impl From<Box<str>> for QualifyingAddressLocality {
    fn from(value: Box<str>) -> Self {
        QualifyingAddressLocality::new(value)
    }
}

impl EMLElement for QualifyingAddressLocality {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Locality", Some(NS_XAL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, QualifyingAddressLocality {
            address_line as Option: AddressLine::EML_NAME => |elem| elem.read_element::<AddressLine>()?,
            locality_name: LocalityName::EML_NAME => |elem| elem.read_element::<LocalityName>()?,
            postal_code as Option: PostalCode::EML_NAME => |elem| elem.read_element::<PostalCode>()?,
            locality_type: elem.attribute_value("Type")?.map(Into::into),
            usage_type: elem.attribute_value("UsageType")?.map(Into::into),
            indicator: elem.attribute_value("Indicator")?.map(Into::into),
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr_opt("Type", self.locality_type.as_ref())?
            .attr_opt("UsageType", self.usage_type.as_ref())?
            .attr_opt("Indicator", self.indicator.as_ref())?
            .child_elem_option(AddressLine::EML_NAME, self.address_line.as_ref())?
            .child_elem(LocalityName::EML_NAME, &self.locality_name)?
            .child_elem_option(PostalCode::EML_NAME, self.postal_code.as_ref())?
            .finish()
    }
}

/// Address line information.
#[derive(Debug, Clone)]
pub struct AddressLine {
    /// The address line value.
    pub value: Box<str>,

    /// The Type attribute, if present.
    pub address_line_type: Option<Box<str>>,

    /// The Code attribute, if present.
    pub code: Option<Box<str>>,
}

impl AddressLine {
    /// Create a new AddressLine.
    pub fn new(value: impl Into<Box<str>>) -> Self {
        AddressLine {
            value: value.into(),
            address_line_type: None,
            code: None,
        }
    }

    /// Set the Type attribute for the address line.
    pub fn with_type(mut self, address_line_type: impl Into<Box<str>>) -> Self {
        self.address_line_type = Some(address_line_type.into());
        self
    }

    /// Set the Code attribute for the address line.
    pub fn with_code(mut self, code: impl Into<Box<str>>) -> Self {
        self.code = Some(code.into());
        self
    }
}

impl From<&str> for AddressLine {
    fn from(value: &str) -> Self {
        AddressLine::new(value)
    }
}

impl From<String> for AddressLine {
    fn from(value: String) -> Self {
        AddressLine::new(value)
    }
}

impl From<Box<str>> for AddressLine {
    fn from(value: Box<str>) -> Self {
        AddressLine::new(value)
    }
}

impl EMLElement for AddressLine {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("AddressLine", Some(NS_XAL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(AddressLine {
            value: elem.text_without_children()?,
            address_line_type: elem.attribute_value("Type")?.map(Into::into),
            code: elem.attribute_value("Code")?.map(Into::into),
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr_opt("Type", self.address_line_type.as_ref())?
            .attr_opt("Code", self.code.as_ref())?
            .text(self.value.as_ref())?
            .finish()
    }
}

/// Qualifying address country.
#[derive(Debug, Clone)]
pub struct QualifyingAddressCountry {
    /// The country name code, if present.
    pub country_name_code: Option<CountryNameCode>,
    /// The locality within the country.
    pub locality: QualifyingAddressLocality,
}

impl QualifyingAddressCountry {
    /// Create a new QualifyingAddressCountry.
    pub fn new(
        country_code: Option<impl Into<Box<str>>>,
        locality: impl Into<QualifyingAddressLocality>,
    ) -> Self {
        Self {
            country_name_code: country_code.map(|code| CountryNameCode::new(code)),
            locality: locality.into(),
        }
    }
}

impl EMLElement for QualifyingAddressCountry {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Country", Some(NS_XAL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, QualifyingAddressCountry {
            country_name_code as Option: CountryNameCode::EML_NAME => |elem| elem.read_element::<CountryNameCode>()?,
            locality: QualifyingAddressLocality::EML_NAME => |elem| elem.read_element::<QualifyingAddressLocality>()?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem_option(CountryNameCode::EML_NAME, self.country_name_code.as_ref())?
            .child_elem(QualifyingAddressLocality::EML_NAME, &self.locality)?
            .finish()
    }
}

/// A mailing address, structured as a qualifying address (Locality or Country).
#[derive(Debug, Clone)]
pub struct MailingAddress {
    /// The address content (Locality or Country).
    pub address: QualifyingAddress,
}

impl MailingAddress {
    /// Create a new mailing address with a locality.
    pub fn new(address: impl Into<QualifyingAddress>) -> Self {
        MailingAddress {
            address: address.into(),
        }
    }
}

impl EMLElement for MailingAddress {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("MailingAddress", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let parent_name = elem.name()?.as_owned();
        let mut found_value = None;
        while let Some(mut next_child) = elem.next_child()? {
            let name = next_child.name()?;
            if found_value.is_some()
                || name != QualifyingAddressLocality::EML_NAME
                    && name != QualifyingAddressCountry::EML_NAME
            {
                let err = EMLErrorKind::UnexpectedElement(name.as_owned(), parent_name.clone())
                    .with_span(next_child.span());
                if next_child.parsing_mode().is_strict() {
                    return Err(err);
                } else {
                    next_child.push_err(err);
                    next_child.skip()?;
                }
            } else {
                match name {
                    name if name == QualifyingAddressLocality::EML_NAME => {
                        let locality = QualifyingAddressLocality::read_eml(&mut next_child)?;
                        found_value = Some(QualifyingAddress::Locality(locality));
                    }
                    name if name == QualifyingAddressCountry::EML_NAME => {
                        let country = QualifyingAddressCountry::read_eml(&mut next_child)?;
                        found_value = Some(QualifyingAddress::Country(country));
                    }
                    _ => unreachable!(),
                }
            }
        }
        let Some(value) = found_value else {
            return Err(EMLErrorKind::MissingChoiceElements(vec![
                QualifyingAddressLocality::EML_NAME.as_owned(),
                QualifyingAddressCountry::EML_NAME.as_owned(),
            ])
            .with_span(elem.span()));
        };
        Ok(MailingAddress { address: value })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        match &self.address {
            QualifyingAddress::Locality(locality) => {
                writer.child_elem(QualifyingAddressLocality::EML_NAME, locality)?
            }
            QualifyingAddress::Country(country) => {
                writer.child_elem(QualifyingAddressCountry::EML_NAME, country)?
            }
        }
        .finish()
    }
}

/// Contact details for a candidate (containing a mailing address).
#[derive(Debug, Clone)]
pub struct Contact {
    /// The mailing address.
    pub mailing_address: MailingAddress,
}

impl Contact {
    /// Create a new Contact with the given mailing address.
    pub fn new(mailing_address: impl Into<MailingAddress>) -> Self {
        Contact {
            mailing_address: mailing_address.into(),
        }
    }
}

impl EMLElement for Contact {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Contact", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, Contact {
            mailing_address: MailingAddress::EML_NAME => |elem| elem.read_element::<MailingAddress>()?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(MailingAddress::EML_NAME, &self.mailing_address)?
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        EMLVersion,
        io::{EMLParsingMode, EMLRead, test_write_eml_element, test_xml_fragment},
    };

    use super::*;

    #[test]
    fn test_qualifying_address_full() {
        let c = QualifyingAddressCountry::new(
            Some("NL"),
            QualifyingAddressLocality::new("Amsterdam")
                .with_address_line(
                    AddressLine::new("Test 1")
                        .with_code("TestCode")
                        .with_type("TestType"),
                )
                .with_postal_code(
                    PostalCode::new("1234 AB")
                        .with_number_code("TestCode")
                        .with_number_type("TestType"),
                )
                .with_indicator("Test")
                .with_locality_type("City")
                .with_usage_type("Example"),
        );

        assert_eq!(c.country_name_code, Some(CountryNameCode::new("NL")));
        assert_eq!(c.locality.locality_name.name.as_ref(), "Amsterdam");
        assert_eq!(
            c.locality.address_line.as_ref().unwrap().value.as_ref(),
            "Test 1"
        );
        assert_eq!(
            c.locality
                .address_line
                .as_ref()
                .unwrap()
                .code
                .as_ref()
                .unwrap()
                .as_ref(),
            "TestCode"
        );
        assert_eq!(
            c.locality
                .address_line
                .as_ref()
                .unwrap()
                .address_line_type
                .as_ref()
                .unwrap()
                .as_ref(),
            "TestType"
        );
        assert_eq!(
            c.locality
                .postal_code
                .as_ref()
                .unwrap()
                .number
                .number
                .as_ref(),
            "1234 AB"
        );
        assert_eq!(
            c.locality
                .postal_code
                .as_ref()
                .unwrap()
                .number
                .code
                .as_ref()
                .unwrap()
                .as_ref(),
            "TestCode"
        );
        assert_eq!(
            c.locality
                .postal_code
                .as_ref()
                .unwrap()
                .number
                .number_type
                .as_ref()
                .unwrap()
                .as_ref(),
            "TestType"
        );
        assert_eq!(c.locality.indicator.as_ref().unwrap().as_ref(), "Test");
        assert_eq!(c.locality.locality_type.as_ref().unwrap().as_ref(), "City");
        assert_eq!(c.locality.usage_type.as_ref().unwrap().as_ref(), "Example");

        test_write_eml_element(&c, &[NS_XAL], EMLVersion::default()).unwrap();
    }

    #[test]
    fn test_qualifying_address_parsing() {
        let xml = test_xml_fragment(
            r#"
            <QualifyingAddress xmlns="urn:oasis:names:tc:evs:schema:eml" xmlns:xal="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0">
                <xal:Country>
                    <xal:Locality>
                        <xal:LocalityName>Amsterdam</xal:LocalityName>
                    </xal:Locality>
                </xal:Country>
            </QualifyingAddress>
            "#,
        );

        let qualifying_address = QualifyingAddress::parse_eml_fragment(
            &xml,
            EMLParsingMode::Strict,
            EMLVersion::default(),
        )
        .ok()
        .unwrap();
        match &qualifying_address {
            QualifyingAddress::Country(country) => {
                assert_eq!(country.country_name_code, None);
                assert_eq!(country.locality.locality_name.name.as_ref(), "Amsterdam");
            }
            _ => panic!("Expected country qualifying address"),
        }

        let xml_output = test_write_eml_element(
            &qualifying_address,
            &[NS_EML, NS_XAL],
            EMLVersion::default(),
        )
        .unwrap();
        assert_eq!(xml_output, xml);
    }
}
