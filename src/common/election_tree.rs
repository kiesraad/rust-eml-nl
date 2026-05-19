use instant_xml::ToXml;

use crate::{
    EMLError, NS_KR,
    io::{EMLElement, EMLElementReader, QualifiedName, collect_struct},
};

/// Election tree as defined in EML_NL.
#[derive(Debug, Clone, ToXml)]
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

impl EMLElement for ElectionTree {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("ElectionTree", Some(NS_KR));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, ElectionTree {
            regions as Vec: ElectionTreeRegion::EML_NAME => |elem| ElectionTreeRegion::read_eml(elem)?,
        }))
    }
}

/// A region in the election tree.
#[derive(Debug, Clone, ToXml)]
#[xml(rename = "Region", ns(NS_KR), force_prefix)]
pub struct ElectionTreeRegion {}

impl EMLElement for ElectionTreeRegion {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Region", Some(NS_KR));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        // TODO: handle election tree region
        elem.skip()?;
        Ok(ElectionTreeRegion {})
    }
}
