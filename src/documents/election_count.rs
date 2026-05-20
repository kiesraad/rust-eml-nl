//! Document variant for the EML_NL Count (`510a`, `510b`, `510c` or `510d`) document.

use std::{collections::BTreeMap, fmt, num::NonZeroU64, str::FromStr};

use instant_xml::{FromXml, ToXml, ser::Context};

use crate::{
    EML_SCHEMA_VERSION, EMLError, EMLErrorKind, EMLResultExt as _, EMLValueResultExt, NS_EML,
    NS_KR,
    common::{
        CandidateIdentifier, CanonicalizationMethod, ContestIdentifier, CreationDateTime,
        ElectionDomain, ManagingAuthority, MinimalQualifyingAddress, PersonNameStructure,
        ReportingUnitIdentifier, TransactionId,
    },
    documents::ElectionIdentifierBuilder,
    eml_ns_context,
    utils::{
        AffiliationId, CandidateId, ElectionCategory, ElectionId, ElectionSubcategory, Gender,
        StringValue, XsDate, XsDateTime,
    },
};

/// Representing a `510a`, `510b`, `510c` or `510d` document, containing a count.
#[derive(Debug, Clone)]
pub struct ElectionCount {
    /// Type of count document.
    pub count_type: CountType,

    /// Transaction ID of the document.
    pub transaction_id: TransactionId,

    /// Managing authority of the document.
    pub managing_authority: ManagingAuthority,

    /// Creation date and time of the document.
    pub creation_date_time: CreationDateTime,

    /// Canonicalization method used in this document, if present.
    pub canonicalization_method: Option<CanonicalizationMethod>,

    /// The actual count data.
    pub count: ElectionCountCount,
}

impl ElectionCount {
    /// Create a builder for the [`ElectionCount`] document.
    pub fn builder() -> ElectionCountBuilder {
        ElectionCountBuilder::new()
    }
}

impl FromStr for ElectionCount {
    type Err = EMLError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::io::EMLRead as _;
        Self::parse_eml(s).ok()
    }
}

impl TryFrom<&str> for ElectionCount {
    type Error = EMLError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use crate::io::EMLRead as _;
        Self::parse_eml(value).ok()
    }
}

impl TryFrom<ElectionCount> for String {
    type Error = EMLError;

    fn try_from(value: ElectionCount) -> Result<Self, Self::Error> {
        use crate::io::EMLWrite as _;
        value.write_eml_root_str(true)
    }
}

/// Builder for [`ElectionCount`].
#[derive(Debug, Clone)]
pub struct ElectionCountBuilder {
    count_type: Option<CountType>,
    transaction_id: Option<TransactionId>,
    managing_authority: Option<ManagingAuthority>,
    creation_date_time: Option<CreationDateTime>,
    canonicalization_method: Option<CanonicalizationMethod>,
    count: Option<ElectionCountCount>,
    election_identifier: Option<ElectionCountElectionIdentifier>,
    contests: Vec<ElectionCountContest>,
}

impl ElectionCountBuilder {
    /// Create a new ElectionCountBuilder for building [`ElectionCount`] documents.
    pub fn new() -> Self {
        Self {
            count_type: None,
            transaction_id: None,
            managing_authority: None,
            creation_date_time: None,
            canonicalization_method: None,
            count: None,
            election_identifier: None,
            contests: vec![],
        }
    }

    /// Set the count type for the document.
    pub fn count_type(mut self, count_type: impl Into<CountType>) -> Self {
        self.count_type = Some(count_type.into());
        self
    }

    /// Set the transaction id for the document.
    pub fn transaction_id(mut self, transaction_id: impl Into<TransactionId>) -> Self {
        self.transaction_id = Some(transaction_id.into());
        self
    }

    /// Set the managing authority for the document.
    pub fn managing_authority(mut self, managing_authority: impl Into<ManagingAuthority>) -> Self {
        self.managing_authority = Some(managing_authority.into());
        self
    }

    /// Set the creation date and time for the document.
    pub fn creation_date_time(mut self, creation_date_time: impl Into<XsDateTime>) -> Self {
        self.creation_date_time = Some(CreationDateTime::new(creation_date_time.into()));
        self
    }

    /// Set the canonicalization method for the document.
    pub fn canonicalization_method(
        mut self,
        canonicalization_method: impl Into<CanonicalizationMethod>,
    ) -> Self {
        self.canonicalization_method = Some(canonicalization_method.into());
        self
    }

    /// Set the count for the document.
    ///
    /// You may either set the entire election count at once using this method,
    /// or use any of [`Self::election_identifier`], [`Self::contests`] and/or
    /// [`Self::push_contest`] to construct the individual components of the
    /// count document.
    pub fn count(mut self, count: impl Into<ElectionCountCount>) -> Self {
        self.count = Some(count.into());
        self
    }

    /// Set the election identifier for the document.
    ///
    /// This only has effect if the count was not set using the  [`Self::count`]
    /// method on this builder.
    pub fn election_identifier(
        mut self,
        election_identifier: impl Into<ElectionCountElectionIdentifier>,
    ) -> Self {
        self.election_identifier = Some(election_identifier.into());
        self
    }

    /// Set the contests for the document. This overrides any previously set contests.
    ///
    /// This only has effect if the count was not set using the  [`Self::count`]
    /// method on this builder.
    pub fn contests(mut self, contests: impl Into<Vec<ElectionCountContest>>) -> Self {
        self.contests = contests.into();
        self
    }

    /// Add a contest to the document.
    ///
    /// This only has effect if the count was not set using the  [`Self::count`]
    /// method on this builder.
    pub fn push_contest(mut self, contest: impl Into<ElectionCountContest>) -> Self {
        self.contests.push(contest.into());
        self
    }

    /// Build the [`ElectionCount`] document, returning an error if any of the required fields are missing.
    pub fn build(self) -> Result<ElectionCount, EMLError> {
        Ok(ElectionCount {
            count_type: self
                .count_type
                .ok_or_else(|| EMLErrorKind::MissingBuildProperty("count_type").without_span())?,
            transaction_id: self.transaction_id.ok_or_else(|| {
                EMLErrorKind::MissingBuildProperty("transaction_id").without_span()
            })?,
            managing_authority: self.managing_authority.ok_or_else(|| {
                EMLErrorKind::MissingBuildProperty("managing_authority").without_span()
            })?,
            creation_date_time: self.creation_date_time.ok_or_else(|| {
                EMLErrorKind::MissingBuildProperty("creation_date_time").without_span()
            })?,
            canonicalization_method: self.canonicalization_method,
            count: self.count.map_or_else(
                || {
                    if self.contests.is_empty() {
                        return Err(EMLErrorKind::MissingBuildProperty("contests").without_span());
                    }

                    Ok(ElectionCountCount::new(ElectionCountElection::new(
                        self.election_identifier.ok_or_else(|| {
                            EMLErrorKind::MissingBuildProperty("election_identifier").without_span()
                        })?,
                        self.contests,
                    )))
                },
                Ok,
            )?,
        })
    }
}

impl Default for ElectionCountBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Custom: root EML element with dynamic Id attribute and full namespace context.
impl<'xml> FromXml<'xml> for ElectionCount {
    fn matches(id: instant_xml::Id<'_>, _field: Option<instant_xml::Id<'_>>) -> bool {
        id == instant_xml::Id {
            ns: NS_EML,
            name: "EML",
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut instant_xml::Deserializer<'cx, 'xml>,
    ) -> Result<(), instant_xml::Error> {
        use instant_xml::{Accumulate, Error, de::Node};

        if into.is_some() {
            return Err(Error::DuplicateValue(field));
        }

        let mut transaction_id = <TransactionId as FromXml>::Accumulator::default();
        let mut managing_authority = <ManagingAuthority as FromXml>::Accumulator::default();
        let mut creation_date_time = <CreationDateTime as FromXml>::Accumulator::default();
        let mut canonicalization_method =
            <CanonicalizationMethod as FromXml>::Accumulator::default();
        let mut count = <ElectionCountCount as FromXml>::Accumulator::default();

        while let Some(node) = deserializer.next() {
            let element = match node? {
                Node::Open(element) => element,
                Node::Text(s) if s.trim().is_empty() => continue,
                node => return Err(Error::UnexpectedNode(format!("{node:?}"))),
            };

            let id = deserializer.element_id(&element)?;
            if TransactionId::matches(id, None) {
                let mut nested = deserializer.nested(element);
                TransactionId::deserialize(&mut transaction_id, field, &mut nested)?;
                nested.ignore()?;
            } else if ManagingAuthority::matches(id, None) {
                let mut nested = deserializer.nested(element);
                ManagingAuthority::deserialize(&mut managing_authority, field, &mut nested)?;
                nested.ignore()?;
            } else if CreationDateTime::matches(id, None) {
                let mut nested = deserializer.nested(element);
                CreationDateTime::deserialize(&mut creation_date_time, field, &mut nested)?;
                nested.ignore()?;
            } else if CanonicalizationMethod::matches(id, None) {
                let mut nested = deserializer.nested(element);
                CanonicalizationMethod::deserialize(
                    &mut canonicalization_method,
                    field,
                    &mut nested,
                )?;
                nested.ignore()?;
            } else if ElectionCountCount::matches(id, None) {
                let mut nested = deserializer.nested(element);
                ElectionCountCount::deserialize(&mut count, field, &mut nested)?;
                nested.ignore()?;
            } else {
                let mut nested = deserializer.nested(element);
                nested.ignore()?;
            }
        }

        *into = Some(ElectionCount {
            count_type: CountType::Municipal, // Default; overridden by EML dispatch
            transaction_id: transaction_id.try_done(field)?,
            managing_authority: managing_authority.try_done(field)?,
            creation_date_time: creation_date_time.try_done(field)?,
            canonicalization_method,
            count: count.try_done(field)?,
        });
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Element;
}

// Custom: root EML element with Id/SchemaVersion attributes and full namespace context.
impl ToXml for ElectionCount {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix = serializer.write_start("EML", NS_EML, Some(eml_ns_context()))?;
        serializer.write_attr("Id", "", self.count_type.to_eml_id())?;
        serializer.write_attr("SchemaVersion", "", EML_SCHEMA_VERSION)?;
        serializer.end_start()?;
        self.transaction_id.serialize(None, serializer)?;
        self.managing_authority.serialize(None, serializer)?;
        self.creation_date_time.serialize(None, serializer)?;
        self.count.serialize(None, serializer)?;
        serializer.write_close(prefix)
    }
}

/// EML document ID for Count of polling stationdocuments.
pub(crate) const EML_COUNT_POLLING_STATION_ID: &str = "510a";

/// EML document ID for Count of municipality documents.
pub(crate) const EML_COUNT_MUNICIPAL_ID: &str = "510b";

/// EML document ID for Count of district documents.
pub(crate) const EML_COUNT_DISTRICT_ID: &str = "510c";

/// EML document ID for central Count documents.
pub(crate) const EML_COUNT_CENTRAL_ID: &str = "510d";

/// Type of Count document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CountType {
    /// Representing a `510a` document, containing the count for a polling station.
    PollingStation,
    /// Representing a `510b` document, containing the count for a local region (= municipality).
    Municipal,
    /// Representing a `510c` document, containing the count for a district (= HSB).
    District,
    /// Representing a `510d` document, containing the count for the entire election.
    Central,
}

