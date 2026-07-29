use std::collections::{HashMap, HashSet, VecDeque};

use thiserror::Error;

use crate::common::{ElectionTree, Region, RegionKey};
use crate::{EMLError, EMLErrorKind};

/// Error returned when the regions of an election tree do not describe a valid
/// tree, or when an operation on an [`ElectionTreeHierarchy`] cannot be
/// performed.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ElectionTreeHierarchyError {
    /// None of the regions is a root region, so the tree has no starting point.
    #[error("Election tree has no root region")]
    NoRoot,

    /// More than one region is a root region, while an election tree must have
    /// exactly one.
    #[error("Election tree has more than one root region: {0:?}")]
    MultipleRoots(Vec<RegionKey>),

    /// Two or more regions share the same category and number.
    #[error("Election tree has more than one region with identity {0:?}")]
    DuplicateRegion(RegionKey),

    /// A region refers to a superior region that does not exist.
    #[error("Region {region:?} refers to unknown superior region {superior:?}")]
    UnknownSuperiorRegion {
        /// The region containing the unresolvable reference.
        region: RegionKey,
        /// The superior region that was referred to.
        superior: RegionKey,
    },

    /// A region refers to a superior region that does not sit at a higher level
    /// in the election tree, while every region must be subordinate to a region
    /// of a higher category than its own.
    #[error(
        "Region {region:?} refers to superior region {superior:?}, which does not sit at a higher level in the election tree"
    )]
    InvalidSuperiorRegionCategory {
        /// The region containing the invalid reference.
        region: RegionKey,
        /// The superior region that was referred to.
        superior: RegionKey,
    },

    /// A region sits below a region it does not refer to as its superior region,
    /// while the position of a region within the election tree and the superior
    /// region it refers to have to agree.
    #[error(
        "Region {region:?} sits below region {superior:?}, but does not refer to it as its superior region"
    )]
    InconsistentSuperiorRegion {
        /// The region referring to the wrong superior region.
        region: RegionKey,
        /// The region it sits below.
        superior: RegionKey,
    },

    /// The election tree does not contain a region with the given identity.
    #[error("Election tree has no region with identity {0:?}")]
    UnknownRegion(RegionKey),

    /// The root region was removed, while an election tree must always have
    /// exactly one root region.
    #[error("The root region of an election tree cannot be removed")]
    CannotRemoveRoot,
}

impl From<ElectionTreeHierarchyError> for EMLError {
    fn from(err: ElectionTreeHierarchyError) -> Self {
        EMLErrorKind::InvalidElectionTree(err).without_span()
    }
}

/// A region within an [`ElectionTreeHierarchy`], together with the regions
/// directly subordinate to it.
///
/// The superior region key of the contained [`Region`] should match the position
/// of the region within the tree: it is set when the region is attached to a
/// superior region by [`RegionNode::add_child`], and cleared for the root region
/// of the tree.
#[derive(Debug, Clone)]
pub struct RegionNode {
    region: Region,
    children: Vec<RegionNode>,
}

impl RegionNode {
    /// Create a new region node for a region and its subordinate regions.
    pub fn new(region: Region, children: impl Into<Vec<RegionNode>>) -> Self {
        RegionNode {
            region,
            children: children.into(),
        }
    }

    /// The region itself, including its committees.
    pub fn region(&self) -> &Region {
        &self.region
    }

    /// The region itself, allowing it to be modified.
    ///
    /// The key and the superior region key of the region are part of the
    /// structure of an election tree, so keeping them pointing at the regions
    /// around this one is up to the caller. Both are checked again when this
    /// region node enters an election tree hierarchy.
    pub fn region_mut(&mut self) -> &mut Region {
        &mut self.region
    }

    /// The regions directly subordinate to this region, in document order.
    pub fn children(&self) -> &[RegionNode] {
        &self.children
    }

    /// The key identifying this region.
    pub fn key(&self) -> RegionKey {
        self.region.key
    }

    /// This region and all regions below it, breadth-first, starting with this
    /// region itself.
    pub fn iter(&self) -> impl Iterator<Item = &RegionNode> {
        // The queue holds the regions that have been reached but not yielded
        // yet, so only the region to start from has to be queued up front: every
        // region queues the regions below it as it is yielded.
        let mut queue = VecDeque::from([self]);

        std::iter::from_fn(move || {
            let node = queue.pop_front()?;
            queue.extend(node.children.iter());

            Some(node)
        })
    }

    /// Find the region with the given key, either this region itself or one of
    /// the regions below it.
    pub fn find(&self, key: RegionKey) -> Option<&RegionNode> {
        self.iter().find(|node| node.key() == key)
    }

    /// Find the region with the given key, allowing it to be modified.
    pub fn find_mut(&mut self, key: RegionKey) -> Option<&mut RegionNode> {
        let mut stack = vec![self];

        while let Some(node) = stack.pop() {
            if node.key() == key {
                return Some(node);
            }

            stack.extend(node.children.iter_mut());
        }

        None
    }

