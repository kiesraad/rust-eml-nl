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
///
/// In essence, the election tree communicates a hierarchy of electoral
/// committees, where each committee handles the election at a certain level.
/// In Dutch elections, we typically have either two or three committee levels
/// (at the time of writing this documentation). The top level committee will
/// always be a CSB (Centraal Stembureau, central electoral committee), the
/// levels below that depend on the election category. Currently however, most
/// elections in the election tree consist of an optional layer of HSBs at the
/// secondary level and a layer of GSBs below that. Which committees actually
/// exist depends on the election category and subcategory as defined in the
/// election identifier.
///
/// The information on committees is however not communicated by
/// [`Committee`](crate::common::Committee) elements only. Instead, the primary
/// entry point will be the root [`Region`] of the election tree. This is the
/// root region that an election will be concerned about. For example, for
/// country wide elections, you will typically find that the root region is of
/// the `STAAT` category. For municipal elections, the root region will be one
/// of the `GEMEENTE` category. This root region is also the region that the CSB
/// committee is responsible for. Any committees lower in the tree will then
/// either sit at the same level (for example the GSB for a GR election will be
/// at the same level in the tree as the CSB for that election), or at a lower
/// level (for example an election of the House of Representatives (Tweede Kamer,
/// or TK) is defined in a level just below the root level). The exact way
/// regions are nested depends on the election category, but most of the time
/// sub-committees will be defined in lower levels only.
///
/// However, looking at practical examples of country wide elections, you will
/// often find that a `Committee` element containing the CSB committee category
/// is not present in the root region, but at some other region in the tree.
/// This is because the `Committee` element instead is used to communicate the
/// seating region of the Committee. For example, for the house of representatives
/// election, the CSB committee is located under the `Region` of category
/// `KIESKRING` for the 's-Gravenhage region, indicating that this is where the
/// CSB committee is seated for that election. Note that if there is no
/// `Committee` element for a certain committee category, it is assumed that the
/// committee is seated directly in the region that it is responsible for. For
/// example, the election tree never defines GSB committees, so they are just
/// assumed to be in their associated municipality (i.e. `GEMEENTE`) region.
/// Note that sometimes Committee elements exist where one is not expected, i.e.
/// an election of the provincial council with one electoral district (PS1) that
/// still has a HSB defined. This seems to be a particularity of how the
/// election tree element is created. In general the existence of a Committee
/// element does not mean that such a committee exists, but the election
/// category and subcategory always define which committees exist.
///
/// There is one additional piece of information that is passed by the election
/// tree, which is the district/contest information. Whether or not an election
/// has districts/multiple contests depends on the election category and
/// subcategory. For example, a PS category election only has districts for a
/// PS2 subcategory election, but not for a PS1 subcategory. Meanwhile, GR
/// elections will never have districts whereas a TK election will always have
/// districts. For most elections with districts/multiple contests, this will be
/// closely associated with the `KIESKRING` region category (i.e. electoral
/// districts). But senate/eerste kamer elections do not use those, but still
/// have multiple contests as defined in the associated EML files. The exact way
/// contest/district information is communicated will depend on the exact
/// election category/subcategory. But in general, if an election has districts,
/// the contests are defined by the secondary level regions and their associated
/// region numbers. District numbers will end up in the [`crate::utils::ContestId`]
/// elements of other EML_NL documents. When there are no districts we will use
/// the `geen` (none) identifier instead. When a document (or part of a document)
/// concerns all the contests within an election we use `alle` (all) instead as
/// the identifier.
///
/// So, in summary, the election tree communicates three main region-based
/// pieces of information:
///
/// - The regions electoral committees are responsible for and the way those
///   regions are structured
/// - The regions where electoral committees are seated
/// - The regions that define the districts/contests of an election
///
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
                <kr:Region RegionCategory="STAAT">
                    <kr:RegionName>Region 1</kr:RegionName>
                </kr:Region>
                <kr:Region RegionNumber="1" RegionCategory="PROVINCIE" RomanNumerals="true" FrysianExportAllowed="true" SuperiorRegionCategory="STAAT">
                    <kr:RegionName>Region 2</kr:RegionName>
                    <kr:Committee CommitteeCategory="CSB"/>
                </kr:Region>
                <kr:Region RegionNumber="2" RegionCategory="KIESKRING" SuperiorRegionNumber="1" SuperiorRegionCategory="PROVINCIE">
                    <kr:RegionName>Region 3</kr:RegionName>
                    <kr:Committee CommitteeCategory="PROV_SB"/>
                </kr:Region>
                <kr:Region RegionNumber="3" RegionCategory="GEMEENTE" SuperiorRegionNumber="2" SuperiorRegionCategory="KIESKRING">
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
        assert!(!tree.regions[0].roman_numerals);
        assert!(!tree.regions[0].frysian_export_allowed);
        assert_eq!(tree.regions[0].superior_region_key, None);
        assert!(tree.regions[0].committees.is_empty());

        assert_eq!(tree.regions[1].name.as_ref(), "Region 2");
        assert_eq!(
            tree.regions[1].key,
            RegionKey::new(RegionCategory::Province, Some(1))
        );
        assert!(tree.regions[1].roman_numerals);
        assert!(tree.regions[1].frysian_export_allowed);
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
