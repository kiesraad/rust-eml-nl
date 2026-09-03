//! Document variant for the Kiesraad master election tree.
//!
//! The master election tree is a Kiesraad-internal document (not officially part
//! of the EML_NL standard) that contains every region, council and committee for
//! every election category at once, organized as a single nested tree rooted
//! at the state (`STAAT`). The flat [`ElectionTree`](crate::common::ElectionTree)
//! contained in an EML_NL `110a` document for a specific election is derived
//! from a subset of this tree.

use std::collections::{BTreeMap, HashSet};
use std::str::FromStr;

use thiserror::Error;

use crate::{
    EMLError, EMLErrorKind,
    common::{Committee, RegionKey},
    error::{EMLResultExt as _, EMLValueResultExt as _},
    io::{
        EMLElement, EMLElementReader, EMLElementWriter, EMLParsingMode, EMLWriteInternal as _,
        QualifiedName, collect_struct,
    },
    utils::{CommitteeCategory, ElectionCategory, RegionCategory, StringValue, XsDateTime},
};

/// Error returned when the regions of a [`MasterElectionTree`] do not describe
/// a valid tree.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum MasterElectionTreeError {
    /// The root region of a master election tree must be of category [`RegionCategory::State`].
    #[error("The root region of a master election tree must be of category STAAT, found {0:?}")]
    RootNotState(RegionKey),

    /// A region has a subregion that does not sit at a lower level in the
    /// election tree, while a region's subregions must always be of a lower
    /// category than the region itself.
    #[error(
        "Region {region:?} has subregion {subregion:?}, which does not sit at a lower level in the election tree"
    )]
    InvalidSubregionCategory {
        /// The region containing the invalid subregion.
        region: RegionKey,
        /// The subregion that does not sit at a lower level than its parent.
        subregion: RegionKey,
    },

    /// A region of a category other than [`RegionCategory::State`] is missing
    /// its `RegionNumber` attribute, which is required for every category but
    /// the state.
    #[error("Region of category {0:?} is missing its RegionNumber attribute")]
    MissingRegionNumber(RegionCategory),

    /// A region has more than one subregion with the same category and number.
    #[error("Region {region:?} has more than one subregion with identity {subregion:?}")]
    DuplicateSubregion {
        /// The region containing the duplicate subregions.
        region: RegionKey,
        /// The identity shared by more than one subregion.
        subregion: RegionKey,
    },

    /// A region has more than one `Council` for the same election category.
    #[error(
        "Region {region:?} has more than one Council for election category {election_category:?}"
    )]
    DuplicateCouncil {
        /// The region containing the duplicate councils.
        region: RegionKey,
        /// The election category shared by more than one council.
        election_category: ElectionCategory,
    },
}

impl From<MasterElectionTreeError> for EMLError {
    fn from(err: MasterElectionTreeError) -> Self {
        EMLErrorKind::InvalidMasterElectionTree(err).without_span()
    }
}

/// Reports a [`MasterElectionTreeError`] encountered while reading a
/// [`MasterElectionTree`] or [`MetRegion`], failing immediately if the parsing
/// mode is strict, or collecting it as a non-fatal error otherwise.
fn report_tree_error(
    elem: &mut EMLElementReader<'_, '_>,
    err: MasterElectionTreeError,
) -> Result<(), EMLError> {
    let err = EMLErrorKind::InvalidMasterElectionTree(err).with_span(elem.full_span());
    if elem.parsing_mode().is_strict() {
        Err(err)
    } else {
        elem.push_err(err);
        Ok(())
    }
}

/// The Kiesraad master election tree, containing every region, council and
/// committee for every election category.
#[derive(Debug, Clone)]
pub struct MasterElectionTree {
    /// The date and time this master election tree was generated.
    pub creation_date: StringValue<XsDateTime>,

    /// The root region of the tree, always a region of category `STAAT`.
    pub root: MetRegion,
}

impl MasterElectionTree {
    /// Create a new master election tree with the given creation date and root region.
    pub fn new(creation_date: XsDateTime, root: MetRegion) -> Self {
        MasterElectionTree {
            creation_date: StringValue::from_value(creation_date),
            root,
        }
    }

