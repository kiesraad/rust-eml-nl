//! Document variants and related types for the all the specific EML_NL documents.

use std::str::FromStr;

use instant_xml::{Accumulate, FromXml, ToXml};

use crate::{
    EMLError, EMLErrorKind, EMLResultExt as _, NS_EML,
    common::ElectionDomain,
    documents::{
        candidate_lists::{CandidateLists, CandidateListsElectionIdentifier, CandidateListsType},
        election_count::{CountType, ElectionCount, ElectionCountElectionIdentifier},
        election_definition::{
            EML_ELECTION_DEFINITION_ID, ElectionDefinition, ElectionDefinitionElectionIdentifier,
        },
        election_result::{
            EML_ELECTION_RESULT_ID, ElectionResult, ElectionResultElectionIdentifier,
        },
        nomination::{EML_NOMINATION_ID, Nomination, NominationElectionIdentifier},
        polling_stations::{
            EML_POLLING_STATIONS_ID, PollingStations, PollingStationsElectionIdentifier,
        },
    },
    utils::{ElectionCategory, ElectionId, ElectionSubcategory, StringValue, XsDate},
};

pub mod candidate_lists;
pub mod election_count;
pub mod election_definition;
pub mod election_result;
pub mod nomination;
pub mod polling_stations;

/// Generic EML document that can represent any of the supported EML variants.
///
/// You can use this struct to parse an EML document of any variant if you don't
/// know in advance which variant you will receive.
#[derive(Debug, Clone)]
pub enum EML {
    /// Representing a `110a` document, containing an election definition.
    ElectionDefinition(Box<ElectionDefinition>),

    /// Representing a `110b` document, containing polling stations.
    PollingStations(Box<PollingStations>),

    /// Representing a `210` document, containing a nomination.
    Nomination(Box<Nomination>),

    /// Representing a `230b` document, containing a candidate list.
    CandidateLists(Box<CandidateLists>),

    /// Representing a `510a`, `510b`, `510c` or `510d` document, containing a count.
    ElectionCount(Box<ElectionCount>),

    /// Representing a `520` document, containing the election result.
    ElectionResult(Box<ElectionResult>),
}