impl CountType {
    /// Create a CountType from an EML document ID string.
    pub fn from_eml_id(s: impl AsRef<str>) -> Result<Self, EMLError> {
        let data = s.as_ref();
        match data {
            EML_COUNT_POLLING_STATION_ID => Ok(CountType::PollingStation),
            EML_COUNT_MUNICIPAL_ID => Ok(CountType::Municipal),
            EML_COUNT_DISTRICT_ID => Ok(CountType::District),
            EML_COUNT_CENTRAL_ID => Ok(CountType::Central),
            _ => Err(
                EMLErrorKind::InvalidDocumentType("510a/510b/510c/510d", data.to_string())
                    .without_span(),
            ),
        }
    }

    /// Get the EML document ID string for this CountType.
    pub fn to_eml_id(&self) -> &'static str {
        match self {
            CountType::PollingStation => EML_COUNT_POLLING_STATION_ID,
            CountType::Municipal => EML_COUNT_MUNICIPAL_ID,
            CountType::District => EML_COUNT_DISTRICT_ID,
            CountType::Central => EML_COUNT_CENTRAL_ID,
        }
    }

    /// Get a friendly name for this CountType.
    pub fn to_friendly_name(&self) -> &'static str {
        match self {
            CountType::PollingStation => "Polling Station Count",
            CountType::Municipal => "Municipal Count",
            CountType::District => "District Count",
            CountType::Central => "Central Count",
        }
    }

    /// Returns if the given EML document ID string is a valid CountType ID.
    pub fn is_valid_eml_id(s: &str) -> bool {
        matches!(
            s,
            EML_COUNT_POLLING_STATION_ID
                | EML_COUNT_MUNICIPAL_ID
                | EML_COUNT_DISTRICT_ID
                | EML_COUNT_CENTRAL_ID
        )
    }
}

/// The actual count data.
#[derive(Debug, Clone)]
pub struct ElectionCountCount {
    /// The election for this count.
    pub election: ElectionCountElection,
}

impl ElectionCountCount {
    /// Create a new count for the election count document.
    pub fn new(election: impl Into<ElectionCountElection>) -> Self {
        ElectionCountCount {
            election: election.into(),
        }
    }
}

impl From<ElectionCountElection> for ElectionCountCount {
    fn from(value: ElectionCountElection) -> Self {
        ElectionCountCount::new(value)
    }
}

// Custom: skips `<EventIdentifier/>` element not present in the struct.
impl<'xml> FromXml<'xml> for ElectionCountCount {
    fn matches(id: instant_xml::Id<'_>, _field: Option<instant_xml::Id<'_>>) -> bool {
        id == instant_xml::Id {
            ns: NS_EML,
            name: "Count",
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut instant_xml::Deserializer<'cx, 'xml>,
    ) -> Result<(), instant_xml::Error> {
        use instant_xml::{Accumulate, Error, de::Node};

        if into.is_some() {
            return Err(Error::DuplicateValue(field));
        }

        let mut election = <ElectionCountElection as FromXml>::Accumulator::default();
        while let Some(node) = deserializer.next() {
            let element = match node? {
                Node::Open(element) => element,
                Node::Text(s) if s.trim().is_empty() => continue,
                node => return Err(Error::UnexpectedNode(format!("{node:?}"))),
            };

            let id = deserializer.element_id(&element)?;
            if ElectionCountElection::matches(id, None) {
                let mut nested = deserializer.nested(element);
                ElectionCountElection::deserialize(&mut election, field, &mut nested)?;
                nested.ignore()?;
            } else {
                // Skip EventIdentifier and other unknown elements
                let mut nested = deserializer.nested(element);
                nested.ignore()?;
            }
        }

        *into = Some(ElectionCountCount {
            election: election.try_done(field)?,
        });
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Element;
}

// Custom: injects empty `<EventIdentifier/>` element not present in the struct.
impl ToXml for ElectionCountCount {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix = serializer.write_start("Count", NS_EML, None::<Context<0>>)?;
        serializer.end_start()?;
        let _ = serializer.write_start("EventIdentifier", NS_EML, None::<Context<0>>)?;
        serializer.end_empty()?;
        self.election.serialize(None, serializer)?;
        serializer.write_close(prefix)
    }
}

/// The election for this count.
#[derive(Debug, Clone)]
pub struct ElectionCountElection {
    /// Identifier
    pub identifier: ElectionCountElectionIdentifier,

    /// Contests within this election.
    pub contests: Vec<ElectionCountContest>,
}

impl ElectionCountElection {
    /// Create a new election for the election count document.
    pub fn new(
        identifier: impl Into<ElectionCountElectionIdentifier>,
        contests: impl Into<Vec<ElectionCountContest>>,
    ) -> Self {
        ElectionCountElection {
            identifier: identifier.into(),
            contests: contests.into(),
        }
    }
}

// Custom: unwraps `<Contests>` wrapper element not present in the struct.
impl<'xml> FromXml<'xml> for ElectionCountElection {
    fn matches(id: instant_xml::Id<'_>, _field: Option<instant_xml::Id<'_>>) -> bool {
        id == instant_xml::Id {
            ns: NS_EML,
            name: "Election",
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut instant_xml::Deserializer<'cx, 'xml>,
    ) -> Result<(), instant_xml::Error> {
        use instant_xml::{Accumulate, Error, de::Node};

        if into.is_some() {
            return Err(Error::DuplicateValue(field));
        }

        let mut identifier = <ElectionCountElectionIdentifier as FromXml>::Accumulator::default();
        let mut contests = <Vec<ElectionCountContest> as FromXml>::Accumulator::default();

        while let Some(node) = deserializer.next() {
            let element = match node? {
                Node::Open(element) => element,
                Node::Text(s) if s.trim().is_empty() => continue,
                node => return Err(Error::UnexpectedNode(format!("{node:?}"))),
            };

            let id = deserializer.element_id(&element)?;
            if ElectionCountElectionIdentifier::matches(id, None) {
                let mut nested = deserializer.nested(element);
                ElectionCountElectionIdentifier::deserialize(&mut identifier, field, &mut nested)?;
                nested.ignore()?;
            } else if id
                == (instant_xml::Id {
                    ns: NS_EML,
                    name: "Contests",
                })
            {
                // Unwrap the <Contests> wrapper and deserialize contests inside
                let mut nested = deserializer.nested(element);
                while let Some(inner) = nested.next() {
                    let inner = match inner? {
                        Node::Open(inner) => inner,
                        Node::Text(s) if s.trim().is_empty() => continue,
                        node => return Err(Error::UnexpectedNode(format!("{node:?}"))),
                    };

                    let inner_id = nested.element_id(&inner)?;
                    if <Vec<ElectionCountContest> as FromXml>::matches(inner_id, None) {
                        let mut inner_nested = nested.nested(inner);
                        <Vec<ElectionCountContest> as FromXml>::deserialize(
                            &mut contests,
                            field,
                            &mut inner_nested,
                        )?;
                        inner_nested.ignore()?;
                    } else {
                        let mut inner_nested = nested.nested(inner);
                        inner_nested.ignore()?;
                    }
                }
            } else {
                let mut nested = deserializer.nested(element);
                nested.ignore()?;
            }
        }

        *into = Some(ElectionCountElection {
            identifier: identifier.try_done(field)?,
            contests: contests.try_done(field)?,
        });
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Element;
}

// Custom: wraps contests in a `<Contests>` wrapper element not present in the struct.
impl ToXml for ElectionCountElection {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix = serializer.write_start("Election", NS_EML, None::<Context<0>>)?;
        serializer.end_start()?;
        self.identifier.serialize(None, serializer)?;

        let contests_prefix = serializer.write_start("Contests", NS_EML, None::<Context<0>>)?;
        serializer.end_start()?;
        for contest in &self.contests {
            contest.serialize(None, serializer)?;
        }

        serializer.write_close(contests_prefix)?;
        serializer.write_close(prefix)
    }
}

/// Identifier for the election in this count.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "ElectionIdentifier", ns(NS_EML, kr = NS_KR))]
pub struct ElectionCountElectionIdentifier {
    /// Id of the election
    #[xml(attribute, rename = "Id")]
    pub id: StringValue<ElectionId>,

    /// Name of the election
    #[xml(rename = "ElectionName")]
    pub name: Option<String>,