    /// Write this master election tree document to an XML string.
    ///
    /// Unlike the other EML_NL document types, a master election tree is not
    /// wrapped in an `<EML>` root element because it is not an official EML_NL
    /// document.
    pub fn write_str(
        &self,
        pretty_print: bool,
        include_declaration: bool,
    ) -> Result<String, EMLError> {
        self.write_root_str(
            Some(Self::EML_NAME),
            Some(None),
            Some(BTreeMap::new()),
            pretty_print,
            include_declaration,
        )
    }
}

impl FromStr for MasterElectionTree {
    type Err = EMLError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::io::EMLRead as _;
        Self::parse_eml(s, EMLParsingMode::Strict).ok()
    }
}

impl TryFrom<&str> for MasterElectionTree {
    type Error = EMLError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use crate::io::EMLRead as _;
        Self::parse_eml(value, EMLParsingMode::Strict).ok()
    }
}

impl TryFrom<MasterElectionTree> for String {
    type Error = EMLError;

    fn try_from(value: MasterElectionTree) -> Result<Self, Self::Error> {
        value.write_str(true, true)
    }
}

impl EMLElement for MasterElectionTree {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("MasterElectionTree", None);

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        if !elem.has_name(Self::EML_NAME)? {
            return Err(EMLErrorKind::InvalidRootElement).with_span(elem.span());
        }

        let creation_date = elem.string_value_attr("CreationDate", None)?;

        let tree = collect_struct!(elem, MasterElectionTree {
            creation_date: creation_date,
            root: MetRegion::EML_NAME => |elem| MetRegion::read_eml(elem)?,
        });

        if tree.root.key.category != RegionCategory::State {
            report_tree_error(elem, MasterElectionTreeError::RootNotState(tree.root.key))?;
        }

        Ok(tree)
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr("CreationDate", &self.creation_date.raw())?
            .child_elem(MetRegion::EML_NAME, &self.root)?
            .finish()
    }
}

/// A region within a [`MasterElectionTree`].
///
/// Unlike [`Region`](crate::common::Region), which appears in the flat
/// election tree of an EML_NL document, this type nests its subregions
/// directly as children, rather than referring to a superior region
/// by `RegionKey`.
#[derive(Debug, Clone)]
pub struct MetRegion {
    /// The name of the region.
    pub name: Box<str>,
    /// The key identifying this region, its category and number.
    pub key: RegionKey,
    /// Whether this region uses roman numerals for the contest identifier in
    /// other parts of the EML_NL spec.
    ///
    /// Note: this only happens within Limburg
    pub roman_numerals: bool,
    /// This region allows exporting in the Frysian language.
    pub frysian_export_allowed: bool,

    /// The councils elected within this region.
    pub councils: Vec<Council>,
    /// The committees active within this region.
    pub committees: Vec<MetCommittee>,
    /// The subregions of this region.
    pub subregions: Vec<MetRegion>,
}

impl MetRegion {
    /// Create a new region.
    pub fn new(region_name: impl Into<Box<str>>, region_category: RegionCategory) -> Self {
        MetRegion {
            name: region_name.into(),
            key: RegionKey::new(region_category, None),
            roman_numerals: false,
            frysian_export_allowed: false,
            councils: Vec::new(),
            committees: Vec::new(),
            subregions: Vec::new(),
        }
    }

    /// Set the `RegionNumber` attribute of the `MetRegion` element.
    pub fn with_number(mut self, region_number: u16) -> Self {
        self.key.number = Some(region_number);
        self
    }

    /// Set the `RomanNumerals` attribute of the `MetRegion` element.
    pub fn with_roman_numerals(mut self, roman_numerals: bool) -> Self {
        self.roman_numerals = roman_numerals;
        self
    }

    /// Set the `FrysianExportAllowed` attribute of the `MetRegion` element.
    pub fn with_frysian_export_allowed(mut self, enabled: bool) -> Self {
        self.frysian_export_allowed = enabled;
        self
    }

    /// Set the `Council` elements of the `MetRegion` element.
    ///
    /// Note: this will replace any existing councils in the region.
    pub fn with_councils(mut self, councils: Vec<Council>) -> Self {
        self.councils = councils;
        self
    }

    /// Add a `Council` element to the `MetRegion` element.
    pub fn push_council(&mut self, council: Council) {
        self.councils.push(council);
    }