impl EML {
    /// Get the EML document ID string for this document variant (e.g. `110a`).
    pub fn to_eml_id(&self) -> &'static str {
        match self {
            EML::ElectionDefinition(_) => EML_ELECTION_DEFINITION_ID,
            EML::PollingStations(_) => EML_POLLING_STATIONS_ID,
            EML::Nomination(_) => EML_NOMINATION_ID,
            EML::CandidateLists(cl) => cl.lists_type.to_eml_id(),
            EML::ElectionCount(c) => c.count_type.to_eml_id(),
            EML::ElectionResult(_) => EML_ELECTION_RESULT_ID,
        }
    }

    /// Get a friendly name for this EML document variant.
    pub fn to_friendly_name(&self) -> &'static str {
        match self {
            EML::ElectionDefinition(_) => "Election Definition",
            EML::PollingStations(_) => "Polling Stations",
            EML::Nomination(_) => "Nomination",
            EML::CandidateLists(cl) => cl.lists_type.to_friendly_name(),
            EML::ElectionCount(c) => c.count_type.to_friendly_name(),
            EML::ElectionResult(_) => "Result",
        }
    }

    /// Create a generic EML document from an Election Definition (`110a`) document.
    pub fn from_election_definition_doc(ed: ElectionDefinition) -> Self {
        EML::ElectionDefinition(Box::new(ed))
    }

    /// Check if this EML document is an Election Definition (`110a`) document.
    pub fn is_election_definition_doc(&self) -> bool {
        matches!(self, EML::ElectionDefinition(_))
    }

    /// Get a reference to this EML document as an Election Definition (`110a`) document, if possible.
    pub fn as_election_definition_doc(&self) -> Option<&ElectionDefinition> {
        match self {
            EML::ElectionDefinition(ed) => Some(ed),
            _ => None,
        }
    }

    /// Create a generic EML document from a Polling Stations (`110b`) document.
    pub fn from_polling_stations_doc(ps: PollingStations) -> Self {
        EML::PollingStations(Box::new(ps))
    }

    /// Check if this EML document is a Polling Stations (`110b`) document.
    pub fn is_polling_stations_doc(&self) -> bool {
        matches!(self, EML::PollingStations(_))
    }

    /// Get a reference to this EML document as a Polling Stations (`110b`) document, if possible.
    pub fn as_polling_stations_doc(&self) -> Option<&PollingStations> {
        match self {
            EML::PollingStations(ps) => Some(ps),
            _ => None,
        }
    }

    /// Create a generic EML document from a Nomination (`210`) document.
    pub fn from_nomination_doc(nom: Nomination) -> Self {
        EML::Nomination(Box::new(nom))
    }

    /// Check if this EML document is a Nomination (`210`) document.
    pub fn is_nomination_doc(&self) -> bool {
        matches!(self, EML::Nomination(_))
    }

    /// Get a reference to this EML document as a Nomination (`210`) document, if possible.
    pub fn as_nomination_doc(&self) -> Option<&Nomination> {
        match self {
            EML::Nomination(nom) => Some(nom),
            _ => None,
        }
    }

    /// Create a generic EML document from a Candidate Lists (`230b`) document.
    pub fn from_candidate_lists_doc(cl: CandidateLists) -> Self {
        EML::CandidateLists(Box::new(cl))
    }

    /// Check if this EML document is a Candidate Lists (`230b`) document.
    pub fn is_candidate_lists_doc(&self) -> bool {
        matches!(self, EML::CandidateLists(_))
    }

    /// Get a reference to this EML document as a Candidate Lists (`230b`) document, if possible.
    pub fn as_candidate_lists_doc(&self) -> Option<&CandidateLists> {
        match self {
            EML::CandidateLists(cl) => Some(cl),
            _ => None,
        }
    }

    /// Create a generic EML document from a Count (`510a`, `510b`, `510c` or `510d`) document.
    pub fn from_count_doc(count: ElectionCount) -> Self {
        EML::ElectionCount(Box::new(count))
    }

    /// Check if this EML document is a Count (`510a`, `510b`, `510c` or `510d`) document.
    pub fn is_count_doc(&self) -> bool {
        matches!(self, EML::ElectionCount(_))
    }

    /// Get a reference to this EML document as a Count (`510a`, `510b`, `510c` or `510d`) document, if possible.
    pub fn as_count_doc(&self) -> Option<&ElectionCount> {
        match self {
            EML::ElectionCount(count) => Some(count),
            _ => None,
        }
    }

    /// Create a generic EML document from a Result (`520`) document.
    pub fn from_result_doc(result: ElectionResult) -> Self {
        EML::ElectionResult(Box::new(result))
    }

    /// Check if this EML document is a Result (`520`) document.
    pub fn is_result_doc(&self) -> bool {
        matches!(self, EML::ElectionResult(_))
    }

    /// Get a reference to this EML document as a Result (`520`) document, if possible.
    pub fn as_result_doc(&self) -> Option<&ElectionResult> {
        match self {
            EML::ElectionResult(result) => Some(result),
            _ => None,
        }
    }
}

// Custom: enum dispatch across six document type variants.
impl ToXml for EML {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        match self {
            EML::ElectionDefinition(ed) => ed.serialize(field, serializer),
            EML::PollingStations(ps) => ps.serialize(field, serializer),
            EML::Nomination(nom) => nom.serialize(field, serializer),
            EML::CandidateLists(cl) => cl.serialize(field, serializer),
            EML::ElectionCount(c) => c.serialize(field, serializer),
            EML::ElectionResult(r) => r.serialize(field, serializer),
        }
    }
}

