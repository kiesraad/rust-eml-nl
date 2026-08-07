use crate::common::region::Region;
use crate::utils::{ElectionTreeHierarchy, ElectionTreeHierarchyError};
use crate::{
    EMLError, EMLErrorKind, NS_KR,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName, collect_struct},
};

/// Election tree as defined in EML_NL.
///
/// This is the flat list of regions as it appears in an EML_NL document, where
/// every region refers to its superior region by category and number. Use
/// [`ElectionTree::hierarchy`] to resolve those references into an
/// [`ElectionTreeHierarchy`], which exposes the same data as an actual tree.
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

    /// This election tree in a structured form, with the flat list of regions
    /// resolved into an actual tree.
    ///
    /// Returns an error if the regions do not describe a valid tree, see
    /// [`ElectionTreeHierarchyError`] for the ways in which that can happen.
    pub fn hierarchy(&self) -> Result<ElectionTreeHierarchy, ElectionTreeHierarchyError> {
        ElectionTreeHierarchy::try_from(self)
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
        let tree = collect_struct!(elem, ElectionTree {
            regions as Vec: Region::EML_NAME => |elem| Region::read_eml(elem)?,
        });

        // The XSD declares `Region` with an implicit `minOccurs="1"`
        // (`maxOccurs="unbounded"`), so an election tree must contain at least
        // one region.
        if tree.regions.is_empty() {
            let err = EMLErrorKind::MissingElement(Region::EML_NAME.as_owned())
                .with_span(elem.full_span());
            if elem.parsing_mode().is_strict() {
                return Err(err);
            } else {
                elem.push_err(err);
            }
        }

        Ok(tree)
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
    use crate::common::region::RegionKey;
    use crate::io::{EMLParsingMode, EMLRead, test_write_eml_element, test_xml_fragment};
    use crate::utils::{CommitteeCategory, RegionCategory};

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
        assert_eq!(
            tree.regions[0].key,
            RegionKey::new(RegionCategory::State, None)
        );
        assert_eq!(tree.regions[0].superior_region_key, None);
        assert!(tree.regions[0].committees.is_empty());

        assert_eq!(tree.regions[1].name.as_ref(), "Region 2");
        assert_eq!(
            tree.regions[1].key,
            RegionKey::new(RegionCategory::Province, Some(1))
        );
        assert_eq!(
            tree.regions[1].superior_region_key,
            Some(RegionKey::new(RegionCategory::State, None))
        );
        assert_eq!(tree.regions[1].committees.len(), 1);
        assert_eq!(
            tree.regions[1].committees[0].category,
            CommitteeCategory::CSB
        );

        assert_eq!(tree.regions[2].name.as_ref(), "Region 3");
        assert_eq!(
            tree.regions[2].key,
            RegionKey::new(RegionCategory::ElectoralDistrict, Some(2))
        );
        assert_eq!(
            tree.regions[2].superior_region_key,
            Some(RegionKey::new(RegionCategory::Province, Some(1)))
        );
        assert_eq!(tree.regions[2].committees.len(), 1);
        assert_eq!(
            tree.regions[2].committees[0].category,
            CommitteeCategory::ProvSB
        );

        assert_eq!(tree.regions[3].name.as_ref(), "Region 4");
        assert_eq!(
            tree.regions[3].key,
            RegionKey::new(RegionCategory::Municipality, Some(3))
        );
        assert_eq!(
            tree.regions[3].superior_region_key,
            Some(RegionKey::new(RegionCategory::ElectoralDistrict, Some(2)))
        );
        assert_eq!(tree.regions[3].committees.len(), 1);
        assert_eq!(
            tree.regions[3].committees[0].category,
            CommitteeCategory::HSB
        );

        let xml_output = test_write_eml_element(&tree, &[NS_KR]).unwrap();
        pretty_assertions::assert_eq!(xml_output, xml);
    }

    #[test]
    fn test_election_tree_rejects_empty() {
        let xml =
            test_xml_fragment(r#"<kr:ElectionTree xmlns:kr="http://www.kiesraad.nl/extensions"/>"#);
        let error = ElectionTree::parse_eml(&xml, EMLParsingMode::Strict)
            .ok()
            .unwrap_err();
        assert!(matches!(error.kind(), EMLErrorKind::MissingElement(_)));
    }
}