    /// Set the `Committee` elements of the `MetRegion` element.
    ///
    /// Note: this will replace any existing committees in the region.
    pub fn with_committees(mut self, committees: Vec<MetCommittee>) -> Self {
        self.committees = committees;
        self
    }

    /// Add a `Committee` element to the `MetRegion` element.
    pub fn push_committee(&mut self, committee: MetCommittee) {
        self.committees.push(committee);
    }

    /// Set the child `MetRegion` elements of the `MetRegion` element.
    ///
    /// Note: this will replace any existing child regions.
    pub fn with_subregions(mut self, regions: Vec<MetRegion>) -> Self {
        self.subregions = regions;
        self
    }

    /// Add a child `MetRegion` element to the `MetRegion` element.
    pub fn push_subregion(&mut self, region: MetRegion) {
        self.subregions.push(region);
    }
}

impl EMLElement for MetRegion {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Region", None);

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

        let region = collect_struct!(elem, MetRegion {
            key: RegionKey::new(category, number),
            roman_numerals: roman_numerals,
            frysian_export_allowed: frysian_export_allowed,
            name: ("RegionName", None) => |elem| elem.text_without_children()?,
            councils as Vec: Council::EML_NAME => |elem| Council::read_eml(elem)?,
            committees as Vec: MetCommittee::EML_NAME => |elem| MetCommittee::read_eml(elem)?,
            subregions as Vec: MetRegion::EML_NAME => |elem| MetRegion::read_eml(elem)?,
        });

        if region.key.category != RegionCategory::State && region.key.number.is_none() {
            report_tree_error(
                elem,
                MasterElectionTreeError::MissingRegionNumber(region.key.category),
            )?;
        }

        let mut seen_subregions = HashSet::new();
        for subregion in &region.subregions {
            if !region
                .key
                .category
                .is_higher_level_than(subregion.key.category)
            {
                report_tree_error(
                    elem,
                    MasterElectionTreeError::InvalidSubregionCategory {
                        region: region.key,
                        subregion: subregion.key,
                    },
                )?;
            }

            if !seen_subregions.insert(subregion.key) {
                report_tree_error(
                    elem,
                    MasterElectionTreeError::DuplicateSubregion {
                        region: region.key,
                        subregion: subregion.key,
                    },
                )?;
            }
        }

        let mut seen_councils = HashSet::new();
        for council in &region.councils {
            if !seen_councils.insert(council.election_category) {
                report_tree_error(
                    elem,
                    MasterElectionTreeError::DuplicateCouncil {
                        region: region.key,
                        election_category: council.election_category,
                    },
                )?;
            }
        }

        Ok(region)
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr_opt(
                "RegionNumber",
                self.key.number.map(|number| number.to_string()),
            )?
            .attr("RegionCategory", self.key.category.to_eml_value())?
            .attr_opt("RomanNumerals", self.roman_numerals.then_some("true"))?
            .attr_opt(
                "FrysianExportAllowed",
                self.frysian_export_allowed.then_some("true"),
            )?
            .content()?
            .child(("RegionName", None), |writer| {
                writer.text(self.name.as_ref())?.finish()
            })?
            .child_elems(Council::EML_NAME, &self.councils)?
            .child_elems(MetCommittee::EML_NAME, &self.committees)?
            .child_elems(MetRegion::EML_NAME, &self.subregions)?
            .finish()
    }
}

/// A council elected within a [`MetRegion`] of a [`MasterElectionTree`], for a
/// specific election category.
#[derive(Debug, Clone)]
pub struct Council {
    /// The election category this council is elected for.
    pub election_category: ElectionCategory,
    /// The name of the council, if not the standard name for the election category.
    pub name: Option<Box<str>>,
    /// The number of seats in the council.
    pub number_of_seats: Option<StringValue<u64>>,
    /// The electoral quota, in percent, at which a candidate is preferred.
    pub preference_threshold: Option<StringValue<u64>>,
}

impl Council {
    /// Create a new council.
    pub fn new(election_category: ElectionCategory) -> Self {
        Council {
            election_category,
            name: None,
            number_of_seats: None,
            preference_threshold: None,
        }
    }