// Custom: enum dispatch across six document type variants based on Id attribute.
impl<'xml> FromXml<'xml> for EML {
    fn matches(id: instant_xml::Id<'_>, field: Option<instant_xml::Id<'_>>) -> bool {
        match field {
            Some(field) => id == field,
            None => {
                id == instant_xml::Id {
                    ns: NS_EML,
                    name: "EML",
                }
            }
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut instant_xml::Deserializer<'cx, 'xml>,
    ) -> Result<(), instant_xml::Error> {
        if into.is_some() {
            return Err(instant_xml::Error::DuplicateValue(field));
        }

        // Read attributes to find the document type Id
        let mut doc_id = None;
        use instant_xml::de::Node;
        loop {
            match deserializer.next() {
                Some(Ok(Node::Attribute(attr))) if attr.local == "Id" => {
                    doc_id = Some(attr.value.to_string());
                }
                Some(Ok(Node::Attribute(_))) => continue,
                Some(Ok(node)) => {
                    // Put the first non-attribute node back and deserialize the body
                    let mut deserializer = deserializer.for_node(node);
                    let doc_id = doc_id
                        .as_deref()
                        .ok_or(instant_xml::Error::MissingValue("EML::Id"))?;

                    *into = Some(match doc_id {
                        EML_ELECTION_DEFINITION_ID => {
                            let mut acc =
                                <ElectionDefinition as FromXml<'xml>>::Accumulator::default();
                            deserialize_eml_body::<ElectionDefinition>(
                                &mut acc,
                                &mut deserializer,
                            )?;
                            EML::ElectionDefinition(Box::new(acc.try_done(field)?))
                        }
                        EML_POLLING_STATIONS_ID => {
                            let mut acc =
                                <PollingStations as FromXml<'xml>>::Accumulator::default();
                            deserialize_eml_body::<PollingStations>(&mut acc, &mut deserializer)?;
                            EML::PollingStations(Box::new(acc.try_done(field)?))
                        }
                        EML_ELECTION_RESULT_ID => {
                            let mut acc = <ElectionResult as FromXml<'xml>>::Accumulator::default();
                            deserialize_eml_body::<ElectionResult>(&mut acc, &mut deserializer)?;
                            EML::ElectionResult(Box::new(acc.try_done(field)?))
                        }
                        EML_NOMINATION_ID => {
                            let mut acc = <Nomination as FromXml<'xml>>::Accumulator::default();
                            deserialize_eml_body::<Nomination>(&mut acc, &mut deserializer)?;
                            EML::Nomination(Box::new(acc.try_done(field)?))
                        }
                        id if CandidateListsType::is_valid_eml_id(id) => {
                            let mut acc = <CandidateLists as FromXml<'xml>>::Accumulator::default();
                            deserialize_eml_body::<CandidateLists>(&mut acc, &mut deserializer)?;
                            let mut cl: CandidateLists = acc.try_done(field)?;
                            cl.lists_type = CandidateListsType::from_eml_id(id)
                                .map_err(|e| instant_xml::Error::UnexpectedValue(e.to_string()))?;
                            EML::CandidateLists(Box::new(cl))
                        }
                        id if CountType::is_valid_eml_id(id) => {
                            let mut acc = <ElectionCount as FromXml<'xml>>::Accumulator::default();
                            deserialize_eml_body::<ElectionCount>(&mut acc, &mut deserializer)?;
                            let mut ec: ElectionCount = acc.try_done(field)?;
                            ec.count_type = CountType::from_eml_id(id)
                                .map_err(|e| instant_xml::Error::UnexpectedValue(e.to_string()))?;
                            EML::ElectionCount(Box::new(ec))
                        }
                        _ => {
                            return Err(instant_xml::Error::UnexpectedValue(format!(
                                "unknown EML document type: {doc_id}"
                            )));
                        }
                    });
                    deserializer.ignore()?;
                    return Ok(());
                }
                Some(Err(e)) => return Err(e),
                None => {
                    return Err(instant_xml::Error::MissingValue("EML::Id"));
                }
            }
        }
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Element;
}

/// Helper: deserialize EML body from a deserializer positioned inside `<EML>`.
fn deserialize_eml_body<'cx, 'xml, T: FromXml<'xml>>(
    acc: &mut T::Accumulator,
    deserializer: &mut instant_xml::Deserializer<'cx, 'xml>,
) -> Result<(), instant_xml::Error> {
    T::deserialize(acc, "EML", deserializer)
}

impl From<ElectionDefinition> for EML {
    fn from(ed: ElectionDefinition) -> Self {
        EML::from_election_definition_doc(ed)
    }
}

impl From<PollingStations> for EML {
    fn from(ps: PollingStations) -> Self {
        EML::from_polling_stations_doc(ps)
    }
}

impl From<Nomination> for EML {
    fn from(nom: Nomination) -> Self {
        EML::from_nomination_doc(nom)
    }
}

impl From<CandidateLists> for EML {
    fn from(cl: CandidateLists) -> Self {
        EML::from_candidate_lists_doc(cl)
    }
}

impl From<ElectionCount> for EML {
    fn from(count: ElectionCount) -> Self {
        EML::from_count_doc(count)
    }
}

impl From<ElectionResult> for EML {
    fn from(result: ElectionResult) -> Self {
        EML::from_result_doc(result)
    }
}

impl FromStr for EML {
    type Err = EMLError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::io::EMLRead as _;
        Self::parse_eml(s, crate::io::EMLParsingMode::Strict).ok()
    }
}