    /// Category of the election
    #[xml(rename = "ElectionCategory")]
    pub category: StringValue<ElectionCategory>,

    /// Subcategory of the election
    #[xml(rename = "ElectionSubcategory", ns(NS_KR))]
    pub subcategory: Option<StringValue<ElectionSubcategory>>,

    /// The (top level) region where the election takes place.
    #[xml(rename = "ElectionDomain")]
    pub domain: Option<ElectionDomain>,

    /// Date of the election
    #[xml(rename = "ElectionDate", ns(NS_KR))]
    pub election_date: StringValue<XsDate>,
}

impl ElectionCountElectionIdentifier {
    /// Create a builder for the [`ElectionCountElectionIdentifier`].
    pub fn builder() -> ElectionIdentifierBuilder {
        ElectionIdentifierBuilder::new()
    }
}

/// A contest within the election count.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "Contest", ns(NS_EML))]
pub struct ElectionCountContest {
    /// Identifier for the contest.
    #[xml(rename = "ContestIdentifier")]
    pub identifier: ContestIdentifier,

    /// Total votes in this contest, if present.
    #[xml(rename = "TotalVotes")]
    pub total_votes: Option<TotalVotes>,

    /// Votes per reporting unit in this contest.
    #[xml(rename = "ReportingUnitVotes")]
    pub reporting_unit_votes: Vec<ReportingUnitVotes>,
}

impl ElectionCountContest {
    /// Create a builder for the [`ElectionCountContest`].
    pub fn builder() -> ElectionCountContestBuilder {
        ElectionCountContestBuilder::new()
    }
}

/// A builder for [`ElectionCountContest`].
#[derive(Debug, Clone)]
pub struct ElectionCountContestBuilder {
    identifier: Option<ContestIdentifier>,
    total_votes: Option<TotalVotes>,
    total_votes_selections: Vec<ElectionCountSelection>,
    total_eligible_voter_count: Option<StringValue<u64>>,
    total_candidate_votes_count: Option<StringValue<u64>>,
    total_rejected_votes: BTreeMap<RejectedVotesReason, StringValue<u64>>,
    total_uncounted_votes: BTreeMap<UncountedVotesReason, StringValue<u64>>,
    reporting_unit_votes: Vec<ReportingUnitVotes>,
}

impl ElectionCountContestBuilder {
    /// Create a new ElectionCountContestBuilder for building [`ElectionCountContest`] documents.
    pub fn new() -> Self {
        Self {
            identifier: None,
            total_votes: None,
            total_votes_selections: vec![],
            total_eligible_voter_count: None,
            total_candidate_votes_count: None,
            total_rejected_votes: BTreeMap::new(),
            total_uncounted_votes: BTreeMap::new(),
            reporting_unit_votes: vec![],
        }
    }

    /// Set the identifier for the contest.
    pub fn identifier(mut self, identifier: impl Into<ContestIdentifier>) -> Self {
        self.identifier = Some(identifier.into());
        self
    }

    /// Set the total votes for the contest.
    pub fn total_votes(mut self, total_votes: impl Into<TotalVotes>) -> Self {
        self.total_votes = Some(total_votes.into());
        self
    }

    /// Set the selections within the total votes for the contest. This overrides any previously set selections.
    pub fn total_votes_selections(
        mut self,
        selections: impl Into<Vec<ElectionCountSelection>>,
    ) -> Self {
        self.total_votes_selections = selections.into();
        self
    }

    /// Add a selection to the selections within the total votes for the contest.
    pub fn push_total_votes_selection(
        mut self,
        selection: impl Into<ElectionCountSelection>,
    ) -> Self {
        self.total_votes_selections.push(selection.into());
        self
    }

    /// Set the total number of eligible voters within the contest.
    pub fn total_eligible_voter_count(mut self, count: impl Into<u64>) -> Self {
        self.total_eligible_voter_count = Some(StringValue::from_value(count.into()));
        self
    }

    /// Set the total number of votes on candidates within the contest.
    pub fn total_candidate_votes_count(mut self, count: impl Into<u64>) -> Self {
        self.total_candidate_votes_count = Some(StringValue::from_value(count.into()));
        self
    }

    /// Set the total number of rejected votes within the contest for a given reason.
    pub fn total_rejected_votes(
        mut self,
        reason: RejectedVotesReason,
        count: impl Into<u64>,
    ) -> Self {
        self.total_rejected_votes
            .insert(reason, StringValue::from_value(count.into()));
        self
    }

    /// Set the total number of uncounted votes within the contest for a given reason.
    pub fn total_uncounted_votes(
        mut self,
        reason: UncountedVotesReason,
        count: impl Into<u64>,
    ) -> Self {
        self.total_uncounted_votes
            .insert(reason, StringValue::from_value(count.into()));
        self
    }

    /// Set the details for all the reporting units within the contest. This
    /// overrides any previously set reporting unit votes.
    pub fn reporting_unit_votes(
        mut self,
        reporting_unit_votes: impl Into<Vec<ReportingUnitVotes>>,
    ) -> Self {
        self.reporting_unit_votes = reporting_unit_votes.into();
        self
    }

    /// Add the details for a reporting unit within the contest.
    pub fn push_reporting_unit_votes(
        mut self,
        reporting_unit_votes: impl Into<ReportingUnitVotes>,
    ) -> Self {
        self.reporting_unit_votes.push(reporting_unit_votes.into());
        self
    }

    /// Build the [`ElectionCountContest`] document, returning an error if any of the required fields are missing.
    pub fn build(self) -> Result<ElectionCountContest, EMLError> {
        Ok(ElectionCountContest {
            identifier: self
                .identifier
                .ok_or_else(|| EMLErrorKind::MissingBuildProperty("identifier").without_span())?,
            total_votes: self.total_votes.map_or_else(
                || {
                    if self.total_votes_selections.is_empty()
                        && self.total_eligible_voter_count.is_none()
                        && self.total_candidate_votes_count.is_none()
                        && self.total_rejected_votes.is_empty()
                        && self.total_uncounted_votes.is_empty()
                    {
                        Ok(None)
                    } else {
                        if self.total_votes_selections.is_empty() {
                            return Err(EMLErrorKind::MissingBuildProperty(
                                "total_votes_selections",
                            )
                            .without_span());
                        }

                        if !self
                            .total_rejected_votes
                            .contains_key(&RejectedVotesReason::Blank)
                        {
                            return Err(EMLErrorKind::MissingRejectedVotesBlank).without_span();
                        }

                        if !self
                            .total_rejected_votes
                            .contains_key(&RejectedVotesReason::Invalid)
                        {
                            return Err(EMLErrorKind::MissingRejectedVotesInvalid).without_span();
                        }

                        Ok(Some(TotalVotes {
                            selections: self.total_votes_selections,
                            eligible_voter_count: self.total_eligible_voter_count.ok_or_else(
                                || {
                                    EMLErrorKind::MissingBuildProperty("total_eligible_voter_count")
                                        .without_span()
                                },
                            )?,
                            candidate_votes_count: self.total_candidate_votes_count.ok_or_else(
                                || {
                                    EMLErrorKind::MissingBuildProperty(
                                        "total_candidate_votes_count",
                                    )
                                    .without_span()
                                },
                            )?,
                            rejected_votes: self.total_rejected_votes,
                            uncounted_votes: self.total_uncounted_votes,
                        }))
                    }
                },
                |v| Ok(Some(v)),
            )?,
            reporting_unit_votes: self.reporting_unit_votes,
        })
    }
}

impl Default for ElectionCountContestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Total votes in a contest.
#[derive(Debug, Clone)]
pub struct TotalVotes {
    /// Selections within the total votes.
    pub selections: Vec<ElectionCountSelection>,

    /// Total number of eligible voters within the reporting unit votes.
    ///
    /// This element is called `Cast` within EML_NL, but is renamed here to
    /// `eligible_voter_count` for clarity. This element was repurposed in
    /// EML_NL to represent the total number of eligible voters instead of the
    /// actual number of cast votes.
    pub eligible_voter_count: StringValue<u64>,

    /// Total number of votes on candidates.
    ///
    /// This element is called `TotalCounted` within EML_NL, but is renamed here
    /// to `candidate_votes_count` for clarity. This element was repurposed in
    /// EML_NL to represent the total number of votes on candidates instead of
    /// the actual total number of counted votes.
    pub candidate_votes_count: StringValue<u64>,

    /// Rejected votes within the reporting unit votes.
    ///
    /// Contains blank and invalid votes.
    pub rejected_votes: BTreeMap<RejectedVotesReason, StringValue<u64>>,

    /// Uncounted votes within the reporting unit votes.
    pub uncounted_votes: BTreeMap<UncountedVotesReason, StringValue<u64>>,
}

impl TotalVotes {
    /// Return the votes of each affiliation with each of their candidates and
    /// the amount of votes they received.
    ///
    /// This errors if any Selections of type ReferendumOption are encountered.
    pub fn selections_per_affiliation(
        &self,
    ) -> Result<Vec<SelectionAffiliationVotes<'_>>, EMLError> {
        selections_per_affiliation(&self.selections)
    }

    /// Return the total number of blank votes.
    pub fn blank_votes(&self) -> Result<&StringValue<u64>, EMLError> {
        self.rejected_votes
            .get(&RejectedVotesReason::Blank)
            .ok_or_else(|| EMLErrorKind::MissingRejectedVotesBlank.without_span())
    }

