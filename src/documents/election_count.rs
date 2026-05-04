//! Document variant for the EML_NL Count (`510a`, `510b`, `510c` or `510d`) document.

use std::{collections::BTreeMap, num::NonZeroU64, str::FromStr};

use crate::{
    EML_SCHEMA_VERSION, EMLError, EMLErrorKind, EMLResultExt as _, EMLValueResultExt, NS_EML,
    NS_KR,
    common::{
        CandidateIdentifier, CanonicalizationMethod, ContestIdentifier, CreationDateTime,
        ElectionDomain, ManagingAuthority, MinimalQualifyingAddress, PersonNameStructure,
        ReportingUnitIdentifier, TransactionId,
    },
    documents::{ElectionIdentifierBuilder, accepted_root},
    io::{
        EMLElement, EMLElementReader, EMLElementWriter, EMLReadElement as _, EMLWriteElement,
        QualifiedName, collect_struct,
    },
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
        Self::parse_eml(s, crate::io::EMLParsingMode::Strict).ok()
    }
}

impl TryFrom<&str> for ElectionCount {
    type Error = EMLError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use crate::io::EMLRead as _;
        Self::parse_eml(value, crate::io::EMLParsingMode::Strict).ok()
    }
}

impl TryFrom<ElectionCount> for String {
    type Error = EMLError;