impl TryFrom<&str> for EML {
    type Error = EMLError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use crate::io::EMLRead as _;
        Self::parse_eml(value, crate::io::EMLParsingMode::Strict).ok()
    }
}

impl TryFrom<EML> for String {
    type Error = EMLError;

    fn try_from(value: EML) -> Result<Self, Self::Error> {
        use crate::io::EMLWrite as _;
        value.write_eml_root_str(true)
    }
}

/// Builder for Election Identifiers
#[derive(Debug, Clone)]
pub struct ElectionIdentifierBuilder {
    id: Option<StringValue<ElectionId>>,
    name: Option<String>,
    category: Option<StringValue<ElectionCategory>>,
    subcategory: Option<StringValue<ElectionSubcategory>>,
    domain: Option<ElectionDomain>,
    election_date: Option<StringValue<XsDate>>,
    nomination_date: Option<StringValue<XsDate>>,
}

impl ElectionIdentifierBuilder {
    /// Create a new ElectionIdentifierBuilder
    pub fn new() -> Self {
        ElectionIdentifierBuilder {
            id: None,
            name: None,
            category: None,
            subcategory: None,
            domain: None,
            election_date: None,
            nomination_date: None,
        }
    }

    /// Set the election identifier for the document.
    pub fn id(mut self, id: impl Into<ElectionId>) -> Self {
        self.id = Some(StringValue::from_value(id.into()));
        self
    }

    /// Set the election name for the document
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the election category for the document
    pub fn category(mut self, category: impl Into<ElectionCategory>) -> Self {
        self.category = Some(StringValue::from_value(category.into()));
        self
    }

    /// Set the election sub category for the document
    pub fn subcategory(mut self, subcategory: impl Into<ElectionSubcategory>) -> Self {
        self.subcategory = Some(StringValue::from_value(subcategory.into()));
        self
    }