    /// Detach the region with the given key from the regions below this region,
    /// returning it together with all regions below it.
    pub fn take_descendant(&mut self, key: RegionKey) -> Option<RegionNode> {
        let mut stack = vec![self];

        while let Some(node) = stack.pop() {
            let position = node.children.iter().position(|child| child.key() == key);

            if let Some(position) = position {
                return Some(node.children.remove(position));
            }

            stack.extend(node.children.iter_mut());
        }

        None
    }

    /// Add a region below this region, setting the superior region key of the
    /// added region accordingly.
    ///
    /// The region being added must be of a lower region category than this
    /// region. The regions below the added region are left as they are, so it is
    /// only once the region enters an election tree that they are checked.
    pub fn add_child(
        &mut self,
        mut child: RegionNode,
    ) -> Result<RegionKey, ElectionTreeHierarchyError> {
        let child_key = child.key();
        let parent_key = self.key();
        if !parent_key.category.is_higher_level_than(child_key.category) {
            return Err(ElectionTreeHierarchyError::InvalidSuperiorRegionCategory {
                region: child_key,
                superior: parent_key,
            });
        }

        child.region.superior_region_key = Some(parent_key);
        self.children.push(child);

        Ok(child_key)
    }

    /// Verify that this region and the regions below it describe a valid part of
    /// an election tree: every region below this one refers to the region it sits
    /// below as its superior region and is of a lower region category, and no
    /// region appears more than once.
    ///
    /// The superior region key of this region itself is not checked, since where
    /// this region sits is not known here.
    fn check_subtree(
        &self,
        keys: &mut HashSet<RegionKey>,
    ) -> Result<(), ElectionTreeHierarchyError> {
        let mut stack = vec![self];

        while let Some(node) = stack.pop() {
            if !keys.insert(node.key()) {
                return Err(ElectionTreeHierarchyError::DuplicateRegion(node.key()));
            }

            let superior = node.key();
            for child in &node.children {
                let key = child.key();

                if !superior.category.is_higher_level_than(key.category) {
                    return Err(ElectionTreeHierarchyError::InvalidSuperiorRegionCategory {
                        region: key,
                        superior,
                    });
                }

                if child.region.superior_region_key != Some(superior) {
                    return Err(ElectionTreeHierarchyError::InconsistentSuperiorRegion {
                        region: key,
                        superior,
                    });
                }
            }

            stack.extend(node.children.iter());
        }

        Ok(())
    }
}

impl From<Region> for RegionNode {
    fn from(region: Region) -> Self {
        RegionNode::new(region, vec![])
    }
}

/// The election tree of an EML_NL election definition in a structured form.
///
/// EML_NL encodes the election tree as a flat list of regions, where every
/// region refers to its superior region by category and number. This type
/// exposes that same data as an actual tree with a single root region, and can
/// be converted back into the flat [`ElectionTree`] representation.
///
/// Every region below the root region is of a lower
/// [`RegionCategory`](crate::utils::RegionCategory) than its superior region,
/// which is what keeps the flat list of regions from describing anything other
/// than a tree.
///
/// Use [`ElectionTree::hierarchy`] or [`TryFrom`] to build a tree from an
/// [`ElectionTree`], and [`ElectionTreeHierarchy::flattened`] or [`From`] to
/// convert a tree back into an [`ElectionTree`].
#[derive(Debug, Clone)]
pub struct ElectionTreeHierarchy {
    root: RegionNode,
}

impl ElectionTreeHierarchy {
    /// Create an election tree consisting of a single root region.
    ///
    /// Any superior region key on the given region is cleared, since the root
    /// region of an election tree has no superior region. The root
    /// region does not have to be a `STAAT`, it can be any region without a
    /// superior region, such as the `GEMEENTE` of a municipal election.
    pub fn new(root: Region) -> Self {
        let mut root = RegionNode::from(root);
        root.region.superior_region_key = None;

        ElectionTreeHierarchy { root }
    }

    /// Create an election tree from a region and the regions below it, checking
    /// that the tree invariants are satisfied.
    pub fn from_region_node(
        node: impl Into<RegionNode>,
    ) -> Result<Self, ElectionTreeHierarchyError> {
        let node: RegionNode = node.into();
        node.try_into()
    }

    /// The root region of this election tree.
    pub fn root(&self) -> &RegionNode {
        &self.root
    }

    /// The total number of regions in this election tree.
    pub fn region_count(&self) -> usize {
        self.iter().count()
    }

    /// All regions in this election tree, breadth-first. The root region is
    /// yielded first.
    pub fn iter(&self) -> impl Iterator<Item = &RegionNode> {
        self.root.iter()
    }

    /// Find the region with the given key.
    pub fn get(&self, key: RegionKey) -> Option<&RegionNode> {
        self.root.find(key)
    }

    /// Whether this election tree contains a region with the given key.
    pub fn contains(&self, key: RegionKey) -> bool {
        self.get(key).is_some()
    }

