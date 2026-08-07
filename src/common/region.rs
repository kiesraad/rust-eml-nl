use crate::common::committee::Committee;
use crate::error::EMLValueResultExt as _;
use crate::io::{
    EMLElement, EMLElementReader, EMLElementWriter, OwnedQualifiedName, QualifiedName,
    collect_struct,
};
use crate::utils::RegionCategory;
use crate::{EMLError, EMLErrorKind, NS_KR};

const MAX_COMMITTEES: usize = 3;

/// The identity of a [`Region`] in the election tree.
///
/// EML_NL identifies a region by the combination of its `RegionCategory` and
/// `RegionNumber` attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegionKey {
    /// The category of the region.
    pub category: RegionCategory,

    /// The number of the region.
    ///
    /// The EML_NL spec tells us this should be i16, but it is in reality always
    /// a positive number, because regions may not have negative numbers in the
    /// rest of the EML spec. Therefore, we use u16 here to avoid processing
    /// election trees that we can't actually use.
    ///
    /// Not every region has a number.
    pub number: Option<u16>,
}

impl RegionKey {
    /// Create a new region key.
    pub fn new(category: RegionCategory, number: Option<u16>) -> Self {
        RegionKey { category, number }
    }

    /// Create a new region key for the [`RegionCategory::State`].
    pub fn state() -> Self {
        RegionKey {
            category: RegionCategory::State,
            number: None,
        }
    }

    /// Create a new region key for the [`RegionCategory::WaterAuthority`].
    pub fn water_authority(number: u16) -> Self {
        RegionKey {
            category: RegionCategory::WaterAuthority,
            number: Some(number),
        }
    }

    /// Create a new region key for the [`RegionCategory::Province`].
    pub fn province(number: u16) -> Self {
        RegionKey {
            category: RegionCategory::Province,
            number: Some(number),
        }
    }

    /// Create a new region key for the [`RegionCategory::ElectoralDistrict`].
    pub fn electoral_district(number: u16) -> Self {
        RegionKey {
            category: RegionCategory::ElectoralDistrict,
            number: Some(number),
        }
    }

    /// Create a new region key for the [`RegionCategory::WaterAuthorityElectoralDistrict`].
    pub fn water_authority_electoral_district(number: u16) -> Self {
        RegionKey {
            category: RegionCategory::WaterAuthorityElectoralDistrict,
            number: Some(number),
        }
    }

    /// Create a new region key for the [`RegionCategory::ProvinceElectoralDistrict`].
    pub fn province_electoral_district(number: u16) -> Self {
        RegionKey {
            category: RegionCategory::ProvinceElectoralDistrict,
            number: Some(number),
        }
    }

    /// Create a new region key for the [`RegionCategory::ProvincePollingStation`].
    pub fn province_polling_station(number: u16) -> Self {
        RegionKey {
            category: RegionCategory::ProvincePollingStation,
            number: Some(number),
        }
    }

    /// Create a new region key for the [`RegionCategory::WaterAuthorityMunicipality`].
    pub fn water_authority_municipality(number: u16) -> Self {
        RegionKey {
            category: RegionCategory::WaterAuthorityMunicipality,
            number: Some(number),
        }
    }

    /// Create a new region key for the [`RegionCategory::Municipality`].
    pub fn municipality(number: u16) -> Self {
        RegionKey {
            category: RegionCategory::Municipality,
            number: Some(number),
        }
    }

    /// Create a new region key for the [`RegionCategory::SubMunicipality`].
    pub fn sub_municipality(number: u16) -> Self {
        RegionKey {
            category: RegionCategory::SubMunicipality,
            number: Some(number),
        }
    }

    /// Create a new region key for the [`RegionCategory::PollingStation`].
    pub fn polling_station(number: u16) -> Self {
        RegionKey {
            category: RegionCategory::PollingStation,
            number: Some(number),
        }
    }
}