    /// Return the total number of invalid votes.
    pub fn invalid_votes(&self) -> Result<&StringValue<u64>, EMLError> {
        self.rejected_votes
            .get(&RejectedVotesReason::Invalid)
            .ok_or_else(|| EMLErrorKind::MissingRejectedVotesInvalid.without_span())
    }
}

// Custom: parses vote elements with ReasonCode attributes into BTreeMaps.
impl<'xml> FromXml<'xml> for TotalVotes {
    fn matches(id: instant_xml::Id<'_>, field: Option<instant_xml::Id<'_>>) -> bool {
        match field {
            Some(field) => id == field,
            None => {
                id == instant_xml::Id {
                    ns: NS_EML,
                    name: "TotalVotes",
                }
            }
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut instant_xml::Deserializer<'cx, 'xml>,
    ) -> Result<(), instant_xml::Error> {
        use instant_xml::{Accumulate, Error, de::Node};

        if into.is_some() {
            return Err(Error::DuplicateValue(field));
        }

        let mut selections = Vec::new();
        let mut eligible_voter_count = None;
        let mut candidate_votes_count = None;
        let mut rejected_votes = BTreeMap::new();
        let mut uncounted_votes = BTreeMap::new();

        while let Some(node) = deserializer.next() {
            let element = match node? {
                Node::Open(element) => element,
                Node::Text(s) if s.trim().is_empty() => continue,
                node => return Err(Error::UnexpectedNode(format!("{node:?}"))),
            };

            let id = deserializer.element_id(&element)?;
            if ElectionCountSelection::matches(id, None) {
                let mut acc = <ElectionCountSelection as FromXml<'xml>>::Accumulator::default();
                let mut nested = deserializer.nested(element);
                ElectionCountSelection::deserialize(&mut acc, field, &mut nested)?;
                nested.ignore()?;
                selections.push(acc.try_done(field)?);
            } else if id
                == (instant_xml::Id {
                    ns: NS_EML,
                    name: "Cast",
                })
            {
                let mut nested = deserializer.nested(element);
                let mut acc = <StringValue<u64> as FromXml<'xml>>::Accumulator::default();
                StringValue::<u64>::deserialize(&mut acc, "Cast", &mut nested)?;
                nested.ignore()?;
                eligible_voter_count = acc;
            } else if id
                == (instant_xml::Id {
                    ns: NS_EML,
                    name: "TotalCounted",
                })
            {
                let mut nested = deserializer.nested(element);
                let mut acc = <StringValue<u64> as FromXml<'xml>>::Accumulator::default();
                StringValue::<u64>::deserialize(&mut acc, "TotalCounted", &mut nested)?;
                nested.ignore()?;
                candidate_votes_count = acc;
            } else if id
                == (instant_xml::Id {
                    ns: NS_EML,
                    name: "RejectedVotes",
                })
            {
                let mut nested = deserializer.nested(element);
                deserialize_reason_map(&mut nested, &mut rejected_votes, |s| {
                    RejectedVotesReason::from_eml_value(s)
                })?;
            } else if id
                == (instant_xml::Id {
                    ns: NS_EML,
                    name: "UncountedVotes",
                })
            {
                let mut nested = deserializer.nested(element);
                deserialize_reason_map(&mut nested, &mut uncounted_votes, |s| {
                    UncountedVotesReason::from_eml_value(s)
                })?;
            } else {
                let mut nested = deserializer.nested(element);
                nested.ignore()?;
            }
        }

        *into = Some(TotalVotes {
            selections,
            eligible_voter_count: eligible_voter_count
                .ok_or(instant_xml::Error::MissingValue("Cast"))?,
            candidate_votes_count: candidate_votes_count
                .ok_or(instant_xml::Error::MissingValue("TotalCounted"))?,
            rejected_votes,
            uncounted_votes,
        });
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Element;
}

/// Helper to deserialize `<Element ReasonCode="...">value</Element>` into a BTreeMap.
fn deserialize_reason_map<K: Ord, V: crate::utils::StringValueData, E: std::fmt::Display>(
    deserializer: &mut instant_xml::Deserializer<'_, '_>,
    map: &mut BTreeMap<K, StringValue<V>>,
    parse_reason: impl for<'a> Fn(&'a str) -> Result<K, E>,
) -> Result<(), instant_xml::Error> {
    use instant_xml::de::Node;
    let mut reason_code = None;
    let mut text = None;
    for node in deserializer.by_ref() {
        let node = node?;
        match node {
            Node::Attribute(attr) if attr.local == "ReasonCode" => {
                reason_code = Some(attr.value.to_string());
            }
            Node::AttributeValue(_) => {}
            Node::Text(s) => {
                text = Some(s.into_owned());
            }
            _ => {}
        }
    }
    if let Some(rc) = reason_code {
        let key = parse_reason(&rc).map_err(|e| instant_xml::Error::Other(format!("{e}")))?;
        let raw = text.unwrap_or_default();
        let sv = match V::parse_from_str(&raw) {
            Ok(v) => StringValue::Parsed(v),
            Err(_) => StringValue::Raw(raw),
        };
        map.insert(key, sv);
    }
    Ok(())
}

// Custom: serializes vote maps as elements with ReasonCode attributes.
impl ToXml for TotalVotes {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix = serializer.write_start("TotalVotes", NS_EML, None::<Context<0>>)?;
        serializer.end_start()?;
        for selection in &self.selections {
            selection.serialize(None, serializer)?;
        }

        self.eligible_voter_count.serialize(
            Some(instant_xml::Id {
                ns: NS_EML,
                name: "Cast",
            }),
            serializer,
        )?;

        self.candidate_votes_count.serialize(
            Some(instant_xml::Id {
                ns: NS_EML,
                name: "TotalCounted",
            }),
            serializer,
        )?;

        for (reason, count) in &self.rejected_votes {
            let rv_prefix = serializer.write_start("RejectedVotes", NS_EML, None::<Context<0>>)?;
            serializer.write_attr("ReasonCode", "", reason.to_eml_value())?;
            serializer.end_start()?;
            serializer.write_str(count.raw().as_ref())?;
            serializer.write_close(rv_prefix)?;
        }

        for (reason, count) in &self.uncounted_votes {
            let uv_prefix = serializer.write_start("UncountedVotes", NS_EML, None::<Context<0>>)?;
            serializer.write_attr("ReasonCode", "", reason.to_eml_value())?;
            serializer.end_start()?;
            serializer.write_str(count.raw().as_ref())?;
            serializer.write_close(uv_prefix)?;
        }

        serializer.write_close(prefix)
    }
}

/// Votes per reporting unit.
#[derive(Debug, Clone)]
pub struct ReportingUnitVotes {
    /// Identifier for the reporting unit votes.
    pub identifier: ReportingUnitIdentifier,

    /// Selections within the reporting unit votes.
    pub selections: Vec<ElectionCountSelection>,

    /// Total number of eligible voters within the reporting unit votes.
    ///
    /// This element is called `Cast` within EML_NL, but is renamed here to
    /// `eligible_voter_count` for clarity. This element was repurposed in
    /// EML_NL to represent the total number of eligible voters instead of the
    /// actual number of cast votes.
    pub eligible_voter_count: StringValue<u64>,

    /// Total number of votes on candidates.
    ///
    /// This element is called `TotalCounted` within EML_NL, but is renamed here
    /// to `candidate_votes_count` for clarity. This element was repurposed in
    /// EML_NL to represent the total number of votes on candidates instead of
    /// the actual total number of counted votes.
    pub candidate_votes_count: StringValue<u64>,

    /// Rejected votes within the reporting unit votes.
    ///
    /// Contains blank and invalid votes.
    pub rejected_votes: BTreeMap<RejectedVotesReason, StringValue<u64>>,

    /// Uncounted votes within the reporting unit votes.
    pub uncounted_votes: BTreeMap<UncountedVotesReason, StringValue<u64>>,

    /// Investigations within the reporting unit votes.
    pub investigations: BTreeMap<InvestigationReason, StringValue<bool>>,
}

/// Gathered votes and candidates for an affiliation selection within a reporting unit or total votes for a contest.
pub struct SelectionAffiliationVotes<'a> {
    /// The affiliation selection for which the votes were gathered.
    pub affiliation: &'a AffiliationSelection,

    /// The total number of valid votes for this affiliation.
    pub valid_votes: u64,

    /// The candidates for this affiliation and the number of votes they received.
    pub candidates: Vec<SelectionCandidateVotes<'a>>,
}

/// Gathered votes for a candidate selection within some affiliation.
pub struct SelectionCandidateVotes<'a> {
    /// The affiliation to which the candidate belongs.
    pub affiliation: &'a AffiliationSelection,

    /// The candidate selection for which the votes were gathered.
    pub candidate: &'a CandidateSelection,

    /// The total number of valid votes for this candidate.
    pub valid_votes: u64,
}

impl ReportingUnitVotes {
    /// Create a builder for the [`ReportingUnitVotes`].
    pub fn builder() -> ReportingUnitVotesBuilder {
        ReportingUnitVotesBuilder::new()
    }

    /// Return the votes of each affiliation with each of their candidates and
    /// the amount of votes they received.
    ///
    /// This errors if any Selections of type ReferendumOption are encountered.
    pub fn selections_per_affiliation(
        &self,
    ) -> Result<Vec<SelectionAffiliationVotes<'_>>, EMLError> {
        selections_per_affiliation(&self.selections)
    }

    /// Return the number of blank votes for this reporting unit.
    pub fn blank_votes(&self) -> Result<&StringValue<u64>, EMLError> {
        self.rejected_votes
            .get(&RejectedVotesReason::Blank)
            .ok_or_else(|| EMLErrorKind::MissingRejectedVotesBlank.without_span())
    }

    /// Return the number of invalid votes for this reporting unit.
    pub fn invalid_votes(&self) -> Result<&StringValue<u64>, EMLError> {
        self.rejected_votes
            .get(&RejectedVotesReason::Invalid)
            .ok_or_else(|| EMLErrorKind::MissingRejectedVotesInvalid.without_span())
    }
}

