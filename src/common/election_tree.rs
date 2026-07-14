use crate::common::region::Region;
use crate::{
    EMLError, NS_KR,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName, collect_struct},
};

/// Election tree as defined in EML_NL.
#[derive(Debug, Clone)]
pub struct ElectionTree {
    /// Regions defined for this part of the election tree
    pub regions: Vec<Region>,
}

impl ElectionTree {
    /// Create a new election tree with the given regions.
    pub fn new(regions: impl Into<Vec<Region>>) -> Self {
        ElectionTree {
            regions: regions.into(),
        }
    }
}

impl From<Vec<Region>> for ElectionTree {
    fn from(regions: Vec<Region>) -> Self {
        ElectionTree::new(regions)
    }
}

impl EMLElement for ElectionTree {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("ElectionTree", Some(NS_KR));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, ElectionTree {
            regions as Vec: Region::EML_NAME => |elem| Region::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elems(Region::EML_NAME, &self.regions)?
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::committee_category::CommitteeCategory;
    use crate::common::region_category::RegionCategory;
    use crate::io::{EMLParsingMode, EMLRead, test_write_eml_element, test_xml_fragment};

    #[test]
    fn test_election_tree_parsing() {
        let xml = test_xml_fragment(
            r#"
            <kr:ElectionTree xmlns:kr="http://www.kiesraad.nl/extensions">
                <kr:Region RegionCategory="STAAT" RomanNumerals="false" FrysianExportAllowed="false">
                    <kr:RegionName>Region 1</kr:RegionName>
                </kr:Region>
                <kr:Region RegionNumber="1" RegionCategory="PROVINCIE" RomanNumerals="false" FrysianExportAllowed="false" SuperiorRegionCategory="STAAT">
                    <kr:RegionName>Region 2</kr:RegionName>
                    <kr:Committee CommitteeCategory="CSB"/>
                </kr:Region>
                <kr:Region RegionNumber="2" RegionCategory="KIESKRING" RomanNumerals="false" FrysianExportAllowed="false" SuperiorRegionNumber="1" SuperiorRegionCategory="PROVINCIE">
                    <kr:RegionName>Region 3</kr:RegionName>
                    <kr:Committee CommitteeCategory="PROV_SB"/>
                </kr:Region>
                <kr:Region RegionNumber="3" RegionCategory="GEMEENTE" RomanNumerals="false" FrysianExportAllowed="false" SuperiorRegionNumber="2" SuperiorRegionCategory="KIESKRING">
                    <kr:RegionName>Region 4</kr:RegionName>
                    <kr:Committee CommitteeCategory="HSB"/>
                </kr:Region>
            </kr:ElectionTree>
            "#,
        );

        let tree = ElectionTree::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(tree.regions.len(), 4);

        assert_eq!(tree.regions[0].name.as_ref(), "Region 1");
        assert_eq!(tree.regions[0].category, RegionCategory::State);
        assert_eq!(tree.regions[0].number, None);
        assert_eq!(tree.regions[0].superior_region_category, None);
        assert!(tree.regions[0].committees.is_empty());

        assert_eq!(tree.regions[1].name.as_ref(), "Region 2");
        assert_eq!(tree.regions[1].number, Some(1));
        assert_eq!(tree.regions[1].category, RegionCategory::Province);
        assert_eq!(
            tree.regions[1].superior_region_category,
            Some(RegionCategory::State)
        );
        assert_eq!(tree.regions[1].committees.len(), 1);
        assert_eq!(
            tree.regions[1].committees[0].category,
            CommitteeCategory::CSB
        );

        assert_eq!(tree.regions[2].name.as_ref(), "Region 3");
        assert_eq!(tree.regions[2].number, Some(2));
        assert_eq!(tree.regions[2].category, RegionCategory::ElectoralDistrict);
        assert_eq!(
            tree.regions[2].superior_region_category,
            Some(RegionCategory::Province)
        );
        assert_eq!(tree.regions[2].superior_region_number, Some(1));
        assert_eq!(tree.regions[2].committees.len(), 1);
        assert_eq!(
            tree.regions[2].committees[0].category,
            CommitteeCategory::ProvSB
        );

        assert_eq!(tree.regions[3].name.as_ref(), "Region 4");
        assert_eq!(tree.regions[3].number, Some(3));
        assert_eq!(tree.regions[3].category, RegionCategory::Municipality);
        assert_eq!(
            tree.regions[3].superior_region_category,
            Some(RegionCategory::ElectoralDistrict)
        );
        assert_eq!(tree.regions[3].superior_region_number, Some(2));
        assert_eq!(tree.regions[3].committees.len(), 1);
        assert_eq!(
            tree.regions[3].committees[0].category,
            CommitteeCategory::HSB
        );

        let xml_output = test_write_eml_element(&tree, &[NS_KR]).unwrap();
        pretty_assertions::assert_eq!(xml_output, xml);
    }
}