    /// Set the `CouncilName` attribute of the `Council` element.
    pub fn with_name(mut self, name: impl Into<Box<str>>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the `NumberOfSeats` attribute of the `Council` element.
    pub fn with_number_of_seats(mut self, number_of_seats: u64) -> Self {
        self.number_of_seats = Some(StringValue::from_value(number_of_seats));
        self
    }

    /// Set the `PreferenceThreshold` attribute of the `Council` element.
    pub fn with_preference_threshold(mut self, preference_threshold: u64) -> Self {
        self.preference_threshold = Some(StringValue::from_value(preference_threshold));
        self
    }
}

impl EMLElement for Council {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Council", None);

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(Council {
            election_category: ElectionCategory::new(
                elem.attribute_value_req("ElectionCategory")?,
            )?,
            name: elem.attribute_value("CouncilName")?.map(|name| name.into()),
            number_of_seats: elem.string_value_attr_opt("NumberOfSeats")?,
            preference_threshold: elem.string_value_attr_opt("PreferenceThreshold")?,
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr_opt(
                "NumberOfSeats",
                self.number_of_seats.as_ref().map(|v| v.raw()),
            )?
            .attr("ElectionCategory", self.election_category.to_eml_value())?
            .attr_opt("CouncilName", self.name.as_ref())?
            .attr_opt(
                "PreferenceThreshold",
                self.preference_threshold.as_ref().map(|v| v.raw()),
            )?
            .empty()
    }
}

/// A committee active within a [`MetRegion`] of a [`MasterElectionTree`], for a
/// specific election category.
///
/// This wraps [`Committee`], which appears in the flat election tree of an EML_NL
/// document, and adds the election category it is active for.
#[derive(Debug, Clone)]
pub struct MetCommittee {
    /// The election category this committee is active for.
    pub election_category: ElectionCategory,

    /// The committee itself.
    pub committee: Committee,
}

impl MetCommittee {
    /// Create a new committee.
    pub fn new(election_category: ElectionCategory, category: CommitteeCategory) -> Self {
        MetCommittee {
            election_category,
            committee: Committee::new(category),
        }
    }

    /// Set the `CommitteeName` attribute of the `Committee` element.
    pub fn with_name(mut self, name: impl Into<Box<str>>) -> Self {
        self.committee = self.committee.with_name(name);
        self
    }