fn selections_per_affiliation(
    selections: &[ElectionCountSelection],
) -> Result<Vec<SelectionAffiliationVotes<'_>>, EMLError> {
    let mut result = Vec::new();

    // Selections consist an affiliation selection followed by zero or more
    // candidate selections for that affiliation.
    let mut current_affiliation = None;
    let mut current_affiliation_selection: Option<&ElectionCountSelection> = None;
    let mut current_candidates = vec![];

    for selection in selections {
        match &selection.selection_type {
            ElectionCountSelectionType::Affiliation(affiliation) => {
                if let (Some(ca), Some(cas)) = (current_affiliation, current_affiliation_selection)
                {
                    result.push(SelectionAffiliationVotes {
                        affiliation: ca,
                        valid_votes: cas
                            .valid_votes
                            .copied_value()
                            .wrap_field_value_error(("ValidVotes", NS_EML))?,
                        candidates: current_candidates,
                    });
                }

                current_affiliation = Some(affiliation);
                current_affiliation_selection = Some(selection);
                current_candidates = vec![];
            }
            ElectionCountSelectionType::Candidate(candidate) => {
                // Candidate selections should only be encountered after an affiliation selection.
                let curr_aff = current_affiliation
                    .ok_or_else(|| EMLErrorKind::CandidateWithoutAffiliationFound.without_span())?;

                current_candidates.push(SelectionCandidateVotes {
                    affiliation: curr_aff,
                    candidate,
                    valid_votes: selection
                        .valid_votes
                        .copied_value()
                        .wrap_field_value_error(("ValidVotes", NS_EML))?,
                });
            }
            // Referendum option selections should not be encountered at all.
            ElectionCountSelectionType::ReferendumOption(_) => {
                return Err(EMLErrorKind::UnexpectedReferendumOptionSelection.without_span());
            }
        }
    }

    // push the last affiliation and its candidates if it exists
    if let (Some(ca), Some(cas)) = (current_affiliation, current_affiliation_selection) {
        result.push(SelectionAffiliationVotes {
            affiliation: ca,
            valid_votes: cas
                .valid_votes
                .copied_value()
                .wrap_field_value_error(("ValidVotes", NS_EML))?,
            candidates: current_candidates,
        });
    }

    Ok(result)
}

/// A builder for [`ReportingUnitVotes`].
#[derive(Debug, Clone)]
pub struct ReportingUnitVotesBuilder {
    identifier: Option<ReportingUnitIdentifier>,
    selections: Vec<ElectionCountSelection>,
    eligible_voter_count: Option<StringValue<u64>>,
    candidate_votes_count: Option<StringValue<u64>>,
    rejected_votes: BTreeMap<RejectedVotesReason, StringValue<u64>>,
    uncounted_votes: BTreeMap<UncountedVotesReason, StringValue<u64>>,
    investigations: BTreeMap<InvestigationReason, StringValue<bool>>,
}

impl ReportingUnitVotesBuilder {
    /// Create a new ReportingUnitVotesBuilder for building [`ReportingUnitVotes`] documents.
    pub fn new() -> Self {
        Self {
            identifier: None,
            selections: vec![],
            eligible_voter_count: None,
            candidate_votes_count: None,
            rejected_votes: BTreeMap::new(),
            uncounted_votes: BTreeMap::new(),
            investigations: BTreeMap::new(),
        }
    }

    /// Set the identifier for the reporting unit votes.
    pub fn identifier(mut self, identifier: impl Into<ReportingUnitIdentifier>) -> Self {
        self.identifier = Some(identifier.into());
        self
    }

    /// Set the selections within the reporting unit votes. This overrides any previously set selections.
    pub fn selections(mut self, selections: impl Into<Vec<ElectionCountSelection>>) -> Self {
        self.selections = selections.into();
        self
    }

    /// Add a selection to the selections within the reporting unit votes.
    pub fn push_selection(mut self, selection: impl Into<ElectionCountSelection>) -> Self {
        self.selections.push(selection.into());
        self
    }

    /// Set the total number of eligible voters within the reporting unit votes.
    pub fn eligible_voter_count(mut self, count: impl Into<u64>) -> Self {
        self.eligible_voter_count = Some(StringValue::from_value(count.into()));
        self
    }

    /// Set the total number of votes on candidates within the reporting unit votes.
    pub fn candidate_votes_count(mut self, count: impl Into<u64>) -> Self {
        self.candidate_votes_count = Some(StringValue::from_value(count.into()));
        self
    }

    /// Set the total number of rejected votes within the reporting unit votes for a given reason.
    pub fn rejected_votes(mut self, reason: RejectedVotesReason, count: impl Into<u64>) -> Self {
        self.rejected_votes
            .insert(reason, StringValue::from_value(count.into()));
        self
    }

    /// Set the total number of uncounted votes within the reporting unit votes for a given reason.
    pub fn uncounted_votes(mut self, reason: UncountedVotesReason, count: impl Into<u64>) -> Self {
        self.uncounted_votes
            .insert(reason, StringValue::from_value(count.into()));
        self
    }

    /// Set the investigations within the reporting unit votes for a given reason.
    pub fn investigation(mut self, reason: InvestigationReason, value: bool) -> Self {
        self.investigations
            .insert(reason, StringValue::from_value(value));
        self
    }

    /// Build the [`ReportingUnitVotes`] document, returning an error if any of the required fields are missing.
    pub fn build(self) -> Result<ReportingUnitVotes, EMLError> {
        if self.selections.is_empty() {
            return Err(EMLErrorKind::MissingBuildProperty("selections").without_span());
        }

        if !self
            .rejected_votes
            .contains_key(&RejectedVotesReason::Blank)
        {
            return Err(EMLErrorKind::MissingRejectedVotesBlank).without_span();
        }

        if !self
            .rejected_votes
            .contains_key(&RejectedVotesReason::Invalid)
        {
            return Err(EMLErrorKind::MissingRejectedVotesInvalid).without_span();
        }

        Ok(ReportingUnitVotes {
            identifier: self
                .identifier
                .ok_or_else(|| EMLErrorKind::MissingBuildProperty("identifier").without_span())?,
            selections: self.selections,
            eligible_voter_count: self.eligible_voter_count.ok_or_else(|| {
                EMLErrorKind::MissingBuildProperty("eligible_voter_count").without_span()
            })?,
            candidate_votes_count: self.candidate_votes_count.ok_or_else(|| {
                EMLErrorKind::MissingBuildProperty("candidate_votes_count").without_span()
            })?,
            rejected_votes: self.rejected_votes,
            uncounted_votes: self.uncounted_votes,
            investigations: self.investigations,
        })
    }
}