    fn try_from(value: ElectionCount) -> Result<Self, Self::Error> {
        use crate::io::EMLWrite as _;
        value.write_eml_root_str(true, true)
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

impl EMLElement for ElectionCount {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("EML", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        accepted_root(elem)?;

        let document_id = elem.attribute_value_req(("Id", None))?;
        let count_type = CountType::from_eml_id(document_id.as_ref())
            .map_err(|e| e.into_kind().with_span(elem.span()))?;

        Ok(collect_struct!(elem, ElectionCount {
            count_type: count_type,
            transaction_id: TransactionId::EML_NAME => |elem| TransactionId::read_eml(elem)?,
            managing_authority: ManagingAuthority::EML_NAME => |elem| ManagingAuthority::read_eml(elem)?,
            creation_date_time: CreationDateTime::EML_NAME => |elem| CreationDateTime::read_eml(elem)?,
            canonicalization_method as Option: CanonicalizationMethod::EML_NAME => |elem| CanonicalizationMethod::read_eml(elem)?,
            count: ElectionCountCount::EML_NAME => |elem| ElectionCountCount::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr(("Id", None), self.count_type.to_eml_id())?
            .attr(("SchemaVersion", None), EML_SCHEMA_VERSION)?
            .child_elem(TransactionId::EML_NAME, &self.transaction_id)?
            .child_elem(ManagingAuthority::EML_NAME, &self.managing_authority)?
            .child_elem(CreationDateTime::EML_NAME, &self.creation_date_time)?
            // Note: we don't output the CanonicalizationMethod because we aren't canonicalizing our output
            // .child_elem_option(
            //     CanonicalizationMethod::EML_NAME,
            //     self.canonicalization_method.as_ref(),
            // )?
            .child_elem(ElectionCountCount::EML_NAME, &self.count)?
            .finish()
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

impl EMLElement for ElectionCountCount {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Count", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, ElectionCountCount {
            id as None: ("EventIdentifier", NS_EML) => |elem| elem.skip().map(|_| ())?,
            election: ElectionCountElection::EML_NAME => |elem| ElectionCountElection::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child(("EventIdentifier", NS_EML), |w| w.empty())?
            .child_elem(ElectionCountElection::EML_NAME, &self.election)?
            .finish()
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

impl EMLElement for ElectionCountElection {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Election", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let data = collect_struct!(elem, ElectionCountElection {
            identifier: ElectionCountElectionIdentifier::EML_NAME => |elem| ElectionCountElectionIdentifier::read_eml(elem)?,
            contests: ("Contests", NS_EML) => |elem| {
                struct VecCollector {
                    contests: Vec<ElectionCountContest>,
                }

                let data = collect_struct!(elem, VecCollector {
                    contests as Vec: ElectionCountContest::EML_NAME => |elem| ElectionCountContest::read_eml(elem)?,
                });

                data.contests
            },
        });

        if data.contests.is_empty() {
            let err = EMLErrorKind::MissingElement(ElectionCountContest::EML_NAME.as_owned())
                .with_span(elem.full_span());
            if elem.parsing_mode().is_strict() {
                return Err(err);
            } else {
                elem.push_err(err);
            }
        }

        Ok(data)
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(ElectionCountElectionIdentifier::EML_NAME, &self.identifier)?
            .child(("Contests", NS_EML), |writer| {
                writer
                    .child_elems(ElectionCountContest::EML_NAME, &self.contests)?
                    .finish()
            })?
            .finish()
    }
}

/// Identifier for the election in this count.
#[derive(Debug, Clone)]
pub struct ElectionCountElectionIdentifier {
    /// Id of the election
    pub id: StringValue<ElectionId>,

    /// Name of the election
    pub name: Option<String>,

    /// Category of the election
    pub category: StringValue<ElectionCategory>,

    /// Subcategory of the election
    pub subcategory: Option<StringValue<ElectionSubcategory>>,

    /// The (top level) region where the election takes place.
    pub domain: Option<ElectionDomain>,

    /// Date of the election
    pub election_date: StringValue<XsDate>,
}

impl ElectionCountElectionIdentifier {
    /// Create a builder for the [`ElectionCountElectionIdentifier`].
    pub fn builder() -> ElectionIdentifierBuilder {
        ElectionIdentifierBuilder::new()
    }
}

impl EMLElement for ElectionCountElectionIdentifier {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ElectionIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, ElectionCountElectionIdentifier {
            id: elem.string_value_attr("Id", None)?,
            name as Option: ("ElectionName", NS_EML) => |elem| elem.text_without_children()?,
            category: ("ElectionCategory", NS_EML) => |elem| elem.string_value()?,
            subcategory as Option: ("ElectionSubcategory", NS_KR) => |elem| elem.string_value()?,
            domain as Option: ElectionDomain::EML_NAME => |elem| ElectionDomain::read_eml(elem)?,
            election_date: ("ElectionDate", NS_KR) => |elem| elem.string_value()?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr("Id", self.id.raw().as_ref())?
            .child_option(
                ("ElectionName", NS_EML),
                self.name.as_ref(),
                |elem, value| elem.text(value.as_ref())?.finish(),
            )?
            .child(("ElectionCategory", NS_EML), |elem| {
                elem.text(self.category.raw().as_ref())?.finish()
            })?
            .child_option(
                ("ElectionSubcategory", NS_KR),
                self.subcategory.as_ref(),
                |elem, value| elem.text(value.raw().as_ref())?.finish(),
            )?
            .child_elem_option(ElectionDomain::EML_NAME, self.domain.as_ref())?
            .child(("ElectionDate", NS_KR), |elem| {
                elem.text(self.election_date.raw().as_ref())?.finish()
            })?
            .finish()
    }
}

/// A contest within the election count.
#[derive(Debug, Clone)]
pub struct ElectionCountContest {
    /// Identifier for the contest.
    pub identifier: ContestIdentifier,

    /// Total votes in this contest, if present.
    pub total_votes: Option<TotalVotes>,

    /// Votes per reporting unit in this contest.
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

impl EMLElement for ElectionCountContest {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Contest", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, ElectionCountContest {
            identifier: ContestIdentifier::EML_NAME => |elem| ContestIdentifier::read_eml(elem)?,
            total_votes as Option: TotalVotes::EML_NAME => |elem| TotalVotes::read_eml(elem)?,
            reporting_unit_votes as Vec: ReportingUnitVotes::EML_NAME => |elem| ReportingUnitVotes::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(ContestIdentifier::EML_NAME, &self.identifier)?
            .child_elem_option(TotalVotes::EML_NAME, self.total_votes.as_ref())?
            .child_elems(ReportingUnitVotes::EML_NAME, &self.reporting_unit_votes)?
            .finish()
    }
}

const REJECTED_VOTES_EML_NAME: QualifiedName<'_, '_> =
    QualifiedName::from_static("RejectedVotes", Some(NS_EML));

const UNCOUNTED_VOTES_EML_NAME: QualifiedName<'_, '_> =
    QualifiedName::from_static("UncountedVotes", Some(NS_EML));

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

impl EMLElement for TotalVotes {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("TotalVotes", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let data = collect_struct!(elem, TotalVotes {
            selections as Vec: ElectionCountSelection::EML_NAME => |elem| ElectionCountSelection::read_eml(elem)?,
            eligible_voter_count: ("Cast", NS_EML) => |elem| elem.string_value()?,
            candidate_votes_count: ("TotalCounted", NS_EML) => |elem| elem.string_value()?,
            rejected_votes as BTreeMap: REJECTED_VOTES_EML_NAME => |elem| {
                let reason_code = elem.attribute_value_req("ReasonCode")?;
                let reason = RejectedVotesReason::from_eml_value(&reason_code)
                    .map_err(|e| EMLError::invalid_value(REJECTED_VOTES_EML_NAME.as_owned(), e, Some(elem.full_span())))?;

                (reason, elem.string_value()?)
            },
            uncounted_votes as BTreeMap: UNCOUNTED_VOTES_EML_NAME => |elem| {
                let reason_code = elem.attribute_value_req("ReasonCode")?;
                let reason = UncountedVotesReason::from_eml_value(&reason_code)
                    .map_err(|e| EMLError::invalid_value(UNCOUNTED_VOTES_EML_NAME.as_owned(), e, Some(elem.full_span())))?;

                (reason, elem.string_value()?)
            },
        });

        if !data
            .rejected_votes
            .contains_key(&RejectedVotesReason::Blank)
        {
            let err = EMLErrorKind::MissingRejectedVotesBlank.with_span(elem.full_span());
            if elem.parsing_mode().is_strict() {
                return Err(err);
            } else {
                elem.push_err(err);
            }
        }

        if !data
            .rejected_votes
            .contains_key(&RejectedVotesReason::Invalid)
        {
            let err = EMLErrorKind::MissingRejectedVotesInvalid.with_span(elem.full_span());
            if elem.parsing_mode().is_strict() {
                return Err(err);
            } else {
                elem.push_err(err);
            }
        }

        Ok(data)
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elems(ElectionCountSelection::EML_NAME, &self.selections)?
            .child(("Cast", NS_EML), |elem| {
                elem.text(self.eligible_voter_count.raw().as_ref())?
                    .finish()
            })?
            .child(("TotalCounted", NS_EML), |elem| {
                elem.text(self.candidate_votes_count.raw().as_ref())?
                    .finish()
            })?
            .child_elems_map(
                REJECTED_VOTES_EML_NAME,
                &self.rejected_votes,
                |elem, (reason, count)| {
                    let reason_code = reason.to_eml_value();
                    elem.attr("ReasonCode", reason_code.as_ref())?
                        .text(count.raw().as_ref())?
                        .finish()
                },
            )?
            .child_elems_map(
                UNCOUNTED_VOTES_EML_NAME,
                &self.uncounted_votes,
                |elem, (reason, count)| {
                    let reason_code = reason.to_eml_value();
                    elem.attr("ReasonCode", reason_code.as_ref())?
                        .text(count.raw().as_ref())?
                        .finish()
                },
            )?
            .finish()
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
                            .wrap_field_value_error(VALID_VOTES_EML_NAME)?,
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
                        .wrap_field_value_error(VALID_VOTES_EML_NAME)?,
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
                .wrap_field_value_error(VALID_VOTES_EML_NAME)?,
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

const REPORTING_UNIT_INVESTIGATIONS_EML_NAME: QualifiedName<'_, '_> =
    QualifiedName::from_static("ReportingUnitInvestigations", Some(NS_KR));

const INVESTIGATION_EML_NAME: QualifiedName<'_, '_> =
    QualifiedName::from_static("Investigation", Some(NS_KR));

impl EMLElement for ReportingUnitVotes {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ReportingUnitVotes", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        struct ReportingUnitVotesInternal {
            identifier: ReportingUnitIdentifier,
            selections: Vec<ElectionCountSelection>,
            eligible_voter_count: StringValue<u64>,
            candidate_votes_count: StringValue<u64>,
            rejected_votes: BTreeMap<RejectedVotesReason, StringValue<u64>>,
            uncounted_votes: BTreeMap<UncountedVotesReason, StringValue<u64>>,
            investigations: Option<BTreeMap<InvestigationReason, StringValue<bool>>>,
        }

        let data = collect_struct!(elem, ReportingUnitVotesInternal {
            identifier: ReportingUnitIdentifier::EML_NAME => |elem| ReportingUnitIdentifier::read_eml(elem)?,
            selections as Vec: ElectionCountSelection::EML_NAME => |elem| ElectionCountSelection::read_eml(elem)?,
            eligible_voter_count: ("Cast", NS_EML) => |elem| elem.string_value()?,
            candidate_votes_count: ("TotalCounted", NS_EML) => |elem| elem.string_value()?,
            rejected_votes as BTreeMap: REJECTED_VOTES_EML_NAME => |elem| {
                let reason_code = elem.attribute_value_req("ReasonCode")?;
                let reason = RejectedVotesReason::from_eml_value(&reason_code)
                    .map_err(|e| EMLError::invalid_value(REJECTED_VOTES_EML_NAME.as_owned(), e, Some(elem.full_span())))?;

                (reason, elem.string_value()?)
            },
            uncounted_votes as BTreeMap: UNCOUNTED_VOTES_EML_NAME => |elem| {
                let reason_code = elem.attribute_value_req("ReasonCode")?;
                let reason = UncountedVotesReason::from_eml_value(&reason_code)
                    .map_err(|e| EMLError::invalid_value(UNCOUNTED_VOTES_EML_NAME.as_owned(), e, Some(elem.full_span())))?;

                (reason, elem.string_value()?)
            },
            investigations as Option: REPORTING_UNIT_INVESTIGATIONS_EML_NAME => |elem| {
                struct Collector {
                    investigations: BTreeMap<InvestigationReason, StringValue<bool>>,
                }

                let data = collect_struct!(elem, Collector {
                    investigations as BTreeMap: INVESTIGATION_EML_NAME => |elem| {
                        let reason_code = elem.attribute_value_req("ReasonCode")?;
                        let reason = InvestigationReason::from_eml_value(&reason_code)
                            .map_err(|e| EMLError::invalid_value(INVESTIGATION_EML_NAME.as_owned(), e, Some(elem.full_span())))?;

                        (reason, elem.string_value()?)
                    },
                });

                data.investigations
            },
        });

        if !data
            .rejected_votes
            .contains_key(&RejectedVotesReason::Blank)
        {
            let err = EMLErrorKind::MissingRejectedVotesBlank.with_span(elem.full_span());
            if elem.parsing_mode().is_strict() {
                return Err(err);
            } else {
                elem.push_err(err);
            }
        }

        if !data
            .rejected_votes
            .contains_key(&RejectedVotesReason::Invalid)
        {
            let err = EMLErrorKind::MissingRejectedVotesInvalid.with_span(elem.full_span());
            if elem.parsing_mode().is_strict() {
                return Err(err);
            } else {
                elem.push_err(err);
            }
        }

        Ok(ReportingUnitVotes {
            identifier: data.identifier,
            selections: data.selections,
            eligible_voter_count: data.eligible_voter_count,
            candidate_votes_count: data.candidate_votes_count,
            rejected_votes: data.rejected_votes,
            uncounted_votes: data.uncounted_votes,
            investigations: data.investigations.unwrap_or_default(),
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(ReportingUnitIdentifier::EML_NAME, &self.identifier)?
            .child_elems(ElectionCountSelection::EML_NAME, &self.selections)?
            .child(("Cast", NS_EML), |elem| {
                elem.text(self.eligible_voter_count.raw().as_ref())?
                    .finish()
            })?
            .child(("TotalCounted", NS_EML), |elem| {
                elem.text(self.candidate_votes_count.raw().as_ref())?
                    .finish()
            })?
            .child_elems_map(
                REJECTED_VOTES_EML_NAME,
                &self.rejected_votes,
                |elem, (reason, count)| {
                    let reason_code = reason.to_eml_value();
                    elem.attr("ReasonCode", reason_code.as_ref())?
                        .text(count.raw().as_ref())?
                        .finish()
                },
            )?
            .child_elems_map(
                UNCOUNTED_VOTES_EML_NAME,
                &self.uncounted_votes,
                |elem, (reason, count)| {
                    let reason_code = reason.to_eml_value();
                    elem.attr("ReasonCode", reason_code.as_ref())?
                        .text(count.raw().as_ref())?
                        .finish()
                },
            )?
            .child_option(
                REPORTING_UNIT_INVESTIGATIONS_EML_NAME,
                if self.investigations.is_empty() {
                    None
                } else {
                    Some(&self.investigations)
                },
                |elem, value| {
                    let mut elem = elem.content()?;

                    for (reason, investigated) in value {
                        elem = elem.child(INVESTIGATION_EML_NAME, |elem| {
                            let reason_code = reason.to_eml_value();
                            elem.attr("ReasonCode", reason_code.as_ref())?
                                .text(investigated.raw().as_ref())?
                                .finish()
                        })?;
                    }

                    elem.finish()
                },
            )?
            .finish()
    }
}

/// Reason code for a specific investigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InvestigationReason {
    /// onderzocht vanwege onverklaard verschil
    UnexplainedDifference,
    /// onderzocht vanwege andere fout
    OtherError,
    /// uitslag gecorrigeerd
    ResultCorrected,
    /// toegelaten kiezers opnieuw vastgesteld
    AdmittedVotersReestablished,
    /// onderzocht vanwege andere reden
    OtherReason,
    /// stembiljetten deels herteld
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum UncountedVotesReason {
    /// geldige stempassen
    ValidPollCards,
    /// geldige volmachtbewijzen
    ValidProxyCertificates,
    /// geldige kiezerspassen
    ValidVoterCards,
    /// toegelaten kiezers
    AdmittedVoters,
    /// meer getelde stembiljetten
    MoreBallotsCounted,
    /// minder getelde stembiljetten
    FewerBallotsCounted,
    /// meegenomen stembiljetten
    BallotsTaken,
    /// te weinig uitgereikte stembiljetten
    TooFewBallotsIssued,
    /// te veel uitgereikte stembiljetten
    TooManyBallotsIssued,
    /// geen briefstembiljetten
    NoPostalBallots,
    /// te veel briefstembiljetten
    TooManyPostalBallots,
    /// kwijtgeraakte stembiljetten
    LostBallots,
    /// geen verklaring
    NoExplanation,
    /// andere verklaring
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RejectedVotesReason {
    /// Blank votes ("blanco")
    Blank,
    /// Invalid votes ("ongeldig")
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
#[derive(Debug, Clone)]
pub struct ElectionCountSelection {
    /// Type of selection.
    pub selection_type: ElectionCountSelectionType,

    /// Number of valid votes for this selection.
    pub valid_votes: StringValue<u64>,

    /// Value of the `Value` attribute, if present.
    pub value: Option<String>,

    /// Value of the `Category` attribute, if present.
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

const VALID_VOTES_EML_NAME: QualifiedName<'_, '_> =
    QualifiedName::from_static("ValidVotes", Some(NS_EML));

impl EMLElement for ElectionCountSelection {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Selection", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let value = elem.attribute_value("Value")?.map(|s| s.to_string());
        let category = elem.attribute_value("Category")?.map(|s| s.to_string());
        let mut selection_type = None;
        let mut valid_votes = None;

        while let Some(mut child) = elem.next_child()? {
            let name = child.name()?;

            match name {
                n if n == CandidateSelection::EML_NAME => {
                    selection_type = Some(ElectionCountSelectionType::Candidate(Box::new(
                        CandidateSelection::read_eml(&mut child)?,
                    )));
                }
                n if n == AffiliationSelection::EML_NAME => {
                    selection_type = Some(ElectionCountSelectionType::Affiliation(Box::new(
                        AffiliationSelection::read_eml(&mut child)?,
                    )));
                }
                n if n == ReferendumOptionSelection::EML_NAME => {
                    selection_type = Some(ElectionCountSelectionType::ReferendumOption(Box::new(
                        ReferendumOptionSelection::read_eml(&mut child)?,
                    )));
                }
                n if n == VALID_VOTES_EML_NAME => {
                    valid_votes = Some(child.string_value()?);
                }
                n => {
                    let err =
                        EMLErrorKind::UnexpectedElement(n.as_owned(), Self::EML_NAME.as_owned())
                            .with_span(child.inner_span());
                    if child.parsing_mode().is_strict() {
                        return Err(err);
                    } else {
                        child.push_err(err);
                        child.skip()?;
                    }
                }
            }
        }
        Ok(ElectionCountSelection {
            selection_type: selection_type
                .ok_or_else(|| EMLErrorKind::MissingSelectionType.with_span(elem.inner_span()))?,
            valid_votes: valid_votes.ok_or_else(|| {
                EMLErrorKind::MissingElement(VALID_VOTES_EML_NAME.as_owned())
                    .with_span(elem.inner_span())
            })?,
            value,
            category,
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        let writer = writer
            .attr_opt("Value", self.value.as_deref())?
            .attr_opt("Category", self.category.as_deref())?;
        let writer = match &self.selection_type {
            ElectionCountSelectionType::Candidate(candidate_selection) => {
                writer.child_elem(CandidateSelection::EML_NAME, candidate_selection.as_ref())?
            }
            ElectionCountSelectionType::Affiliation(affiliation_selection) => writer.child_elem(
                AffiliationSelection::EML_NAME,
                affiliation_selection.as_ref(),
            )?,
            ElectionCountSelectionType::ReferendumOption(referendum_option_selection) => writer
                .child_elem(
                    ReferendumOptionSelection::EML_NAME,
                    referendum_option_selection.as_ref(),
                )?,
        };
        writer
            .child(("ValidVotes", NS_EML), |elem| {
                elem.text(self.valid_votes.raw().as_ref())?.finish()
            })?
            .finish()
    }
}

/// The type of selection.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct CandidateSelection {
    /// Identifier of the candidate.
    pub identifier: CandidateIdentifier,

    /// Name of the candidate.
    pub name: Option<PersonNameStructure>,

    /// Gender of the candidate.
    pub gender: Option<StringValue<Gender>>,

    /// Qualified address of the candidate, if present.
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

impl EMLElement for CandidateSelection {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Candidate", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, CandidateSelection {
            identifier: CandidateIdentifier::EML_NAME => |elem| CandidateIdentifier::read_eml(elem)?,
            name as Option: ("CandidateFullName", NS_EML) => |elem| PersonNameStructure::read_eml_element(elem)?,
            gender as Option: ("Gender", NS_EML) => |elem| elem.string_value()?,
            qualifying_address as Option: MinimalQualifyingAddress::EML_NAME => |elem| MinimalQualifyingAddress::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(CandidateIdentifier::EML_NAME, &self.identifier)?
            .child_option(
                ("CandidateFullName", NS_EML),
                self.name.as_ref(),
                |elem, value| value.write_eml_element(elem),
            )?
            .child_option(("Gender", NS_EML), self.gender.as_ref(), |elem, value| {
                elem.text(value.raw().as_ref())?.finish()
            })?
            .child_elem_option(
                MinimalQualifyingAddress::EML_NAME,
                self.qualifying_address.as_ref(),
            )?
            .finish()
    }
}

/// Selection of an affiliation.
#[derive(Debug, Clone)]
pub struct AffiliationSelection {
    /// Id of the affiliation.
    pub id: StringValue<AffiliationId>,

    /// Name of the affiliation.
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

impl EMLElement for AffiliationSelection {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("AffiliationIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, AffiliationSelection {
            id: elem.string_value_attr("Id", None)?,
            name: ("RegisteredName", NS_EML) => |elem| elem.text_without_children()?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr("Id", self.id.raw().as_ref())?
            .child(("RegisteredName", NS_EML), |elem| {
                elem.text(self.name.as_ref())?.finish()
            })?
            .finish()
    }
}

/// Selection of a referendum option.
#[derive(Debug, Clone)]
pub struct ReferendumOptionSelection {
    /// Value of the referendum option.
    pub value: String,

    /// Id of the referendum option, if present.
    pub id: Option<String>,

    /// Display order of the referendum option, if present.
    pub display_order: Option<StringValue<NonZeroU64>>,

    /// Short code of the referendum option, if present.
    pub short_code: Option<String>,

    /// Expected confirmation reference of the referendum option, if present.
    pub expected_confirmation_reference: Option<String>,
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

impl EMLElement for ReferendumOptionSelection {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ReferendumOptionIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(ReferendumOptionSelection {
            value: elem.text_without_children()?,
            id: elem.attribute_value("Id")?.map(|s| s.into_owned()),
            display_order: elem.string_value_attr_opt("DisplayOrder")?,
            short_code: elem.attribute_value("ShortCode")?.map(|s| s.into_owned()),
            expected_confirmation_reference: elem
                .attribute_value("ExpectedConfirmationReference")?
                .map(|s| s.into_owned()),
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr_opt("Id", self.id.as_deref())?
            .attr_opt("DisplayOrder", self.display_order.as_ref().map(|s| s.raw()))?
            .attr_opt("ShortCode", self.short_code.as_deref())?
            .attr_opt(
                "ExpectedConfirmationReference",
                self.expected_confirmation_reference.as_deref(),
            )?
            .text(self.value.as_ref())?
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, TimeZone as _};

    use crate::{
        common::{AuthorityIdentifier, PersonName},
        io::{EMLParsingMode, EMLRead, EMLWrite},
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

        let xml = ec.write_eml_root_str(true, true).unwrap();
        assert_eq!(
            xml,
            include_str!("../../test-emls/election_count/eml510b_construction_output.eml.xml")
        );

        // check if it still is the same after a second parse and write
        let parsed = ElectionCount::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        let xml2 = parsed.write_eml_root_str(true, true).unwrap();
        assert_eq!(xml, xml2);
    }

    #[test]
    fn test_parse_510b() {
        let xml = include_str!("../../test-emls/election_count/deserialize_eml510b_test.eml.xml");

        assert!(
            ElectionCount::parse_eml(xml, EMLParsingMode::Strict)
                .ok_with_errors()
                .is_ok()
        );
    }

    #[test]
    fn test_parse_510d() {
        let xml = include_str!("../../test-emls/election_count/deserialize_eml510d_test.eml.xml");

        assert!(
            ElectionCount::parse_eml(xml, EMLParsingMode::Strict)
                .ok_with_errors()
                .is_ok()
        );
    }

    #[test]
    fn test_parse_with_investigations() {
        let xml =
            include_str!("../../test-emls/election_count/eml510b_with_investigations.eml.xml");

        assert!(
            ElectionCount::parse_eml(xml, EMLParsingMode::Strict)
                .ok_with_errors()
                .is_ok()
        );
    }
}