    /// Set the `AcceptCentralSubmissions` attribute of the `Committee` element.
    pub fn with_accept_central_submissions(mut self, accept: bool) -> Self {
        self.committee = self.committee.with_accept_central_submissions(accept);
        self
    }
}

impl EMLElement for MetCommittee {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Committee", None);

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(MetCommittee {
            election_category: ElectionCategory::new(
                elem.attribute_value_req("ElectionCategory")?,
            )?,
            committee: crate::common::Committee::read_eml(elem)?,
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr("CommitteeCategory", self.committee.category.to_eml_value())?
            .attr("ElectionCategory", self.election_category.to_eml_value())?
            .attr_opt("CommitteeName", self.committee.name.as_ref())?
            .attr_opt(
                "AcceptCentralSubmissions",
                self.committee
                    .accept_central_submissions
                    .map(|value| value.to_string()),
            )?
            .empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{EMLParsingMode, EMLRead as _, test_write_eml_element, test_xml_fragment};

    #[test]
    fn test_committee_parsing() {
        let xml = test_xml_fragment(
            r#"<Committee CommitteeCategory="HSB" ElectionCategory="EP" CommitteeName="De Kiesraad" AcceptCentralSubmissions="true"/>"#,
        );
        let committee = MetCommittee::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(committee.election_category, ElectionCategory::EP);
        assert_eq!(committee.committee.category, CommitteeCategory::HSB);
        assert_eq!(committee.committee.name, Some("De Kiesraad".into()));
        assert_eq!(committee.committee.accept_central_submissions, Some(true));

        let xml_output = test_write_eml_element(&committee, &[]).unwrap();
        assert_eq!(xml_output, xml);
    }

    #[test]
    fn test_council_parsing() {
        let xml = test_xml_fragment(
            r#"<Council NumberOfSeats="150" ElectionCategory="TK" PreferenceThreshold="25"/>"#,
        );
        let council = Council::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(council.election_category, ElectionCategory::TK);
        assert_eq!(council.name, None);
        assert_eq!(council.number_of_seats, Some(StringValue::Parsed(150)));
        assert_eq!(council.preference_threshold, Some(StringValue::Parsed(25)));

        let xml_output = test_write_eml_element(&council, &[]).unwrap();
        assert_eq!(xml_output, xml);
    }

    #[test]
    fn test_region_parsing() {
        let xml = test_xml_fragment(
            r#"
            <Region RegionNumber="1" RegionCategory="PROVINCIE">
                <RegionName>Groningen</RegionName>
                <Committee CommitteeCategory="PROV_SB" ElectionCategory="EK"/>
                <Region RegionNumber="14" RegionCategory="GEMEENTE">
                    <RegionName>Groningen</RegionName>
                    <Committee CommitteeCategory="CSB" ElectionCategory="GR"/>
                </Region>
            </Region>
            "#,
        );

        let region = MetRegion::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(region.name.as_ref(), "Groningen");
        assert_eq!(region.key, RegionKey::province(1));
        assert!(!region.roman_numerals);
        assert!(!region.frysian_export_allowed);
        assert!(region.councils.is_empty());
        assert_eq!(region.committees.len(), 1);
        assert_eq!(
            region.committees[0].committee.category,
            CommitteeCategory::ProvSB
        );
        assert_eq!(region.subregions.len(), 1);
        assert_eq!(region.subregions[0].name.as_ref(), "Groningen");
        assert_eq!(region.subregions[0].key, RegionKey::municipality(14));

        let xml_output = test_write_eml_element(&region, &[]).unwrap();
        pretty_assertions::assert_eq!(xml_output, xml);
    }

    #[test]
    fn test_region_rejects_subregion_of_invalid_category() {
        // A GEMEENTE cannot contain a PROVINCIE: subregions must always sit at
        // a lower level in the election tree than their parent region.
        let xml = test_xml_fragment(
            r#"
            <Region RegionNumber="14" RegionCategory="GEMEENTE">
                <RegionName>Groningen</RegionName>
                <Region RegionNumber="1" RegionCategory="PROVINCIE">
                    <RegionName>Groningen</RegionName>
                </Region>
            </Region>
            "#,
        );

        let error = MetRegion::parse_eml(&xml, EMLParsingMode::Strict)
            .ok()
            .unwrap_err();
        assert!(matches!(
            error.kind(),
            EMLErrorKind::InvalidMasterElectionTree(
                MasterElectionTreeError::InvalidSubregionCategory { .. }
            )
        ));
    }

    #[test]
    fn test_region_rejects_missing_region_number() {
        let xml = test_xml_fragment(
            r#"
            <Region RegionCategory="GEMEENTE">
                <RegionName>Groningen</RegionName>
            </Region>
            "#,
        );

        let error = MetRegion::parse_eml(&xml, EMLParsingMode::Strict)
            .ok()
            .unwrap_err();
        assert!(matches!(
            error.kind(),
            EMLErrorKind::InvalidMasterElectionTree(MasterElectionTreeError::MissingRegionNumber(
                RegionCategory::Municipality
            ))
        ));
    }

    #[test]
    fn test_region_rejects_duplicate_subregion() {
        let xml = test_xml_fragment(
            r#"
            <Region RegionNumber="1" RegionCategory="PROVINCIE">
                <RegionName>Groningen</RegionName>
                <Region RegionNumber="14" RegionCategory="GEMEENTE">
                    <RegionName>Groningen</RegionName>
                </Region>
                <Region RegionNumber="14" RegionCategory="GEMEENTE">
                    <RegionName>Groningen</RegionName>
                </Region>
            </Region>
            "#,
        );

        let error = MetRegion::parse_eml(&xml, EMLParsingMode::Strict)
            .ok()
            .unwrap_err();
        assert!(matches!(
            error.kind(),
            EMLErrorKind::InvalidMasterElectionTree(
                MasterElectionTreeError::DuplicateSubregion { .. }
            )
        ));
    }

    #[test]
    fn test_region_rejects_duplicate_council() {
        let xml = test_xml_fragment(
            r#"
            <Region RegionCategory="STAAT">
                <RegionName>Nederland</RegionName>
                <Council NumberOfSeats="150" ElectionCategory="TK"/>
                <Council NumberOfSeats="151" ElectionCategory="TK"/>
            </Region>
            "#,
        );

        let error = MetRegion::parse_eml(&xml, EMLParsingMode::Strict)
            .ok()
            .unwrap_err();
        assert!(matches!(
            error.kind(),
            EMLErrorKind::InvalidMasterElectionTree(
                MasterElectionTreeError::DuplicateCouncil { .. }
            )
        ));
    }

    #[test]
    fn test_master_election_tree_rejects_non_state_root() {
        let xml = test_xml_fragment(
            r#"
            <MasterElectionTree CreationDate="2025-07-16T06:24:52+00:00">
                <Region RegionNumber="1" RegionCategory="PROVINCIE">
                    <RegionName>Groningen</RegionName>
                </Region>
            </MasterElectionTree>
            "#,
        );

        let error = MasterElectionTree::parse_eml(&xml, EMLParsingMode::Strict)
            .ok()
            .unwrap_err();
        assert!(matches!(
            error.kind(),
            EMLErrorKind::InvalidMasterElectionTree(MasterElectionTreeError::RootNotState(_))
        ));
    }

    #[test]
    fn test_master_election_tree_parsing() {
        let xml = test_xml_fragment(
            r#"
            <MasterElectionTree CreationDate="2025-07-16T06:24:52+00:00">
                <Region RegionCategory="STAAT">
                    <RegionName>Nederland</RegionName>
                    <Council NumberOfSeats="150" ElectionCategory="TK"/>
                </Region>
            </MasterElectionTree>
            "#,
        );

        let tree = MasterElectionTree::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(tree.creation_date.raw(), "2025-07-16T06:24:52+00:00");
        assert_eq!(tree.root.name.as_ref(), "Nederland");
        assert_eq!(tree.root.key, RegionKey::state());
        assert_eq!(tree.root.councils.len(), 1);
        assert_eq!(
            tree.root.councils[0].election_category,
            ElectionCategory::TK
        );

        let xml_output = tree.write_str(true, false).unwrap();
        pretty_assertions::assert_eq!(xml_output, xml);
    }

    #[test]
    fn test_master_election_tree_full_fixture() {
        let xml = include_str!("../../test-files/master_election_tree/MasterElectionTree.xml");

        let tree = MasterElectionTree::parse_eml(xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(tree.root.name.as_ref(), "Nederland");
        assert_eq!(tree.root.key, RegionKey::state());
        assert_eq!(tree.root.councils.len(), 3);
        assert_eq!(tree.root.committees.len(), 2);
        // 12 provinces, 21 waterschappen, 4 kiescolleges and 1 top-level kieskring (Bonaire)
        assert_eq!(tree.root.subregions.len(), 38);

        fn count_regions(region: &MetRegion) -> usize {
            1 + region.subregions.iter().map(count_regions).sum::<usize>()
        }
        assert_eq!(count_regions(&tree.root), 909);

        let bonaire_kiescollege = tree
            .root
            .subregions
            .iter()
            .find(|r| r.key == RegionKey::new(RegionCategory::ElectoralCollege, Some(13)))
            .unwrap();
        assert_eq!(bonaire_kiescollege.name.as_ref(), "Bonaire");

        let bonaire_kieskring = tree
            .root
            .subregions
            .iter()
            .find(|r| r.key == RegionKey::electoral_district(20))
            .unwrap();
        assert_eq!(bonaire_kieskring.name.as_ref(), "Bonaire");
        assert_eq!(bonaire_kieskring.subregions.len(), 3);
        assert_eq!(
            bonaire_kieskring.subregions[0].key,
            RegionKey::new(RegionCategory::IslandMunicipality, Some(9001))
        );
    }

    #[test]
    fn test_master_election_tree_round_trip() {
        // Differences compared to the "real" master election tree:
        // - Shortened by removing many regions
        // - Uses spaces instead of tabs
        // - RegionNumbers are not prefixed with 0s
        // - Order of RegionNumber and RegionCategory of PROVINCIAAL_STEMBUREAU has been made consistent
        // - xsi properties have been removed from MasterElectionTree element
        // - CreationDate uses +00:00 as timezone specification
        let xml = include_str!("../../test-files/master_election_tree/met_test.xml");

        let tree = MasterElectionTree::parse_eml(xml, EMLParsingMode::Strict).unwrap();
        let xml_output = tree.write_str(true, true).unwrap();
        pretty_assertions::assert_eq!(xml_output, xml);
    }
}