impl Default for ReportingUnitVotesBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Custom: unwraps conditional `<ReportingUnitInvestigations>` wrapper; vote maps with ReasonCode attributes.
impl<'xml> FromXml<'xml> for ReportingUnitVotes {
    fn matches(id: instant_xml::Id<'_>, field: Option<instant_xml::Id<'_>>) -> bool {
        match field {
            Some(field) => id == field,
            None => {
                id == instant_xml::Id {
                    ns: NS_EML,
                    name: "ReportingUnitVotes",
                }
            }
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut instant_xml::Deserializer<'cx, 'xml>,
    ) -> Result<(), instant_xml::Error> {
        use instant_xml::{Accumulate, Error, de::Node};

        if into.is_some() {
            return Err(Error::DuplicateValue(field));
        }

        let mut identifier = None;
        let mut selections = Vec::new();
        let mut eligible_voter_count = None;
        let mut candidate_votes_count = None;
        let mut rejected_votes = BTreeMap::new();
        let mut uncounted_votes = BTreeMap::new();
        let mut investigations = BTreeMap::new();

        while let Some(node) = deserializer.next() {
            let element = match node? {
                Node::Open(element) => element,
                Node::Text(s) if s.trim().is_empty() => continue,
                node => return Err(Error::UnexpectedNode(format!("{node:?}"))),
            };

            let id = deserializer.element_id(&element)?;
            if ReportingUnitIdentifier::matches(id, None) {
                let mut acc = <ReportingUnitIdentifier as FromXml<'xml>>::Accumulator::default();
                let mut nested = deserializer.nested(element);
                ReportingUnitIdentifier::deserialize(&mut acc, field, &mut nested)?;
                nested.ignore()?;
                identifier = acc;
            } else if ElectionCountSelection::matches(id, None) {
                let mut acc = <ElectionCountSelection as FromXml<'xml>>::Accumulator::default();
                let mut nested = deserializer.nested(element);
                ElectionCountSelection::deserialize(&mut acc, field, &mut nested)?;
                nested.ignore()?;
                selections.push(acc.try_done(field)?);
            } else if id
                == (instant_xml::Id {
                    ns: NS_EML,
                    name: "Cast",
                })
            {
                let mut nested = deserializer.nested(element);
                let mut acc = <StringValue<u64> as FromXml<'xml>>::Accumulator::default();
                StringValue::<u64>::deserialize(&mut acc, "Cast", &mut nested)?;
                nested.ignore()?;
                eligible_voter_count = acc;
            } else if id
                == (instant_xml::Id {
                    ns: NS_EML,
                    name: "TotalCounted",
                })
            {
                let mut nested = deserializer.nested(element);
                let mut acc = <StringValue<u64> as FromXml<'xml>>::Accumulator::default();
                StringValue::<u64>::deserialize(&mut acc, "TotalCounted", &mut nested)?;
                nested.ignore()?;
                candidate_votes_count = acc;
            } else if id
                == (instant_xml::Id {
                    ns: NS_EML,
                    name: "RejectedVotes",
                })
            {
                let mut nested = deserializer.nested(element);
                deserialize_reason_map(&mut nested, &mut rejected_votes, |s| {
                    RejectedVotesReason::from_eml_value(s)
                })?;
            } else if id
                == (instant_xml::Id {
                    ns: NS_EML,
                    name: "UncountedVotes",
                })
            {
                let mut nested = deserializer.nested(element);
                deserialize_reason_map(&mut nested, &mut uncounted_votes, |s| {
                    UncountedVotesReason::from_eml_value(s)
                })?;
            } else if id
                == (instant_xml::Id {
                    ns: NS_KR,
                    name: "Investigations",
                })
            {
                let mut nested = deserializer.nested(element);
                while let Some(node) = nested.next() {
                    let node = node?;
                    if let Node::Open(inv_elem) = node {
                        let mut inv_nested = nested.nested(inv_elem);
                        deserialize_reason_map(&mut inv_nested, &mut investigations, |s| {
                            InvestigationReason::from_eml_value(s)
                        })?;
                    }
                }
            } else {
                let mut nested = deserializer.nested(element);
                nested.ignore()?;
            }
        }

        *into = Some(ReportingUnitVotes {
            identifier: identifier
                .ok_or(instant_xml::Error::MissingValue("ReportingUnitIdentifier"))?,
            selections,
            eligible_voter_count: eligible_voter_count
                .ok_or(instant_xml::Error::MissingValue("Cast"))?,
            candidate_votes_count: candidate_votes_count
                .ok_or(instant_xml::Error::MissingValue("TotalCounted"))?,
            rejected_votes,
            uncounted_votes,
            investigations,
        });
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Element;
}

// Custom: conditional `<ReportingUnitInvestigations>` wrapper; vote maps with ReasonCode attributes.
impl ToXml for ReportingUnitVotes {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix = serializer.write_start("ReportingUnitVotes", NS_EML, None::<Context<0>>)?;
        serializer.end_start()?;
        self.identifier.serialize(None, serializer)?;
        for selection in &self.selections {
            selection.serialize(None, serializer)?;
        }

        self.eligible_voter_count.serialize(
            Some(instant_xml::Id {
                ns: NS_EML,
                name: "Cast",
            }),
            serializer,
        )?;

        self.candidate_votes_count.serialize(
            Some(instant_xml::Id {
                ns: NS_EML,
                name: "TotalCounted",
            }),
            serializer,
        )?;

        for (reason, count) in &self.rejected_votes {
            let rv_prefix = serializer.write_start("RejectedVotes", NS_EML, None::<Context<0>>)?;
            serializer.write_attr("ReasonCode", "", reason.to_eml_value())?;
            serializer.end_start()?;
            serializer.write_str(count.raw().as_ref())?;
            serializer.write_close(rv_prefix)?;
        }

        for (reason, count) in &self.uncounted_votes {
            let uv_prefix = serializer.write_start("UncountedVotes", NS_EML, None::<Context<0>>)?;
            serializer.write_attr("ReasonCode", "", reason.to_eml_value())?;
            serializer.end_start()?;
            serializer.write_str(count.raw().as_ref())?;
            serializer.write_close(uv_prefix)?;
        }

        if !self.investigations.is_empty() {
            let inv_prefix =
                serializer.write_start("ReportingUnitInvestigations", NS_KR, None::<Context<0>>)?;
            serializer.end_start()?;
            for (reason, investigated) in &self.investigations {
                let reason_prefix =
                    serializer.write_start("Investigation", NS_KR, None::<Context<0>>)?;
                serializer.write_attr("ReasonCode", "", reason.to_eml_value())?;
                serializer.end_start()?;
                serializer.write_str(investigated.raw().as_ref())?;
                serializer.write_close(reason_prefix)?;
            }
            serializer.write_close(inv_prefix)?;
        }

        serializer.write_close(prefix)
    }
}

/// Reason code for a specific investigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, FromXml, ToXml)]
#[xml(scalar)]
pub enum InvestigationReason {
    /// onderzocht vanwege onverklaard verschil
    #[xml(rename = "onderzocht vanwege onverklaard verschil")]
    UnexplainedDifference,
    /// onderzocht vanwege andere fout
    #[xml(rename = "onderzocht vanwege andere fout")]
    OtherError,
    /// uitslag gecorrigeerd
    #[xml(rename = "uitslag gecorrigeerd")]
    ResultCorrected,
    /// toegelaten kiezers opnieuw vastgesteld
    #[xml(rename = "toegelaten kiezers opnieuw vastgesteld")]
    AdmittedVotersReestablished,
    /// onderzocht vanwege andere reden
    #[xml(rename = "onderzocht vanwege andere reden")]
    OtherReason,
    /// stembiljetten deels herteld
    #[xml(rename = "stembiljetten deels herteld")]
    PartiallyRecountedBallots,
}

impl InvestigationReason {
    /// Create an InvestigationReason from an EML reason code string.
    pub fn from_eml_value(s: impl AsRef<str>) -> Result<Self, InvalidInvestigationReasonError> {
        let data = s.as_ref();
        match data {
            "onderzocht vanwege onverklaard verschil" => {
                Ok(InvestigationReason::UnexplainedDifference)
            }
            "onderzocht vanwege andere fout" => Ok(InvestigationReason::OtherError),
            "uitslag gecorrigeerd" => Ok(InvestigationReason::ResultCorrected),
            "toegelaten kiezers opnieuw vastgesteld" => {
                Ok(InvestigationReason::AdmittedVotersReestablished)
            }
            "onderzocht vanwege andere reden" => Ok(InvestigationReason::OtherReason),
            "stembiljetten deels herteld" => Ok(InvestigationReason::PartiallyRecountedBallots),
            _ => Err(InvalidInvestigationReasonError(data.to_string())),
        }
    }

    /// Get the EML reason code string for this InvestigationReason.
    pub fn to_eml_value(&self) -> &'static str {
        match self {
            InvestigationReason::UnexplainedDifference => "onderzocht vanwege onverklaard verschil",
            InvestigationReason::OtherError => "onderzocht vanwege andere fout",
            InvestigationReason::ResultCorrected => "uitslag gecorrigeerd",
            InvestigationReason::AdmittedVotersReestablished => {
                "toegelaten kiezers opnieuw vastgesteld"
            }
            InvestigationReason::OtherReason => "onderzocht vanwege andere reden",
            InvestigationReason::PartiallyRecountedBallots => "stembiljetten deels herteld",
        }
    }
}

/// Error indicating an invalid InvestigationReason.
#[derive(Debug, Clone, thiserror::Error)]
#[error("Invalid investigation reason: {0}")]
pub struct InvalidInvestigationReasonError(String);

/// Reason code for a specific uncounted votes entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, FromXml, ToXml)]
#[xml(scalar)]
pub enum UncountedVotesReason {
    /// geldige stempassen
    #[xml(rename = "geldige stempassen")]
    ValidPollCards,
    /// geldige volmachtbewijzen
    #[xml(rename = "geldige volmachtbewijzen")]
    ValidProxyCertificates,
    /// geldige kiezerspassen
    #[xml(rename = "geldige kiezerspassen")]
    ValidVoterCards,
    /// toegelaten kiezers
    #[xml(rename = "toegelaten kiezers")]
    AdmittedVoters,
    /// meer getelde stembiljetten
    #[xml(rename = "meer getelde stembiljetten")]
    MoreBallotsCounted,
    /// minder getelde stembiljetten
    #[xml(rename = "minder getelde stembiljetten")]
    FewerBallotsCounted,
    /// meegenomen stembiljetten
    #[xml(rename = "meegenomen stembiljetten")]
    BallotsTaken,
    /// te weinig uitgereikte stembiljetten
    #[xml(rename = "te weinig uitgereikte stembiljetten")]
    TooFewBallotsIssued,
    /// te veel uitgereikte stembiljetten
    #[xml(rename = "te veel uitgereikte stembiljetten")]
    TooManyBallotsIssued,
    /// geen briefstembiljetten
    #[xml(rename = "geen briefstembiljetten")]
    NoPostalBallots,
    /// te veel briefstembiljetten
    #[xml(rename = "te veel briefstembiljetten")]
    TooManyPostalBallots,
    /// kwijtgeraakte stembiljetten
    #[xml(rename = "kwijtgeraakte stembiljetten")]
    LostBallots,
    /// geen verklaring
    #[xml(rename = "geen verklaring")]
    NoExplanation,
    /// andere verklaring
    #[xml(rename = "andere verklaring")]
    OtherExplanation,
}

impl UncountedVotesReason {
    /// Create an UncountedVotesReason from an EML reason code string.
    pub fn from_eml_value(s: impl AsRef<str>) -> Result<Self, InvalidUncountedVotesReasonError> {
        match s.as_ref() {
            "geldige stempassen" => Ok(UncountedVotesReason::ValidPollCards),
            "geldige volmachtbewijzen" => Ok(UncountedVotesReason::ValidProxyCertificates),
            "geldige kiezerspassen" => Ok(UncountedVotesReason::ValidVoterCards),
            "toegelaten kiezers" => Ok(UncountedVotesReason::AdmittedVoters),
            "meer getelde stembiljetten" => Ok(UncountedVotesReason::MoreBallotsCounted),
            "minder getelde stembiljetten" => Ok(UncountedVotesReason::FewerBallotsCounted),
            "meegenomen stembiljetten" => Ok(UncountedVotesReason::BallotsTaken),
            "te weinig uitgereikte stembiljetten" => Ok(UncountedVotesReason::TooFewBallotsIssued),
            "te veel uitgereikte stembiljetten" => Ok(UncountedVotesReason::TooManyBallotsIssued),
            "geen briefstembiljetten" => Ok(UncountedVotesReason::NoPostalBallots),
            "te veel briefstembiljetten" => Ok(UncountedVotesReason::TooManyPostalBallots),
            "kwijtgeraakte stembiljetten" => Ok(UncountedVotesReason::LostBallots),
            "geen verklaring" => Ok(UncountedVotesReason::NoExplanation),
            "andere verklaring" => Ok(UncountedVotesReason::OtherExplanation),
            _ => Err(InvalidUncountedVotesReasonError(s.as_ref().to_string())),
        }
    }