    /// Attach a region below the region identified by `parent`, setting the
    /// superior region key of the inserted region accordingly.
    ///
    /// The region being inserted must be of a lower region category than the
    /// region it is attached to, since a region is always subordinate to a
    /// region at a higher level of the election tree.
    ///
    /// The regions below the region being inserted are inserted along with it,
    /// and have to describe a valid part of an election tree themselves: each of
    /// them refers to the region it sits below as its superior region and is of a
    /// lower region category, and no region ends up in the tree twice.
    ///
    /// Returns the key identifying the inserted region.
    pub fn insert(
        &mut self,
        parent: RegionKey,
        region: impl Into<RegionNode>,
    ) -> Result<RegionKey, ElectionTreeHierarchyError> {
        let node = region.into();
        let key = node.key();

        if !parent.category.is_higher_level_than(key.category) {
            return Err(ElectionTreeHierarchyError::InvalidSuperiorRegionCategory {
                region: key,
                superior: parent,
            });
        }

        self.check_can_insert(&node)?;

        let Some(parent_node) = self.root.find_mut(parent) else {
            return Err(ElectionTreeHierarchyError::UnknownRegion(parent));
        };

        parent_node.add_child(node)
    }

    /// Detach the region identified by `key` together with all regions below
    /// it, and return it.
    ///
    /// The root region cannot be removed, since an election tree always has
    /// exactly one root region.
    pub fn remove(&mut self, key: RegionKey) -> Result<RegionNode, ElectionTreeHierarchyError> {
        if key == self.root.key() {
            return Err(ElectionTreeHierarchyError::CannotRemoveRoot);
        }

        self.root
            .take_descendant(key)
            .ok_or(ElectionTreeHierarchyError::UnknownRegion(key))
    }

    /// Verify that the given region and the regions below it can be added to this
    /// election tree without breaking its invariants.
    fn check_can_insert(&self, node: &RegionNode) -> Result<(), ElectionTreeHierarchyError> {
        node.check_subtree(&mut self.iter().map(|node| node.key()).collect())
    }

    /// This election tree in the flat form EML_NL uses, with every region
    /// referring to its superior region by category and number.
    ///
    /// The regions are listed breadth-first.
    pub fn flattened(&self) -> ElectionTree {
        ElectionTree::from(self)
    }

    /// Consume the tree and retrieve the root node, allowing manipulation
    /// on the node level.
    pub fn into_inner(self) -> RegionNode {
        self.root
    }

    /// Build a structured election tree from the flat list of regions of an
    /// [`ElectionTree`].
    ///
    /// The checks performed while indexing the regions guarantee that the result
    /// is a tree covering every region: keys are unique, exactly one region has
    /// no superior region, every other region refers to a superior region that
    /// exists and that sits at a strictly higher level.
    fn from_regions(regions: &[Region]) -> Result<Self, ElectionTreeHierarchyError> {
        let (root, index) = index_regions(regions)?;
        let tree = ElectionTreeHierarchy {
            root: assemble(&index, root),
        };

        Ok(tree)
    }
}

impl TryFrom<RegionNode> for ElectionTreeHierarchy {
    type Error = ElectionTreeHierarchyError;

    /// Build an election tree around the given region and the regions below it,
    /// checking the tree invariants for the entire tree.
    fn try_from(mut root: RegionNode) -> Result<Self, Self::Error> {
        root.check_subtree(&mut HashSet::new())?;
        root.region.superior_region_key = None;

        Ok(ElectionTreeHierarchy { root })
    }
}

impl TryFrom<&ElectionTree> for ElectionTreeHierarchy {
    type Error = ElectionTreeHierarchyError;

    fn try_from(tree: &ElectionTree) -> Result<Self, Self::Error> {
        ElectionTreeHierarchy::from_regions(&tree.regions)
    }
}

impl TryFrom<ElectionTree> for ElectionTreeHierarchy {
    type Error = ElectionTreeHierarchyError;

    fn try_from(tree: ElectionTree) -> Result<Self, Self::Error> {
        ElectionTreeHierarchy::from_regions(&tree.regions)
    }
}

impl From<&ElectionTreeHierarchy> for ElectionTree {
    fn from(tree: &ElectionTreeHierarchy) -> Self {
        let regions: Vec<Region> = tree.iter().map(|node| node.region.clone()).collect();

        ElectionTree::new(regions)
    }
}

impl From<ElectionTreeHierarchy> for ElectionTree {
    fn from(tree: ElectionTreeHierarchy) -> Self {
        let mut regions = Vec::new();
        let mut queue = VecDeque::from([tree.root]);

        while let Some(node) = queue.pop_front() {
            regions.push(node.region);
            queue.extend(node.children);
        }

        ElectionTree::new(regions)
    }
}

/// The regions of an election tree by key, each together with the keys of the
/// regions directly subordinate to it in document order.
type RegionIndex<'a> = HashMap<RegionKey, (&'a Region, Vec<RegionKey>)>;