/// Region in the election tree, which can contain a number of electoral
/// committees and defines some properties of the region.
#[derive(Debug, Clone)]
pub struct Region {
    /// The name of the region.
    pub name: Box<str>,
    /// The committees in this region.
    pub committees: Vec<Committee>,
    /// The key identifying this region, its category and number.
    pub key: RegionKey,
    /// Whether this region uses roman numerals for the contest identifier in
    /// other parts of the EML_NL spec.
    ///
    /// Note: this only happens within Limburg
    pub roman_numerals: bool,
    /// This region allows exporting in the Frysian language.
    pub frysian_export_allowed: bool,
    /// The key identifying the superior region on the tree, if this region has
    /// one. Regions at the root of an election tree do not have a superior
    /// region.
    pub superior_region_key: Option<RegionKey>,
}

impl Region {
    /// Create a new region.
    pub fn new(region_name: impl Into<Box<str>>, region_category: RegionCategory) -> Self {
        Region {
            name: region_name.into(),
            committees: Vec::new(),
            key: RegionKey::new(region_category, None),
            roman_numerals: false,
            frysian_export_allowed: false,
            superior_region_key: None,
        }
    }

    /// Set the `RegionNumber` attribute of the `Region` element.
    pub fn with_number(mut self, region_number: u16) -> Self {
        self.key.number = Some(region_number);
        self
    }

    /// Set the `RomanNumerals` attribute of the `Region` element.
    pub fn with_roman_numerals(mut self, roman_numerals: bool) -> Self {
        self.roman_numerals = roman_numerals;
        self
    }

    /// Set the `FrysianExportAllowed` attribute of the `Region` element.
    pub fn with_frysian_export_allowed(mut self, enabled: bool) -> Self {
        self.frysian_export_allowed = enabled;
        self
    }

    /// Set the `SuperiorRegionCategory` and `SuperiorRegionNumber` attributes of
    /// the `Region` element.
    pub fn with_superior_region_key(mut self, superior_region_key: RegionKey) -> Self {
        self.superior_region_key = Some(superior_region_key);
        self
    }

    /// Set the `Committee` elements of the `Region` element.
    ///
    /// Note: this will replace any existing committees in the region.
    pub fn with_committees(mut self, committees: Vec<Committee>) -> Self {
        self.committees = committees;
        self
    }

    /// Add a `Committee` element to the `Region` element.
    pub fn push_committee(&mut self, committee: Committee) {
        self.committees.push(committee);
    }
}

impl EMLElement for Region {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Region", Some(NS_KR));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let number = elem
            .attribute_value("RegionNumber")?
            .map(|value| value.parse::<u16>())
            .transpose()
            .wrap_value_error()?;
        let category = RegionCategory::new(elem.attribute_value_req("RegionCategory")?)?;
        let roman_numerals = elem
            .string_value_attr("RomanNumerals", Some("false"))?
            .copied_value()?;
        let frysian_export_allowed = elem
            .string_value_attr("FrysianExportAllowed", Some("false"))?
            .copied_value()?;
        let superior_region_number = elem
            .attribute_value("SuperiorRegionNumber")?
            .map(|value| value.parse::<u16>())
            .transpose()
            .wrap_value_error()?;
        let superior_region_category = elem
            .attribute_value("SuperiorRegionCategory")?
            .map(RegionCategory::new)
            .transpose()?;

        // A superior region is identified by its category and number combined,
        // so a superior region number without a category does not refer to any
        // region at all. In non-strict mode we ignore the number and treat the
        // region as having no superior region.
        let superior_region_key = match (superior_region_category, superior_region_number) {
            (Some(category), number) => Some(RegionKey::new(category, number)),
            (None, None) => None,
            (None, Some(_)) => {
                let err = EMLErrorKind::MissingAttribute(OwnedQualifiedName::from_static(
                    "SuperiorRegionCategory",
                    None,
                ))
                .with_span(elem.span());
                if elem.parsing_mode().is_strict() {
                    return Err(err);
                } else {
                    elem.push_err(err);
                    None
                }
            }
        };

