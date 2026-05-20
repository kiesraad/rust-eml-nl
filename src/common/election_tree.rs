use instant_xml::{FromXml, ToXml};

use crate::NS_KR;

/// Election tree as defined in EML_NL.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(ns(NS_KR), force_prefix)]
pub struct ElectionTree {
    /// Regions defined for this part of the election tree
    #[xml(rename = "Region")]
    pub regions: Vec<ElectionTreeRegion>,
}

impl ElectionTree {
    /// Create a new election tree with the given regions.
    pub fn new(regions: impl Into<Vec<ElectionTreeRegion>>) -> Self {
        ElectionTree {
            regions: regions.into(),
        }
    }
}

impl From<Vec<ElectionTreeRegion>> for ElectionTree {
    fn from(regions: Vec<ElectionTreeRegion>) -> Self {
        ElectionTree::new(regions)
    }
}

/// A region in the election tree.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "Region", ns(NS_KR), force_prefix)]
pub struct ElectionTreeRegion {}