    /// Get the EML reason code string for this UncountedVotesReason.
    pub fn to_eml_value(&self) -> &'static str {
        match self {
            UncountedVotesReason::ValidPollCards => "geldige stempassen",
            UncountedVotesReason::ValidProxyCertificates => "geldige volmachtbewijzen",
            UncountedVotesReason::ValidVoterCards => "geldige kiezerspassen",
            UncountedVotesReason::AdmittedVoters => "toegelaten kiezers",
            UncountedVotesReason::MoreBallotsCounted => "meer getelde stembiljetten",
            UncountedVotesReason::FewerBallotsCounted => "minder getelde stembiljetten",
            UncountedVotesReason::BallotsTaken => "meegenomen stembiljetten",
            UncountedVotesReason::TooFewBallotsIssued => "te weinig uitgereikte stembiljetten",
            UncountedVotesReason::TooManyBallotsIssued => "te veel uitgereikte stembiljetten",
            UncountedVotesReason::NoPostalBallots => "geen briefstembiljetten",
            UncountedVotesReason::TooManyPostalBallots => "te veel briefstembiljetten",
            UncountedVotesReason::LostBallots => "kwijtgeraakte stembiljetten",
            UncountedVotesReason::NoExplanation => "geen verklaring",
            UncountedVotesReason::OtherExplanation => "andere verklaring",
        }
    }
}

/// Error indicating an invalid uncounted votes reason.
#[derive(Debug, Clone, thiserror::Error)]
#[error("Invalid uncounted votes reason: {0}")]
pub struct InvalidUncountedVotesReasonError(String);

/// Reason code for rejected votes entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, FromXml, ToXml)]
#[xml(scalar)]
pub enum RejectedVotesReason {
    /// Blank votes ("blanco")
    #[xml(rename = "blanco")]
    Blank,
    /// Invalid votes ("ongeldig")
    #[xml(rename = "ongeldig")]
    Invalid,
}

impl RejectedVotesReason {
    /// Create a RejectedVotesReason from an EML reason code string.
    pub fn from_eml_value(s: impl AsRef<str>) -> Result<Self, InvalidRejectedVotesReasonError> {
        let data = s.as_ref();
        match data {
            "blanco" => Ok(RejectedVotesReason::Blank),
            "ongeldig" => Ok(RejectedVotesReason::Invalid),
            _ => Err(InvalidRejectedVotesReasonError(data.to_string())),
        }
    }

    /// Get the EML reason code string for this RejectedVotesReason.
    pub fn to_eml_value(&self) -> &'static str {
        match self {
            RejectedVotesReason::Blank => "blanco",
            RejectedVotesReason::Invalid => "ongeldig",
        }
    }
}

/// Error indicating an invalid rejected votes reason.
#[derive(Debug, Clone, thiserror::Error)]
#[error("Invalid rejected votes reason: {0}")]
pub struct InvalidRejectedVotesReasonError(String);

/// A selection within the reporting unit votes.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "Selection", rename_all = "PascalCase", ns(NS_EML))]
pub struct ElectionCountSelection {
    /// Type of selection.
    pub selection_type: ElectionCountSelectionType,

    /// Number of valid votes for this selection.
    pub valid_votes: StringValue<u64>,

    /// Value of the `Value` attribute, if present.
    #[xml(attribute)]
    pub value: Option<String>,

    /// Value of the `Category` attribute, if present.
    #[xml(attribute)]
    pub category: Option<String>,
}

impl ElectionCountSelection {
    /// Create a builder for the [`ElectionCountSelection`].
    pub fn builder() -> ElectionCountSelectionBuilder {
        ElectionCountSelectionBuilder::new()
    }
}

/// A builder for [`ElectionCountSelection`].
#[derive(Debug, Clone)]
pub struct ElectionCountSelectionBuilder {
    selection_type: Option<ElectionCountSelectionType>,
    valid_votes: Option<StringValue<u64>>,
    value: Option<String>,
    category: Option<String>,
}

impl ElectionCountSelectionBuilder {
    /// Create a new ElectionCountSelectionBuilder for building [`ElectionCountSelection`] documents.
    pub fn new() -> Self {
        Self {
            selection_type: None,
            valid_votes: None,
            value: None,
            category: None,
        }
    }

    /// Set the selection type to a candidate selection with the given candidate.
    pub fn candidate(mut self, candidate: impl Into<CandidateSelection>) -> Self {
        self.selection_type = Some(ElectionCountSelectionType::Candidate(Box::new(
            candidate.into(),
        )));
        self
    }

    /// Set the selection type to an affiliation selection with the given affiliation.
    pub fn affiliation(mut self, affiliation: impl Into<AffiliationSelection>) -> Self {
        self.selection_type = Some(ElectionCountSelectionType::Affiliation(Box::new(
            affiliation.into(),
        )));
        self
    }

    /// Set the selection type to a referendum option selection with the given referendum option.
    pub fn referendum_option(
        mut self,
        referendum_option: impl Into<ReferendumOptionSelection>,
    ) -> Self {
        self.selection_type = Some(ElectionCountSelectionType::ReferendumOption(Box::new(
            referendum_option.into(),
        )));
        self
    }

    /// Set the number of valid votes for the election count selection.
    pub fn valid_votes(mut self, valid_votes: impl Into<u64>) -> Self {
        self.valid_votes = Some(StringValue::from_value(valid_votes.into()));
        self
    }

    /// Set the Value attribute of the election count selection.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Set the Category attribute of the election count selection.
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Build the [`ElectionCountSelection`] document, returning an error if any of the required fields are missing.
    pub fn build(self) -> Result<ElectionCountSelection, EMLError> {
        Ok(ElectionCountSelection {
            selection_type: self.selection_type.ok_or_else(|| {
                EMLErrorKind::MissingBuildProperty("selection_type").without_span()
            })?,
            valid_votes: self
                .valid_votes
                .ok_or_else(|| EMLErrorKind::MissingBuildProperty("valid_votes").without_span())?,
            value: self.value,
            category: self.category,
        })
    }
}

impl Default for ElectionCountSelectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// The type of selection.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(forward)]
pub enum ElectionCountSelectionType {
    /// Selection of a candidate.
    Candidate(Box<CandidateSelection>),

    /// Selection of an affiliation.
    Affiliation(Box<AffiliationSelection>),

    /// Selection of a referendum option.
    ReferendumOption(Box<ReferendumOptionSelection>),
}

impl ElectionCountSelectionType {
    /// Check if the selection type is a candidate selection.
    pub fn is_candidate(&self) -> bool {
        matches!(self, ElectionCountSelectionType::Candidate(_))
    }

    /// Check if the selection type is an affiliation selection.
    pub fn is_affiliation(&self) -> bool {
        matches!(self, ElectionCountSelectionType::Affiliation(_))
    }

    /// Check if the selection type is a referendum option selection.
    pub fn is_referendum_option(&self) -> bool {
        matches!(self, ElectionCountSelectionType::ReferendumOption(_))
    }

    /// Get the candidate selection if the selection type is a candidate selection.
    pub fn as_candidate(&self) -> Option<&CandidateSelection> {
        if let ElectionCountSelectionType::Candidate(candidate_selection) = self {
            Some(candidate_selection)
        } else {
            None
        }
    }

    /// Get the affiliation selection if the selection type is an affiliation selection.
    pub fn as_affiliation(&self) -> Option<&AffiliationSelection> {
        if let ElectionCountSelectionType::Affiliation(affiliation_selection) = self {
            Some(affiliation_selection)
        } else {
            None
        }
    }

    /// Get the referendum option selection if the selection type is a referendum option selection.
    pub fn as_referendum_option(&self) -> Option<&ReferendumOptionSelection> {
        if let ElectionCountSelectionType::ReferendumOption(referendum_option_selection) = self {
            Some(referendum_option_selection)
        } else {
            None
        }
    }
}

/// Selection of a candidate.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "Candidate", ns(NS_EML))]
pub struct CandidateSelection {
    /// Identifier of the candidate.
    #[xml(rename = "CandidateIdentifier")]
    pub identifier: CandidateIdentifier,

    /// Name of the candidate.
    #[xml(rename = "CandidateFullName")]
    pub name: Option<PersonNameStructure>,

    /// Gender of the candidate.
    #[xml(rename = "Gender")]
    pub gender: Option<StringValue<Gender>>,

    /// Qualified address of the candidate, if present.
    #[xml(rename = "QualifyingAddress")]
    pub qualifying_address: Option<MinimalQualifyingAddress>,
}

impl CandidateSelection {
    /// Create a new builder for building an instance.
    pub fn builder() -> CandidateSelectionBuilder {
        CandidateSelectionBuilder::new()
    }
}

impl From<CandidateIdentifier> for CandidateSelection {
    fn from(identifier: CandidateIdentifier) -> Self {
        CandidateSelection {
            identifier,
            name: None,
            gender: None,
            qualifying_address: None,
        }
    }
}

impl From<CandidateId> for CandidateSelection {
    fn from(id: CandidateId) -> Self {
        CandidateSelection::from(CandidateIdentifier::from(id))
    }
}

/// A builder for [`CandidateSelection`].
#[derive(Debug, Clone)]
pub struct CandidateSelectionBuilder {
    identifier: Option<CandidateIdentifier>,
    name: Option<PersonNameStructure>,
    gender: Option<StringValue<Gender>>,
    qualifying_address: Option<MinimalQualifyingAddress>,
    locality_name: Option<String>,
    country_name_code: Option<String>,
}

impl CandidateSelectionBuilder {
    /// Create a new CandidateSelectionBuilder for building [`CandidateSelection`] instances.
    pub fn new() -> Self {
        CandidateSelectionBuilder {
            identifier: None,
            name: None,
            gender: None,
            qualifying_address: None,
            locality_name: None,
            country_name_code: None,
        }
    }