        let region = collect_struct!(elem, Region {
        key: RegionKey::new(category, number),
        roman_numerals: roman_numerals,
        frysian_export_allowed: frysian_export_allowed,
        superior_region_key: superior_region_key,
        name: ("RegionName", NS_KR) => |elem| elem.text_without_children()?,
        committees as Vec: Committee::EML_NAME => |elem| Committee::read_eml(elem)?,
            });

        if region.committees.len() > MAX_COMMITTEES {
            let err = EMLErrorKind::TooManyElements(Committee::EML_NAME.as_owned(), MAX_COMMITTEES)
                .with_span(elem.full_span());
            if elem.parsing_mode().is_strict() {
                return Err(err);
            } else {
                elem.push_err(err);
            }
        }

        Ok(region)
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        let roman_numerals = self.roman_numerals.to_string();
        let frysian_export_allowed = self.frysian_export_allowed.to_string();

        writer
            .attr_opt(
                "RegionNumber",
                self.key.number.map(|number| number.to_string()),
            )?
            .attr("RegionCategory", self.key.category.to_eml_value())?
            .attr("RomanNumerals", &roman_numerals)?
            .attr("FrysianExportAllowed", &frysian_export_allowed)?
            .attr_opt(
                "SuperiorRegionNumber",
                self.superior_region_key
                    .and_then(|key| key.number)
                    .map(|number| number.to_string()),
            )?
            .attr_opt(
                "SuperiorRegionCategory",
                self.superior_region_key
                    .map(|key| key.category.to_eml_value()),
            )?
            .content()?
            .child(("RegionName", NS_KR), |writer| {
                writer.text(self.name.as_ref())?.finish()
            })?
            .child_elems(Committee::EML_NAME, &self.committees)?
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EMLErrorKind;
    use crate::io::{EMLParsingMode, EMLRead, test_write_eml_element, test_xml_fragment};
    use crate::utils::CommitteeCategory;

    #[test]
    fn test_region_parsing() {
        let xml = test_xml_fragment(
            r#"
                <kr:Region xmlns:kr="http://www.kiesraad.nl/extensions" RegionNumber="1" RegionCategory="WATERSCHAP" RomanNumerals="false" FrysianExportAllowed="false" SuperiorRegionNumber="0" SuperiorRegionCategory="STAAT">
                    <kr:RegionName>Region 1</kr:RegionName>
                    <kr:Committee CommitteeCategory="HSB"/>
                    <kr:Committee CommitteeCategory="PSB"/>
                </kr:Region>
            "#,
        );
        let region = Region::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(region.name.as_ref(), "Region 1");
        assert_eq!(region.committees.len(), 2);
        assert_eq!(region.committees[0].category, CommitteeCategory::HSB);
        assert_eq!(region.committees[1].category, CommitteeCategory::PSB);
        assert_eq!(
            region.key,
            RegionKey::new(RegionCategory::WaterAuthority, Some(1))
        );
        assert!(!region.roman_numerals);
        assert!(!region.frysian_export_allowed);
        assert_eq!(
            region.superior_region_key,
            Some(RegionKey::new(RegionCategory::State, Some(0)))
        );

        let xml_output = test_write_eml_element(&region, &[NS_KR]).unwrap();
        assert_eq!(xml_output, xml);
    }

    #[test]
    fn test_region_rejects_too_many_committees() {
        let xml = test_xml_fragment(
            r#"
                <kr:Region xmlns:kr="http://www.kiesraad.nl/extensions" RegionCategory="GEMEENTE">
                    <kr:RegionName>Region 1</kr:RegionName>
                    <kr:Committee CommitteeCategory="CSB"/>
                    <kr:Committee CommitteeCategory="HSB"/>
                    <kr:Committee CommitteeCategory="PROV_SB"/>
                    <kr:Committee CommitteeCategory="PSB"/>
                </kr:Region>
            "#,
        );
        let error = Region::parse_eml(&xml, EMLParsingMode::Strict)
            .ok()
            .unwrap_err();
        assert!(matches!(error.kind(), EMLErrorKind::TooManyElements(_, 3)));
    }
}