/// Index the given regions by their key, resolving the superior region of every
/// region, and return the key of the root region along with the index.
fn index_regions(
    regions: &[Region],
) -> Result<(RegionKey, RegionIndex<'_>), ElectionTreeHierarchyError> {
    let mut index: RegionIndex<'_> = HashMap::with_capacity(regions.len());
    let mut roots = Vec::new();

    for region in regions {
        if index.insert(region.key, (region, Vec::new())).is_some() {
            return Err(ElectionTreeHierarchyError::DuplicateRegion(region.key));
        }
    }

    for region in regions {
        let Some(superior) = region.superior_region_key else {
            roots.push(region.key);
            continue;
        };

        if !superior.category.is_higher_level_than(region.key.category) {
            return Err(ElectionTreeHierarchyError::InvalidSuperiorRegionCategory {
                region: region.key,
                superior,
            });
        }

        let Some((_, subordinates)) = index.get_mut(&superior) else {
            return Err(ElectionTreeHierarchyError::UnknownSuperiorRegion {
                region: region.key,
                superior,
            });
        };

        subordinates.push(region.key);
    }

    match roots.as_slice() {
        [root] => Ok((*root, index)),
        [] => Err(ElectionTreeHierarchyError::NoRoot),
        _ => Err(ElectionTreeHierarchyError::MultipleRoots(roots)),
    }
}