    /// Set the identifier of the candidate selection.
    pub fn identifier(mut self, identifier: impl Into<CandidateIdentifier>) -> Self {
        self.identifier = Some(identifier.into());
        self
    }

    /// Set the name of the candidate selection.
    pub fn name(mut self, name: impl Into<PersonNameStructure>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the gender of the candidate selection.
    pub fn gender(mut self, gender: impl Into<Gender>) -> Self {
        self.gender = Some(StringValue::from_value(gender.into()));
        self
    }

    /// Set the minimal qualifying address of the candidate selection.
    ///
    /// You may also set the locality name and country name code separately
    /// using the [`Self::locality_name`] and [`Self::country_name_code`] methods.
    pub fn qualifying_address(
        mut self,
        qualifying_address: impl Into<MinimalQualifyingAddress>,
    ) -> Self {
        self.qualifying_address = Some(qualifying_address.into());
        self
    }

    /// Set the locality name for the candidate selection.
    ///
    /// Has no effect if the qualifying address is already set using the
    /// [`Self::qualifying_address`] method.
    pub fn locality_name(mut self, locality_name: impl Into<String>) -> Self {
        self.locality_name = Some(locality_name.into());
        self
    }

    /// Set the country name code for the candidate selection.
    ///
    /// Has no effect if the qualifying address is already set using the
    /// [`Self::qualifying_address`] method.
    pub fn country_name_code(mut self, country_name_code: impl Into<String>) -> Self {
        self.country_name_code = Some(country_name_code.into());
        self
    }

    /// Build a CandidateSelection, returning an error if any required properties are missing.
    pub fn build(self) -> Result<CandidateSelection, EMLError> {
        Ok(CandidateSelection {
            identifier: self
                .identifier
                .ok_or_else(|| EMLErrorKind::MissingBuildProperty("identifier").without_span())?,
            name: self.name,
            gender: self.gender,
            qualifying_address: self.qualifying_address.map_or_else(
                || {
                    if let Some(locality_name) = self.locality_name {
                        if let Some(country_name_code) = self.country_name_code {
                            Ok(Some(MinimalQualifyingAddress::new_country(
                                country_name_code,
                                locality_name,
                            )))
                        } else {
                            Ok(Some(MinimalQualifyingAddress::new_locality(locality_name)))
                        }
                    } else {
                        if self.country_name_code.is_some() {
                            return Err(
                                EMLErrorKind::MissingBuildProperty("locality_name").without_span()
                            );
                        }
                        Ok(None)
                    }
                },
                |address| Ok(Some(address)),
            )?,
        })
    }
}

impl Default for CandidateSelectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Selection of an affiliation.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "AffiliationIdentifier", ns(NS_EML))]
pub struct AffiliationSelection {
    /// Id of the affiliation.
    #[xml(attribute, rename = "Id")]
    pub id: StringValue<AffiliationId>,

    /// Name of the affiliation.
    #[xml(rename = "RegisteredName")]
    pub name: String,
}

impl AffiliationSelection {
    /// Create a new AffiliationSelection with the given id and name.
    pub fn new(id: impl Into<AffiliationId>, name: impl Into<String>) -> Self {
        AffiliationSelection {
            id: StringValue::from_value(id.into()),
            name: name.into(),
        }
    }
}

/// Selection of a referendum option.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(
    rename = "ReferendumOptionIdentifier",
    rename_all = "PascalCase",
    ns(NS_EML)
)]
pub struct ReferendumOptionSelection {
    /// Id of the referendum option, if present.
    #[xml(attribute)]
    pub id: Option<String>,

    /// Display order of the referendum option, if present.
    #[xml(attribute)]
    pub display_order: Option<StringValue<NonZeroU64>>,

    /// Short code of the referendum option, if present.
    #[xml(attribute)]
    pub short_code: Option<String>,

    /// Expected confirmation reference of the referendum option, if present.
    #[xml(attribute)]
    pub expected_confirmation_reference: Option<String>,

    /// Value of the referendum option.
    #[xml(direct)]
    pub value: String,
}

impl ReferendumOptionSelection {
    /// Create a new ReferendumOptionSelection with the given value.
    pub fn new(value: impl Into<String>) -> Self {
        ReferendumOptionSelection {
            value: value.into(),
            id: None,
            display_order: None,
            short_code: None,
            expected_confirmation_reference: None,
        }
    }

    /// Set the Id attribute of the referendum option.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the DisplayOrder attribute of the referendum option.
    pub fn with_display_order(mut self, display_order: impl Into<NonZeroU64>) -> Self {
        self.display_order = Some(StringValue::from_value(display_order.into()));
        self
    }

    /// Set the ShortCode attribute of the referendum option.
    pub fn with_short_code(mut self, short_code: impl Into<String>) -> Self {
        self.short_code = Some(short_code.into());
        self
    }

    /// Set the ExpectedConfirmationReference attribute of the referendum option.
    pub fn with_expected_confirmation_reference(
        mut self,
        expected_confirmation_reference: impl Into<String>,
    ) -> Self {
        self.expected_confirmation_reference = Some(expected_confirmation_reference.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, TimeZone as _};

    use crate::{
        common::{AuthorityIdentifier, PersonName},
        documents::EML,
        io::{EMLRead, EMLWrite},
        utils::{AuthorityId, CandidateId, ReportingUnitIdentifierId},
    };

    use super::*;

    #[test]
    fn test_election_count_construction() {
        let ec = ElectionCount::builder()
            .count_type(CountType::Municipal)
            .transaction_id(TransactionId::new(1))
            .creation_date_time(
                chrono::Utc
                    .with_ymd_and_hms(2014, 11, 28, 12, 0, 9)
                    .unwrap(),
            )
            .managing_authority(ManagingAuthority::new(
                AuthorityIdentifier::new(AuthorityId::new("1234").unwrap()).with_name("Rotterdam"),
            ))
            .election_identifier(
                ElectionCountElectionIdentifier::builder()
                    .id(ElectionId::new("GR2222_Cyber").unwrap())
                    .category(ElectionCategory::GR)
                    .election_date(NaiveDate::from_ymd_opt(2222, 11, 16).unwrap())
                    .build_for_count()
                    .unwrap(),
            )
            .contests([ElectionCountContest::builder()
                .identifier(ContestIdentifier::geen())
                .total_candidate_votes_count(65u64)
                .total_eligible_voter_count(100u64)
                .total_rejected_votes(RejectedVotesReason::Blank, 100u64)
                .total_rejected_votes(RejectedVotesReason::Invalid, 0u64)
                .total_uncounted_votes(UncountedVotesReason::AdmittedVoters, 10u64)
                .total_votes_selections([
                    ElectionCountSelection::builder()
                        .affiliation(AffiliationSelection::new(
                            AffiliationId::new(NonZeroU64::new(1).unwrap()),
                            "Example",
                        ))
                        .valid_votes(16u64)
                        .build()
                        .unwrap(),
                    ElectionCountSelection::builder()
                        .candidate(
                            CandidateSelection::builder()
                                .identifier(CandidateId::new(NonZeroU64::new(1).unwrap()))
                                .name(PersonName::new("Smid").with_first_name("Example"))
                                .build()
                                .unwrap(),
                        )
                        .valid_votes(16u64)
                        .build()
                        .unwrap(),
                ])
                .reporting_unit_votes([ReportingUnitVotes::builder()
                    .identifier(ReportingUnitIdentifier::new(
                        ReportingUnitIdentifierId::new("SB1234").unwrap(),
                        "Stembureau",
                    ))
                    .rejected_votes(RejectedVotesReason::Blank, 10u64)
                    .rejected_votes(RejectedVotesReason::Invalid, 0u64)
                    .uncounted_votes(UncountedVotesReason::AdmittedVoters, 5u64)
                    .candidate_votes_count(10u64)
                    .eligible_voter_count(100u64)
                    .selections([
                        ElectionCountSelection::builder()
                            .affiliation(AffiliationSelection::new(
                                AffiliationId::new(NonZeroU64::new(1).unwrap()),
                                "Example",
                            ))
                            .valid_votes(16u64)
                            .build()
                            .unwrap(),
                        ElectionCountSelection::builder()
                            .candidate(
                                CandidateSelection::builder()
                                    .identifier(CandidateId::new(NonZeroU64::new(1).unwrap()))
                                    .name(PersonName::new("Smid").with_first_name("Example"))
                                    .build()
                                    .unwrap(),
                            )
                            .valid_votes(16u64)
                            .build()
                            .unwrap(),
                    ])
                    .build()
                    .unwrap()])
                .build()
                .unwrap()])
            .build()
            .unwrap();

        let xml = ec.write_eml_root_str(true).unwrap();

        // check if it still is the same after a second parse and write
        let eml = EML::parse_eml(&xml).unwrap();
        let parsed = eml.as_count_doc().expect("expected election count variant");
        let xml2 = parsed.write_eml_root_str(true).unwrap();
        assert_eq!(xml, xml2);
    }

    #[test]
    fn test_parse_510b() {
        let xml = include_str!("../../test-emls/election_count/deserialize_eml510b_test.eml.xml");

        let eml = EML::parse_eml(xml).ok_with_errors();
        assert!(eml.unwrap().0.as_count_doc().is_some());
    }

    #[test]
    fn test_parse_510d() {
        let xml = include_str!("../../test-emls/election_count/deserialize_eml510d_test.eml.xml");

        let eml = EML::parse_eml(xml).ok_with_errors();
        assert!(eml.unwrap().0.as_count_doc().is_some());
    }

    #[test]
    fn test_parse_with_investigations() {
        let xml =
            include_str!("../../test-emls/election_count/eml510b_with_investigations.eml.xml");

        let eml = EML::parse_eml(xml).ok_with_errors();
        assert!(eml.unwrap().0.as_count_doc().is_some());
    }
}