    /// Set the election domain for the document
    pub fn domain(mut self, domain: impl Into<ElectionDomain>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Set the election date for the document
    pub fn election_date(mut self, date: impl Into<XsDate>) -> Self {
        self.election_date = Some(StringValue::from_value(date.into()));
        self
    }

    /// Set the nomination date for the document
    pub fn nomination_date(mut self, date: impl Into<XsDate>) -> Self {
        self.nomination_date = Some(StringValue::from_value(date.into()));
        self
    }

    /// Build an election identifier for nomination documents.
    ///
    /// Requires specifying [`Self::id`], [`Self::category`], [`Self::election_date`]
    /// and [`Self::nomination_date`]. Optionally, [`Self::name`], [`Self::subcategory`]
    /// and [`Self::domain`] may be specified as well.
    pub fn build_for_nomination(self) -> Result<NominationElectionIdentifier, EMLError> {
        let category = self
            .category
            .ok_or(EMLErrorKind::MissingBuildProperty("category").without_span())?;
        validate_category_and_subcategory(&category, self.subcategory.as_ref())?;

        let election_date = self
            .election_date
            .ok_or(EMLErrorKind::MissingBuildProperty("election_date").without_span())?;
        let nomination_date = self
            .nomination_date
            .ok_or(EMLErrorKind::MissingBuildProperty("nomination_date").without_span())?;
        validate_election_and_nomination_dates(Some(&election_date), Some(&nomination_date))?;

        Ok(NominationElectionIdentifier {
            id: self
                .id
                .ok_or(EMLErrorKind::MissingBuildProperty("id").without_span())?,
            name: self.name,
            category,
            subcategory: self.subcategory,
            domain: self.domain,
            election_date,
            nomination_date,
        })
    }

    /// Build an election identifier for candidate lists.
    ///
    /// Requires specifying [`Self::id`], [`Self::category`], [`Self::election_date`]
    /// and [`Self::nomination_date`]. Optionally, [`Self::name`], [`Self::subcategory`]
    /// and [`Self::domain`] may be specified as well.
    pub fn build_for_candidate_lists(self) -> Result<CandidateListsElectionIdentifier, EMLError> {
        let category = self
            .category
            .ok_or(EMLErrorKind::MissingBuildProperty("category").without_span())?;
        validate_category_and_subcategory(&category, self.subcategory.as_ref())?;

        let election_date = self
            .election_date
            .ok_or(EMLErrorKind::MissingBuildProperty("election_date").without_span())?;
        let nomination_date = self
            .nomination_date
            .ok_or(EMLErrorKind::MissingBuildProperty("nomination_date").without_span())?;
        validate_election_and_nomination_dates(Some(&election_date), Some(&nomination_date))?;

        Ok(CandidateListsElectionIdentifier {
            id: self
                .id
                .ok_or(EMLErrorKind::MissingBuildProperty("id").without_span())?,
            name: self.name,
            category,
            subcategory: self.subcategory,
            domain: self.domain,
            election_date,
            nomination_date,
        })
    }

    /// Build an election identifier for election count.
    ///
    /// Requires specifying [`Self::id`], [`Self::category`] and [`Self::election_date`].
    /// Optionally, [`Self::name`], [`Self::subcategory`] and [`Self::domain`]
    /// may be specified as well.
    pub fn build_for_count(self) -> Result<ElectionCountElectionIdentifier, EMLError> {
        let category = self
            .category
            .ok_or(EMLErrorKind::MissingBuildProperty("category").without_span())?;
        validate_category_and_subcategory(&category, self.subcategory.as_ref())?;

        Ok(ElectionCountElectionIdentifier {
            id: self
                .id
                .ok_or(EMLErrorKind::MissingBuildProperty("id").without_span())?,
            name: self.name,
            category,
            subcategory: self.subcategory,
            domain: self.domain,
            election_date: self
                .election_date
                .ok_or(EMLErrorKind::MissingBuildProperty("election_date").without_span())?,
        })
    }

    /// Build an election identifier for election result.
    ///
    /// Requires specifying [`Self::id`], [`Self::category`] and [`Self::election_date`].
    /// Optionally, [`Self::name`], [`Self::subcategory`] and [`Self::domain`] may be specified as well.
    pub fn build_for_result(self) -> Result<ElectionResultElectionIdentifier, EMLError> {
        let category = self
            .category
            .ok_or(EMLErrorKind::MissingBuildProperty("category").without_span())?;
        validate_category_and_subcategory(&category, self.subcategory.as_ref())?;

        Ok(ElectionResultElectionIdentifier {
            id: self
                .id
                .ok_or(EMLErrorKind::MissingBuildProperty("id").without_span())?,
            name: self.name,
            category,
            subcategory: self.subcategory,
            domain: self.domain,
            election_date: self
                .election_date
                .ok_or(EMLErrorKind::MissingBuildProperty("election_date").without_span())?,
        })
    }

    /// Build an election identifier for election definition.
    ///
    /// Requires specifying [`Self::id`], [`Self::name`], [`Self::category`],
    /// [`Self::subcategory`], [`Self::election_date`] and [`Self::nomination_date`].
    /// Optionally [`Self::domain`] may be specified as well.
    pub fn build_for_definition(self) -> Result<ElectionDefinitionElectionIdentifier, EMLError> {
        let category = self
            .category
            .ok_or(EMLErrorKind::MissingBuildProperty("category").without_span())?;
        let subcategory = self
            .subcategory
            .ok_or(EMLErrorKind::MissingBuildProperty("subcategory").without_span())?;
        validate_category_and_subcategory(&category, Some(&subcategory))?;

        let election_date = self
            .election_date
            .ok_or(EMLErrorKind::MissingBuildProperty("election_date").without_span())?;
        let nomination_date = self
            .nomination_date
            .ok_or(EMLErrorKind::MissingBuildProperty("nomination_date").without_span())?;
        validate_election_and_nomination_dates(Some(&election_date), Some(&nomination_date))?;

        Ok(ElectionDefinitionElectionIdentifier {
            id: self
                .id
                .ok_or(EMLErrorKind::MissingBuildProperty("id").without_span())?,
            name: self
                .name
                .ok_or(EMLErrorKind::MissingBuildProperty("name").without_span())?,
            category,
            subcategory,
            domain: self.domain,
            election_date,
            nomination_date,
        })
    }

    /// Build an election identifier for polling stations.
    ///
    /// Requires specifying [`Self::id`], [`Self::category`] and [`Self::election_date`].
    /// Optionally, [`Self::name`], [`Self::subcategory`] and [`Self::domain`]
    /// may be specified as well.
    pub fn build_for_polling_stations(self) -> Result<PollingStationsElectionIdentifier, EMLError> {
        let category = self
            .category
            .ok_or(EMLErrorKind::MissingBuildProperty("category").without_span())?;
        validate_category_and_subcategory(&category, self.subcategory.as_ref())?;

        Ok(PollingStationsElectionIdentifier {
            id: self
                .id
                .ok_or(EMLErrorKind::MissingBuildProperty("id").without_span())?,
            name: self.name,
            category,
            subcategory: self.subcategory,
            domain: self.domain,
            election_date: self
                .election_date
                .ok_or(EMLErrorKind::MissingBuildProperty("election_date").without_span())?,
        })
    }
}

impl Default for ElectionIdentifierBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn validate_election_and_nomination_dates(
    election_date: Option<&StringValue<XsDate>>,
    nomination_date: Option<&StringValue<XsDate>>,
) -> Result<(), EMLError> {
    let election_date = election_date.and_then(|ed| ed.copied_value().ok());
    let nomination_date = nomination_date.and_then(|nd| nd.copied_value().ok());

    if let (Some(ed), Some(nd)) = (election_date, nomination_date)
        && nd.date >= ed.date
    {
        return Err(EMLErrorKind::NominationDateNotBeforeElectionDate).without_span();
    }
    Ok(())
}

pub(crate) fn validate_category_and_subcategory(
    category: &StringValue<ElectionCategory>,
    subcategory: Option<&StringValue<ElectionSubcategory>>,
) -> Result<(), EMLError> {
    let category = category.copied_value().ok();
    let subcategory = subcategory.and_then(|sc| sc.copied_value().ok());

    if let (Some(c), Some(sc)) = (category, subcategory)
        && !sc.is_subcategory_of(c)
    {
        return Err(EMLErrorKind::InvalidElectionSubcategory).without_span();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{EMLParsingMode, EMLRead as _, EMLWrite as _};

    #[test]
    fn test_parsing_arbitrary_eml_documents() {
        let doc = include_str!("../../test-emls/nomination/eml210_test.eml.xml");
        let eml = EML::parse_eml(doc, EMLParsingMode::Strict)
            .ok()
            .expect("Failed to parse EML document");
        assert!(matches!(eml, EML::Nomination(_)));

        let doc = include_str!("../../test-emls/candidate_lists/eml230b_test.eml.xml");
        let eml = EML::parse_eml(doc, EMLParsingMode::Strict)
            .ok()
            .expect("Failed to parse EML document");
        assert!(matches!(eml, EML::CandidateLists(_)));

        let doc = include_str!("../../test-emls/election_definition/eml110a_test.eml.xml");
        let eml = EML::parse_eml(doc, EMLParsingMode::Strict)
            .ok()
            .expect("Failed to parse EML document");
        assert!(matches!(eml, EML::ElectionDefinition(_)));

        let doc = include_str!("../../test-emls/polling_stations/eml110b_test.eml.xml");
        let eml = EML::parse_eml(doc, EMLParsingMode::Strict)
            .ok()
            .expect("Failed to parse EML document");
        assert!(matches!(eml, EML::PollingStations(_)));

        let doc = include_str!("../../test-emls/election_count/deserialize_eml510b_test.eml.xml");
        let eml = EML::parse_eml(doc, EMLParsingMode::Strict)
            .ok()
            .expect("Failed to parse EML document");
        assert!(matches!(eml, EML::ElectionCount(_)));

        let doc = include_str!("../../test-emls/election_result/eml520_test.eml.xml");
        let eml = EML::parse_eml(doc, EMLParsingMode::Strict)
            .ok()
            .expect("Failed to parse EML document");
        assert!(matches!(eml, EML::ElectionResult(_)));
    }

    #[test]
    fn parse_and_write_210_eml_document_should_not_fail() {
        let doc = include_str!("../../test-emls/nomination/eml210_test.eml.xml");
        let eml = EML::parse_eml(doc, EMLParsingMode::Strict)
            .ok()
            .expect("Failed to parse EML document");
        assert_eq!(eml.to_eml_id(), "210");
        assert_eq!(eml.to_friendly_name(), "Nomination");
        assert!(eml.is_nomination_doc());
        assert!(!eml.is_election_definition_doc());
        assert!(eml.as_nomination_doc().is_some());
        assert!(eml.as_election_definition_doc().is_none());
        eml.write_eml_root_str(true).unwrap();
    }

    #[test]
    fn parse_and_write_110a_eml_document_should_not_fail() {
        let doc = include_str!("../../test-emls/election_definition/eml110a_test.eml.xml");
        let eml = EML::parse_eml(doc, EMLParsingMode::Strict)
            .ok()
            .expect("Failed to parse EML document");
        assert_eq!(eml.to_eml_id(), "110a");
        assert_eq!(eml.to_friendly_name(), "Election Definition");
        assert!(eml.is_election_definition_doc());
        assert!(!eml.is_result_doc());
        assert!(eml.as_election_definition_doc().is_some());
        assert!(eml.as_result_doc().is_none());
        eml.write_eml_root_str(true).unwrap();
    }

    #[test]
    fn parse_and_write_110b_eml_document_should_not_fail() {
        let doc = include_str!("../../test-emls/polling_stations/eml110b_test.eml.xml");
        let eml = EML::parse_eml(doc, EMLParsingMode::Strict)
            .ok()
            .expect("Failed to parse EML document");
        assert_eq!(eml.to_eml_id(), "110b");
        assert_eq!(eml.to_friendly_name(), "Polling Stations");
        assert!(eml.is_polling_stations_doc());
        assert!(!eml.is_election_definition_doc());
        assert!(eml.as_polling_stations_doc().is_some());
        assert!(eml.as_election_definition_doc().is_none());
        eml.write_eml_root_str(true).unwrap();
    }

    #[test]
    fn parse_and_write_230b_eml_document_should_not_fail() {
        let doc = include_str!("../../test-emls/candidate_lists/eml230b_test.eml.xml");
        let eml = EML::parse_eml(doc, EMLParsingMode::Strict)
            .ok()
            .expect("Failed to parse EML document");
        assert_eq!(eml.to_eml_id(), "230b");
        assert_eq!(eml.to_friendly_name(), "Candidate Lists");
        assert!(eml.is_candidate_lists_doc());
        assert!(!eml.is_polling_stations_doc());
        assert!(eml.as_candidate_lists_doc().is_some());
        assert!(eml.as_polling_stations_doc().is_none());
        eml.write_eml_root_str(true).unwrap();
    }

    #[test]
    fn parse_and_write_510b_eml_document_should_not_fail() {
        let doc = include_str!("../../test-emls/election_count/deserialize_eml510b_test.eml.xml");
        let eml = EML::parse_eml(doc, EMLParsingMode::Strict)
            .ok()
            .expect("Failed to parse EML document");
        assert_eq!(eml.to_eml_id(), "510b");
        assert_eq!(eml.to_friendly_name(), "Municipal Count");
        assert!(eml.is_count_doc());
        assert!(!eml.is_candidate_lists_doc());
        assert!(eml.as_count_doc().is_some());
        assert!(eml.as_candidate_lists_doc().is_none());
        eml.write_eml_root_str(true).unwrap();
    }

    #[test]
    fn parse_and_write_520_eml_document_should_not_fail() {
        let doc = include_str!("../../test-emls/election_result/eml520_test.eml.xml");
        let eml = EML::parse_eml(doc, EMLParsingMode::Strict)
            .ok()
            .expect("Failed to parse EML document");
        assert_eq!(eml.to_eml_id(), "520");
        assert_eq!(eml.to_friendly_name(), "Result");
        assert!(eml.is_result_doc());
        assert!(!eml.is_count_doc());
        assert!(eml.as_result_doc().is_some());
        assert!(eml.as_count_doc().is_none());
        // Also test via the inner type to compare
        let inner = eml.as_result_doc().unwrap();
        inner.write_eml_root_str(true).expect("inner failed");
        eml.write_eml_root_str(true).expect("eml failed");
    }
}