/// Build the node for the region with the given key, together with the nodes of
/// all regions below it.
///
/// This recurses once per level of the election tree. Every region sits below a
/// region of a higher category, which [`index_regions`] has already verified, so
/// the levels of a tree are limited to the number of region categories that
/// exist, however many regions are being assembled.
fn assemble(index: &RegionIndex<'_>, key: RegionKey) -> RegionNode {
    let (region, subordinates) = &index[&key];

    RegionNode::new(
        (*region).clone(),
        subordinates
            .iter()
            .map(|&child| assemble(index, child))
            .collect::<Vec<_>>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::election_definition::ElectionDefinition;
    use crate::io::{EMLParsingMode, EMLRead};
    use crate::utils::{CommitteeCategory, RegionCategory};

    /// The number of regions in the TK2025 election definition: one `STAAT`,
    /// twenty `KIESKRING` regions and 346 `GEMEENTE` regions.
    const TK2025_REGION_COUNT: usize = 367;

    /// The `GEMEENTE` regions of the TK2025 election definition that allow
    /// exporting in Frysian, in document order. These are exactly the
    /// municipalities of the Leeuwarden electoral district.
    const TK2025_FRYSIAN_MUNICIPALITIES: [u16; 18] = [
        59, 60, 72, 74, 80, 85, 86, 88, 90, 93, 96, 98, 737, 1891, 1900, 1940, 1949, 1970,
    ];

    /// The election tree of the TK2025 election definition, in its flat EML_NL
    /// form.
    fn tk2025_election_tree() -> ElectionTree {
        let xml = include_str!(
            "../../test-files/election_definition/Verkiezingsdefinitie_TK2025.eml.xml"
        );
        let definition = ElectionDefinition::parse_eml(xml, EMLParsingMode::Strict).unwrap();

        definition.election_event.election.election_tree
    }

    /// The election tree of the TK2025 election definition, in its structured
    /// form.
    fn tk2025_hierarchy() -> ElectionTreeHierarchy {
        tk2025_election_tree().hierarchy().unwrap()
    }

    #[test]
    fn test_tk2025_root_region() {
        let tree = tk2025_hierarchy();
        let root = tree.root();

        assert_eq!(root.key(), RegionKey::state());
        assert_eq!(root.region().name.as_ref(), "Nederland");
        assert_eq!(root.region().superior_region_key, None);
        assert!(root.region().committees.is_empty());
        assert_eq!(tree.region_count(), TK2025_REGION_COUNT);
    }

    #[test]
    fn test_tk2025_electoral_districts_are_below_the_state() {
        let tree = tk2025_hierarchy();

        // All twenty electoral districts are directly below the root region, in
        // document order, and each refers back to it.
        let districts: Vec<RegionKey> = tree
            .root()
            .children()
            .iter()
            .map(|node| node.key())
            .collect();
        let expected: Vec<RegionKey> = (1..=20).map(RegionKey::electoral_district).collect();
        assert_eq!(districts, expected);

        for node in tree.root().children() {
            assert_eq!(node.region().superior_region_key, Some(RegionKey::state()));
        }

        let names: Vec<&str> = tree
            .root()
            .children()
            .iter()
            .map(|node| node.region().name.as_ref())
            .collect();
        assert_eq!(names[0], "Groningen");
        assert_eq!(names[1], "Leeuwarden");
        assert_eq!(names[11], "'s-Gravenhage");
        assert_eq!(names[19], "Bonaire");
    }

    #[test]
    fn test_tk2025_municipalities_are_below_an_electoral_district() {
        let tree = tk2025_hierarchy();

        // Every municipality is a leaf region below the electoral district it
        // refers to as its superior region.
        let mut municipalities = 0;

        for node in tree.iter() {
            if node.region().key.category != RegionCategory::Municipality {
                continue;
            }

            municipalities += 1;
            assert!(node.children().is_empty(), "{:?}", node.key());
            assert_eq!(
                node.region().superior_region_key.map(|key| key.category),
                Some(RegionCategory::ElectoralDistrict),
                "{:?}",
                node.key()
            );
        }

        assert_eq!(municipalities, 346);

        // All of them sit two levels below the root region, directly below one of
        // the electoral districts.
        let below_the_districts = tree
            .root()
            .children()
            .iter()
            .flat_map(|district| district.children())
            .count();
        assert_eq!(below_the_districts, 346);

        // Spot check a few districts: Amsterdam and Rotterdam consist of a
        // single municipality, while Bonaire covers the three Caribbean ones.
        let amsterdam = tree.get(RegionKey::electoral_district(9)).unwrap();
        assert_eq!(amsterdam.children().len(), 1);
        assert_eq!(amsterdam.children()[0].key(), RegionKey::municipality(363));

        assert_eq!(
            tree.get(RegionKey::electoral_district(13))
                .unwrap()
                .children()
                .len(),
            1
        );

        let bonaire = tree.get(RegionKey::electoral_district(20)).unwrap();
        let caribbean: Vec<RegionKey> = bonaire.children().iter().map(|node| node.key()).collect();
        assert_eq!(
            caribbean,
            vec![
                RegionKey::municipality(9001),
                RegionKey::municipality(9002),
                RegionKey::municipality(9003)
            ]
        );
    }

    /// The number of regions at every level of the tree below the given region,
    /// starting with the region itself.
    fn regions_per_level(root: &RegionNode) -> Vec<usize> {
        let mut level = vec![root];
        let mut counts = Vec::new();

        while !level.is_empty() {
            counts.push(level.len());
            level = level.iter().flat_map(|node| node.children()).collect();
        }

        counts
    }

    #[test]
    fn test_tk2025_tree_is_three_levels_deep() {
        let tree = tk2025_hierarchy();

        assert_eq!(regions_per_level(tree.root()), vec![1, 20, 346]);
    }

    #[test]
    fn test_tk2025_breadth_first_order_matches_document_order() {
        let election_tree = tk2025_election_tree();
        let tree = ElectionTreeHierarchy::try_from(&election_tree).unwrap();

        // The TK2025 definition lists its regions level by level, which is the
        // order the tree yields them in as well.
        let document_order: Vec<RegionKey> = election_tree
            .regions
            .iter()
            .map(|region| region.key)
            .collect();
        let tree_order: Vec<RegionKey> = tree.iter().map(|node| node.key()).collect();

        assert_eq!(tree_order, document_order);
    }

    #[test]
    fn test_tk2025_region_lookup() {
        let tree = tk2025_hierarchy();

        let leeuwarden = tree.get(RegionKey::municipality(80)).unwrap();
        assert_eq!(leeuwarden.region().name.as_ref(), "Leeuwarden");
        assert_eq!(
            leeuwarden.region().superior_region_key,
            Some(RegionKey::electoral_district(2))
        );
        assert!(leeuwarden.region().frysian_export_allowed);
        assert!(!leeuwarden.region().roman_numerals);

        assert!(tree.contains(RegionKey::state()));
        assert!(tree.contains(RegionKey::electoral_district(20)));

        // A municipality number that is not part of the tree, and existing
        // numbers in the wrong category.
        assert!(tree.get(RegionKey::municipality(1)).is_none());
        assert!(!tree.contains(RegionKey::municipality(1)));
        assert!(!tree.contains(RegionKey::new(RegionCategory::Province, Some(2))));
        assert!(!tree.contains(RegionKey::new(RegionCategory::ElectoralDistrict, None)));
    }

    #[test]
    fn test_tk2025_committees_are_preserved() {
        let tree = tk2025_hierarchy();

        // 's-Gravenhage holds the central electoral committee alongside its own
        // main electoral committee.
        let the_hague = tree
            .get(RegionKey::electoral_district(12))
            .unwrap()
            .region();
        assert_eq!(the_hague.committees.len(), 2);
        assert_eq!(the_hague.committees[0].category, CommitteeCategory::CSB);
        assert_eq!(the_hague.committees[0].name.as_deref(), Some("De Kiesraad"));
        assert_eq!(the_hague.committees[1].category, CommitteeCategory::HSB);
        assert_eq!(
            the_hague.committees[1].accept_central_submissions,
            Some(true)
        );
    }

    #[test]
    fn test_tk2025_frysian_regions_are_the_leeuwarden_district() {
        let tree = tk2025_hierarchy();

        let frysian: Vec<RegionKey> = tree
            .iter()
            .filter(|node| node.region().frysian_export_allowed)
            .map(|node| node.key())
            .collect();
        let expected: Vec<RegionKey> = TK2025_FRYSIAN_MUNICIPALITIES
            .iter()
            .copied()
            .map(RegionKey::municipality)
            .collect();
        assert_eq!(frysian, expected);

        // The Leeuwarden electoral district itself does not allow Frysian
        // exports, but all of the municipalities below it do.
        let leeuwarden = tree.get(RegionKey::electoral_district(2)).unwrap();
        assert!(!leeuwarden.region().frysian_export_allowed);

        let below: Vec<RegionKey> = leeuwarden
            .children()
            .iter()
            .map(|node| node.key())
            .collect();
        assert_eq!(below, expected);
    }

    #[test]
    fn test_tk2025_node_iteration() {
        let tree = tk2025_hierarchy();
        let leeuwarden = tree.get(RegionKey::electoral_district(2)).unwrap();

        // Iterating a node starts at that node itself, followed by the regions
        // below it.
        let iterated: Vec<RegionKey> = leeuwarden.iter().map(|node| node.key()).collect();
        assert_eq!(iterated.len(), 19);
        assert_eq!(iterated[0], RegionKey::electoral_district(2));
        assert_eq!(iterated[1..].len(), leeuwarden.children().len());

        // Finding is relative to the node it starts from.
        assert_eq!(
            leeuwarden
                .find(RegionKey::municipality(80))
                .map(|node| node.key()),
            Some(RegionKey::municipality(80))
        );
        assert_eq!(
            leeuwarden
                .find(RegionKey::electoral_district(2))
                .map(|node| node.key()),
            Some(RegionKey::electoral_district(2))
        );
        assert!(leeuwarden.find(RegionKey::municipality(14)).is_none());
        assert!(leeuwarden.find(RegionKey::state()).is_none());
    }

    #[test]
    fn test_tk2025_hierarchy_from_owned_election_tree() {
        let tree = ElectionTreeHierarchy::try_from(tk2025_election_tree()).unwrap();

        assert_eq!(tree.root().key(), RegionKey::state());
        assert_eq!(tree.region_count(), TK2025_REGION_COUNT);
    }

    #[test]
    fn test_tk2025_flattening_round_trips() {
        let election_tree = tk2025_election_tree();
        let tree = election_tree.hierarchy().unwrap();

        // The TK2025 definition lists its regions breadth-first, which is the
        // order flattening produces, so the round trip is exact.
        let flattened = tree.flattened();
        let keys: Vec<RegionKey> = flattened.regions.iter().map(|region| region.key).collect();
        let expected: Vec<RegionKey> = election_tree
            .regions
            .iter()
            .map(|region| region.key)
            .collect();
        assert_eq!(keys, expected);

        // Flattening borrows, so the tree is still usable afterwards, and the
        // owned conversion produces the same regions.
        assert_eq!(tree.region_count(), TK2025_REGION_COUNT);
        let owned: Vec<RegionKey> = ElectionTree::from(tree)
            .regions
            .iter()
            .map(|region| region.key)
            .collect();
        assert_eq!(owned, expected);

        // Committees and the attributes of a region survive the round trip.
        let the_hague = flattened
            .regions
            .iter()
            .find(|region| region.key == RegionKey::electoral_district(12))
            .unwrap();
        assert_eq!(the_hague.committees.len(), 2);
        assert_eq!(the_hague.superior_region_key, Some(RegionKey::state()));
        assert_eq!(the_hague.name.as_ref(), "'s-Gravenhage");
    }

    #[test]
    fn test_hierarchy_error_converts_into_an_eml_error() {
        // Every chain of superior regions strictly rises in category and so has
        // to end at a region without a superior, which makes an election tree
        // with no regions at all the only way to end up without a root.
        let error = tree_error(Vec::new());
        assert_eq!(error, ElectionTreeHierarchyError::NoRoot);

        let eml_error = EMLError::from(error);
        assert!(matches!(
            eml_error.kind(),
            EMLErrorKind::InvalidElectionTree(ElectionTreeHierarchyError::NoRoot)
        ));
        assert_eq!(eml_error.span(), None);

        // The conversion is also available through `?` on an EMLError result.
        fn hierarchy(tree: &ElectionTree) -> Result<ElectionTreeHierarchy, EMLError> {
            Ok(tree.hierarchy()?)
        }

        let regions = vec![
            region(RegionKey::state(), None),
            region(RegionKey::municipality(14), Some(RegionKey::state())),
        ];
        assert!(hierarchy(&ElectionTree::new(regions)).is_ok());

        let duplicated = vec![
            region(RegionKey::state(), None),
            region(RegionKey::state(), None),
        ];
        let eml_error = hierarchy(&ElectionTree::new(duplicated)).unwrap_err();
        assert!(matches!(
            eml_error.kind(),
            EMLErrorKind::InvalidElectionTree(ElectionTreeHierarchyError::DuplicateRegion(key))
                if *key == RegionKey::state()
        ));
    }

    /// A region with the given key, subordinate to the region with the given
    /// superior key, if any.
    fn region(key: RegionKey, superior: Option<RegionKey>) -> Region {
        let region = Region::new("Region", key.category);
        let region = match key.number {
            Some(number) => region.with_number(number),
            None => region,
        };

        match superior {
            Some(superior) => region.with_superior_region_key(superior),
            None => region,
        }
    }

    /// The error returned when the given regions do not describe a valid region
    /// tree.
    fn tree_error(regions: Vec<Region>) -> ElectionTreeHierarchyError {
        ElectionTreeHierarchy::try_from(&ElectionTree::new(regions)).unwrap_err()
    }

    #[test]
    fn test_superior_region_must_be_of_a_higher_category() {
        // A municipality below an electoral district is fine, the other way
        // around is not.
        let regions = vec![
            region(RegionKey::electoral_district(1), None),
            region(
                RegionKey::municipality(14),
                Some(RegionKey::electoral_district(1)),
            ),
        ];
        let tree = ElectionTreeHierarchy::try_from(&ElectionTree::new(regions)).unwrap();
        assert_eq!(tree.region_count(), 2);

        let regions = vec![
            region(RegionKey::municipality(14), None),
            region(
                RegionKey::electoral_district(1),
                Some(RegionKey::municipality(14)),
            ),
        ];
        assert_eq!(
            tree_error(regions),
            ElectionTreeHierarchyError::InvalidSuperiorRegionCategory {
                region: RegionKey::electoral_district(1),
                superior: RegionKey::municipality(14),
            }
        );
    }

    #[test]
    fn test_superior_region_of_the_same_category_is_rejected() {
        let regions = vec![
            region(RegionKey::state(), None),
            region(RegionKey::municipality(14), Some(RegionKey::state())),
            region(
                RegionKey::municipality(80),
                Some(RegionKey::municipality(14)),
            ),
        ];

        assert_eq!(
            tree_error(regions),
            ElectionTreeHierarchyError::InvalidSuperiorRegionCategory {
                region: RegionKey::municipality(80),
                superior: RegionKey::municipality(14),
            }
        );
    }

    #[test]
    fn test_regions_referring_to_each_other_cannot_form_a_cycle() {
        // Every cycle contains at least one region whose superior region does not
        // sit at a higher level, so a cycle is always rejected on the category of
        // that superior region. Here that is the electoral district, which refers
        // downwards to a municipality.
        let regions = vec![
            region(
                RegionKey::electoral_district(1),
                Some(RegionKey::municipality(14)),
            ),
            region(
                RegionKey::municipality(14),
                Some(RegionKey::electoral_district(1)),
            ),
        ];
        assert_eq!(
            tree_error(regions),
            ElectionTreeHierarchyError::InvalidSuperiorRegionCategory {
                region: RegionKey::electoral_district(1),
                superior: RegionKey::municipality(14),
            }
        );

        // Two regions of the same category referring to each other are rejected
        // for the same reason.
        let regions = vec![
            region(
                RegionKey::municipality(14),
                Some(RegionKey::municipality(80)),
            ),
            region(
                RegionKey::municipality(80),
                Some(RegionKey::municipality(14)),
            ),
        ];
        assert_eq!(
            tree_error(regions),
            ElectionTreeHierarchyError::InvalidSuperiorRegionCategory {
                region: RegionKey::municipality(14),
                superior: RegionKey::municipality(80),
            }
        );
    }

    #[test]
    fn test_unknown_superior_region_is_reported() {
        // The superior region sits at a higher level, but does not exist.
        let regions = vec![
            region(RegionKey::state(), None),
            region(
                RegionKey::municipality(14),
                Some(RegionKey::electoral_district(1)),
            ),
        ];

        assert_eq!(
            tree_error(regions),
            ElectionTreeHierarchyError::UnknownSuperiorRegion {
                region: RegionKey::municipality(14),
                superior: RegionKey::electoral_district(1),
            }
        );

        // Comparing the categories is the cheaper of the two checks, so a
        // superior region that sits at a lower level and does not exist either is
        // reported on its category.
        let regions = vec![
            region(RegionKey::state(), None),
            region(
                RegionKey::electoral_district(1),
                Some(RegionKey::municipality(14)),
            ),
        ];

        assert_eq!(
            tree_error(regions),
            ElectionTreeHierarchyError::InvalidSuperiorRegionCategory {
                region: RegionKey::electoral_district(1),
                superior: RegionKey::municipality(14),
            }
        );
    }

    #[test]
    fn test_insert_requires_a_superior_region_of_a_higher_category() {
        let mut tree = ElectionTreeHierarchy::new(region(RegionKey::electoral_district(1), None));

        let key = tree
            .insert(
                RegionKey::electoral_district(1),
                region(RegionKey::municipality(14), None),
            )
            .unwrap();
        assert_eq!(key, RegionKey::municipality(14));
        assert_eq!(
            tree.get(key).unwrap().region().superior_region_key,
            Some(RegionKey::electoral_district(1))
        );

        // Neither a region of the same category nor one of a higher category can
        // be attached below the municipality.
        assert_eq!(
            tree.insert(
                RegionKey::municipality(14),
                region(RegionKey::municipality(80), None)
            ),
            Err(ElectionTreeHierarchyError::InvalidSuperiorRegionCategory {
                region: RegionKey::municipality(80),
                superior: RegionKey::municipality(14),
            })
        );
        assert_eq!(
            tree.insert(
                RegionKey::municipality(14),
                region(RegionKey::state(), None)
            ),
            Err(ElectionTreeHierarchyError::InvalidSuperiorRegionCategory {
                region: RegionKey::state(),
                superior: RegionKey::municipality(14),
            })
        );

        // A polling station sits below the municipality, and the failed inserts
        // left the tree untouched.
        tree.insert(
            RegionKey::municipality(14),
            region(RegionKey::polling_station(1), None),
        )
        .unwrap();
        assert_eq!(tree.region_count(), 3);
    }

    #[test]
    fn test_insert_reports_an_unknown_parent_region() {
        let mut tree = ElectionTreeHierarchy::new(region(RegionKey::electoral_district(1), None));

        // The region being inserted is of a lower category than the region it is
        // attached to, so it is the missing parent region that is reported.
        assert_eq!(
            tree.insert(
                RegionKey::electoral_district(2),
                region(RegionKey::municipality(14), None)
            ),
            Err(ElectionTreeHierarchyError::UnknownRegion(
                RegionKey::electoral_district(2)
            ))
        );
    }

    #[test]
    fn test_insert_checks_the_regions_below_the_inserted_region() {
        let mut tree = ElectionTreeHierarchy::new(region(RegionKey::state(), None));
        let district = RegionKey::electoral_district(1);
        let node = |superior| {
            RegionNode::new(
                region(district, None),
                [RegionNode::from(region(
                    RegionKey::municipality(14),
                    superior,
                ))],
            )
        };

        // The municipality below the district being inserted has to refer to that
        // district as its superior region.
        assert_eq!(
            tree.insert(RegionKey::state(), node(None)),
            Err(ElectionTreeHierarchyError::InconsistentSuperiorRegion {
                region: RegionKey::municipality(14),
                superior: district,
            })
        );
        assert_eq!(tree.region_count(), 1);

        // Once it does, both regions are inserted.
        tree.insert(RegionKey::state(), node(Some(district)))
            .unwrap();
        assert_eq!(tree.region_count(), 3);

        // A region already in the tree cannot be inserted again, wherever below
        // the inserted region it appears.
        let duplicated = RegionNode::new(
            region(RegionKey::electoral_district(2), None),
            [RegionNode::from(region(
                RegionKey::municipality(14),
                Some(RegionKey::electoral_district(2)),
            ))],
        );
        assert_eq!(
            tree.insert(RegionKey::state(), duplicated),
            Err(ElectionTreeHierarchyError::DuplicateRegion(
                RegionKey::municipality(14)
            ))
        );
        assert_eq!(tree.region_count(), 3);
    }

    #[test]
    fn test_hierarchy_from_a_region_node() {
        let mut tk2025 = tk2025_hierarchy();
        let leeuwarden = tk2025.remove(RegionKey::electoral_district(2)).unwrap();

        // A detached region becomes the root region of an election tree of its
        // own, which clears the superior region it used to refer to.
        let district = ElectionTreeHierarchy::try_from(leeuwarden).unwrap();
        assert_eq!(district.root().key(), RegionKey::electoral_district(2));
        assert_eq!(district.root().region().superior_region_key, None);
        assert_eq!(district.region_count(), 19);

        // The root region of a tree can be taken out and put back unchanged.
        let mut root = ElectionTreeHierarchy::try_from(tk2025_hierarchy().into_inner())
            .unwrap()
            .into_inner();
        assert_eq!(root.iter().count(), TK2025_REGION_COUNT);

        // Manipulating a region below the root region into disagreeing about
        // where it sits is rejected.
        root.find_mut(RegionKey::municipality(80))
            .unwrap()
            .region_mut()
            .superior_region_key = Some(RegionKey::state());
        assert_eq!(
            ElectionTreeHierarchy::try_from(root).unwrap_err(),
            ElectionTreeHierarchyError::InconsistentSuperiorRegion {
                region: RegionKey::municipality(80),
                superior: RegionKey::electoral_district(2),
            }
        );
    }

    #[test]
    fn test_hierarchy_from_a_region_node_checks_region_categories() {
        // The electoral district below the municipality does not sit at a lower
        // level of the election tree, so these regions do not form a tree.
        let inverted = RegionNode::new(
            region(RegionKey::municipality(14), None),
            [RegionNode::from(region(
                RegionKey::electoral_district(1),
                Some(RegionKey::municipality(14)),
            ))],
        );

        assert_eq!(
            ElectionTreeHierarchy::try_from(inverted).unwrap_err(),
            ElectionTreeHierarchyError::InvalidSuperiorRegionCategory {
                region: RegionKey::electoral_district(1),
                superior: RegionKey::municipality(14),
            }
        );
    }
}
