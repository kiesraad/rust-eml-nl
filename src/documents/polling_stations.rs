//! Document variant for the EML_NL Polling Stations (`110b`) document.

use std::{num::NonZeroU64, str::FromStr, sync::LazyLock};

use regex::Regex;
use thiserror::Error;

use crate::{
    EML_SCHEMA_VERSION, EMLError, EMLValueResultExt, NS_EML, NS_KR,
    common::{
        CanonicalizationMethod, ContestIdentifier, ContestIdentifierGeen, CreationDateTime,
        ElectionDomain, IssueDate, LocalityName, ManagingAuthority, PostalCode,
        ReportingUnitIdentifier, TransactionId,
    },
    documents::{ElectionIdentifierBuilder, accepted_root},
    error::{EMLErrorKind, EMLResultExt},
    io::{
        EMLElement, EMLElementReader, EMLElementWriter, OwnedQualifiedName, QualifiedName,
        collect_struct,
    },
    utils::{
        ElectionCategory, ElectionId, ElectionSubcategory, StringValue, StringValueData,
        VotingChannelType, VotingMethod, XsDate, XsDateOrDateTime, XsDateTime,
    },
};

pub(crate) const EML_POLLING_STATIONS_ID: &str = "110b";

/// Representing a `110b` document, containing polling stations.
#[derive(Debug, Clone)]
pub struct PollingStations {
    /// Transaction id of the document.
    pub transaction_id: TransactionId,

    /// Managing authority of the document.
    pub managing_authority: ManagingAuthority,

    /// Issue date of the document.
    pub issue_date: Option<IssueDate>,

    /// Creation date and time of the document.
    pub creation_date_time: CreationDateTime,

    /// Canonicalization method used in this document, if present.
    pub canonicalization_method: Option<CanonicalizationMethod>,

    /// Election event containing the polling stations.
    pub election_event: PollingStationsElectionEvent,
}

impl PollingStations {
    /// Create a new builder for constructing a [`PollingStations`] document.
    pub fn builder() -> PollingStationsBuilder {
        PollingStationsBuilder::new()
    }
}

impl FromStr for PollingStations {
    type Err = EMLError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::io::EMLRead as _;
        Self::parse_eml(s, crate::io::EMLParsingMode::Strict).ok()
    }
}

impl TryFrom<&str> for PollingStations {
    type Error = EMLError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use crate::io::EMLRead as _;
        Self::parse_eml(value, crate::io::EMLParsingMode::Strict).ok()
    }
}

impl TryFrom<PollingStations> for String {
    type Error = EMLError;

    fn try_from(value: PollingStations) -> Result<Self, Self::Error> {
        use crate::io::EMLWrite as _;
        value.write_eml_root_str(true, true)
    }
}

/// Builder for the [`PollingStations`] document.
#[derive(Debug, Clone)]
pub struct PollingStationsBuilder {
    transaction_id: Option<TransactionId>,
    managing_authority: Option<ManagingAuthority>,
    issue_date: Option<IssueDate>,
    creation_date_time: Option<CreationDateTime>,
    canonicalization_method: Option<CanonicalizationMethod>,
    election_event: Option<PollingStationsElectionEvent>,
    election_identifier: Option<PollingStationsElectionIdentifier>,
    contests: Vec<PollingStationsContest>,
}

impl PollingStationsBuilder {
    /// Create a new builder for the [`PollingStations`] document.
    pub fn new() -> Self {
        Self {
            transaction_id: None,
            managing_authority: None,
            issue_date: None,
            creation_date_time: None,
            canonicalization_method: None,
            election_event: None,
            election_identifier: None,
            contests: vec![],
        }
    }

    /// Set the transaction id of the document.
    pub fn transaction_id(mut self, transaction_id: impl Into<TransactionId>) -> Self {
        self.transaction_id = Some(transaction_id.into());
        self
    }

    /// Set the managing authority of the document.
    pub fn managing_authority(mut self, managing_authority: impl Into<ManagingAuthority>) -> Self {
        self.managing_authority = Some(managing_authority.into());
        self
    }

    /// Set the issue date of the document.
    pub fn issue_date(mut self, issue_date: impl Into<XsDateOrDateTime>) -> Self {
        self.issue_date = Some(IssueDate::new(issue_date.into()));
        self
    }

    /// Set the creation date and time of the document.
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

    /// Set the election event containing the polling stations.
    ///
    /// You may either set the entire election event at once using this method,
    /// or use any of [`Self::election_identifier`], [`Self::contests`] and/or
    /// [`Self::push_contest`] to set the individual components of the
    /// election event and allow this builder to construct them for you.
    pub fn election_event(
        mut self,
        election_event: impl Into<PollingStationsElectionEvent>,
    ) -> Self {
        self.election_event = Some(election_event.into());
        self
    }

    /// Set the election identifier for the contained Election element.
    ///
    /// This only has effect if the election event was not set directly using
    /// [`Self::election_event`].
    pub fn election_identifier(
        mut self,
        election_identifier: impl Into<PollingStationsElectionIdentifier>,
    ) -> Self {
        self.election_identifier = Some(election_identifier.into());
        self
    }

    /// Set the list of contests within the election. This will replace any
    /// existing contests set using this method or the [`Self::push_contest`]
    /// method.
    ///
    /// This only has effect if the election event was not set directly using
    /// [`Self::election_event`].
    pub fn contests(mut self, contests: impl Into<Vec<PollingStationsContest>>) -> Self {
        self.contests = contests.into();
        self
    }

    /// Add a contest to the election.
    ///
    /// This only has effect if the election event was not set directly using
    /// [`Self::election_event`].
    pub fn push_contest(mut self, contest: impl Into<PollingStationsContest>) -> Self {
        self.contests.push(contest.into());
        self
    }

    /// Build the [`PollingStations`] document, returning any errors if required fields are missing.
    pub fn build(self) -> Result<PollingStations, EMLError> {
        Ok(PollingStations {
            transaction_id: self
                .transaction_id
                .ok_or(EMLErrorKind::MissingBuildProperty("transaction_id").without_span())?,
            managing_authority: self
                .managing_authority
                .ok_or(EMLErrorKind::MissingBuildProperty("managing_authority").without_span())?,
            issue_date: self.issue_date,
            creation_date_time: self
                .creation_date_time
                .ok_or(EMLErrorKind::MissingBuildProperty("creation_date_time").without_span())?,
            canonicalization_method: self.canonicalization_method,
            election_event: self.election_event.map_or_else(
                || {
                    if self.contests.is_empty() {
                        return Err(EMLErrorKind::MissingBuildProperty("contests").without_span());
                    }

                    let election = PollingStationsElection::new(self.election_identifier.ok_or(
                        EMLErrorKind::MissingBuildProperty("election_identifier").without_span(),
                    )?)
                    .with_contests(self.contests);

                    let event = PollingStationsElectionEvent::new(election);

                    Ok(event)
                },
                Ok,
            )?,
        })
    }
}

impl Default for PollingStationsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EMLElement for PollingStations {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("EML", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        accepted_root(elem)?;

        let document_id = elem.attribute_value_req(("Id", None))?;
        if document_id != EML_POLLING_STATIONS_ID {
            return Err(EMLErrorKind::InvalidDocumentType(
                EML_POLLING_STATIONS_ID,
                document_id.to_string(),
            ))
            .with_span(elem.span());
        }

        Ok(collect_struct!(elem, PollingStations {
            transaction_id: TransactionId::EML_NAME => |elem| TransactionId::read_eml(elem)?,
            managing_authority: ManagingAuthority::EML_NAME => |elem| ManagingAuthority::read_eml(elem)?,
            issue_date as Option: IssueDate::EML_NAME => |elem| IssueDate::read_eml(elem)?,
            creation_date_time: CreationDateTime::EML_NAME => |elem| CreationDateTime::read_eml(elem)?,
            canonicalization_method as Option: CanonicalizationMethod::EML_NAME => |elem| CanonicalizationMethod::read_eml(elem)?,
            election_event: PollingStationsElectionEvent::EML_NAME => |elem| PollingStationsElectionEvent::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr(("Id", None), EML_POLLING_STATIONS_ID)?
            .attr(("SchemaVersion", None), EML_SCHEMA_VERSION)?
            .child_elem(TransactionId::EML_NAME, &self.transaction_id)?
            .child_elem(ManagingAuthority::EML_NAME, &self.managing_authority)?
            .child_elem_option(IssueDate::EML_NAME, self.issue_date.as_ref())?
            .child_elem(CreationDateTime::EML_NAME, &self.creation_date_time)?
            // Note: we don't output the CanonicalizationMethod because we aren't canonicalizing our output
            // .child_elem_option(
            //     CanonicalizationMethod::EML_NAME,
            //     self.canonicalization_method.as_ref(),
            // )?
            .child_elem(PollingStationsElectionEvent::EML_NAME, &self.election_event)?
            .finish()?;

        Ok(())
    }
}

/// Election event containing polling stations.
#[derive(Debug, Clone)]
pub struct PollingStationsElectionEvent {
    /// Election details.
    pub election: PollingStationsElection,
}

impl PollingStationsElectionEvent {
    /// Create a new election event containing the given election.
    pub fn new(election: impl Into<PollingStationsElection>) -> Self {
        PollingStationsElectionEvent {
            election: election.into(),
        }
    }
}

impl From<PollingStationsElection> for PollingStationsElectionEvent {
    fn from(value: PollingStationsElection) -> Self {
        PollingStationsElectionEvent::new(value)
    }
}

impl EMLElement for PollingStationsElectionEvent {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ElectionEvent", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError>
    where
        Self: Sized,
    {
        Ok(collect_struct!(elem, PollingStationsElectionEvent {
            id as None: ("EventIdentifier", NS_EML) => |elem| elem.skip().map(|_| ())?,
            election: PollingStationsElection::EML_NAME => |elem| PollingStationsElection::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child(("EventIdentifier", NS_EML), |w| w.empty())?
            .child_elem(PollingStationsElection::EML_NAME, &self.election)?
            .finish()
    }
}

/// Election definition containing polling stations.
#[derive(Debug, Clone)]
pub struct PollingStationsElection {
    /// Identifier of the election.
    pub identifier: PollingStationsElectionIdentifier,

    /// Contests containing the polling stations.
    pub contests: Vec<PollingStationsContest>,
}

impl PollingStationsElection {
    /// Create a new election within the polling stations document.
    pub fn new(identifier: impl Into<PollingStationsElectionIdentifier>) -> Self {
        PollingStationsElection {
            identifier: identifier.into(),
            contests: vec![],
        }
    }

    /// Set the contests within the election. This will replace any existing
    /// contests set using this method or the [`Self::push_contest`] method.
    pub fn with_contests(mut self, contests: impl Into<Vec<PollingStationsContest>>) -> Self {
        self.contests = contests.into();
        self
    }

    /// Add a contest to the election.
    pub fn push_contest(mut self, contest: impl Into<PollingStationsContest>) -> Self {
        self.contests.push(contest.into());
        self
    }
}

impl EMLElement for PollingStationsElection {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Election", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError>
    where
        Self: Sized,
    {
        Ok(collect_struct!(elem, PollingStationsElection {
            identifier: PollingStationsElectionIdentifier::EML_NAME => |elem| PollingStationsElectionIdentifier::read_eml(elem)?,
            contests as Vec: PollingStationsContest::EML_NAME => |elem| PollingStationsContest::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(
                PollingStationsElectionIdentifier::EML_NAME,
                &self.identifier,
            )?
            .child_elems(PollingStationsContest::EML_NAME, &self.contests)?
            .finish()
    }
}

/// Identifier of an election in the polling stations document.
#[derive(Debug, Clone)]
pub struct PollingStationsElectionIdentifier {
    /// Election id.
    pub id: StringValue<ElectionId>,

    /// Election name, if present.
    pub name: Option<Box<str>>,

    /// Election category.
    pub category: StringValue<ElectionCategory>,

    /// Election subcategory, if present.
    pub subcategory: Option<StringValue<ElectionSubcategory>>,

    /// The (top level) region where the election takes place.
    pub domain: Option<ElectionDomain>,

    /// Date of the election
    pub election_date: StringValue<XsDate>,
}

impl PollingStationsElectionIdentifier {
    /// Create a new builder for constructing a [`PollingStationsElectionIdentifier`].
    pub fn builder() -> ElectionIdentifierBuilder {
        ElectionIdentifierBuilder::new()
    }
}

impl EMLElement for PollingStationsElectionIdentifier {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ElectionIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        struct PollingStationsElectionIdentifierInternal {
            id: StringValue<ElectionId>,
            name: Option<Box<str>>,
            category: StringValue<ElectionCategory>,
            subcategory: Option<StringValue<ElectionSubcategory>>,
            domain: Option<ElectionDomain>,
            election_date: Option<StringValue<XsDate>>,
            election_date_eml: Option<StringValue<XsDate>>,
        }

        let data = collect_struct!(
            elem,
            PollingStationsElectionIdentifierInternal {
                id: elem.string_value_attr("Id", None)?,
                name as Option: ("ElectionName", NS_EML) => |elem| elem.text_without_children()?,
                category: ("ElectionCategory", NS_EML) => |elem| elem.string_value()?,
                subcategory as Option: ("ElectionSubcategory", NS_KR) => |elem| elem.string_value()?,
                domain as Option: ElectionDomain::EML_NAME => |elem| ElectionDomain::read_eml(elem)?,
                election_date as Option: ("ElectionDate", NS_KR) => |elem| elem.string_value()?,
                election_date_eml as Option: ("ElectionDate", NS_EML) => |elem| {
                    if elem.parsing_mode().is_strict() {
                        let err = EMLErrorKind::InvalidElectionDateNamespace.with_span(elem.span());
                        return Err(err);
                    } else {
                        elem.push_err(EMLErrorKind::InvalidElectionDateNamespace.with_span(elem.span()));
                    }
                    elem.string_value()?
                },
            }
        );

        let election_date = match (data.election_date, data.election_date_eml) {
            (Some(date), _) => date,
            (None, Some(date)) => date,
            (None, None) => {
                return Err(
                    EMLErrorKind::MissingElement(OwnedQualifiedName::from_static(
                        "ElectionDate",
                        Some(NS_KR),
                    ))
                    .with_span(elem.full_span()),
                );
            }
        };

        Ok(PollingStationsElectionIdentifier {
            id: data.id,
            name: data.name,
            category: data.category,
            subcategory: data.subcategory,
            domain: data.domain,
            election_date,
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr("Id", self.id.raw().as_ref())?
            .child_option(
                ("ElectionName", NS_EML),
                self.name.as_ref(),
                |elem, value| elem.text(value)?.finish(),
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

/// Contest containing polling stations.
#[derive(Debug, Clone)]
pub struct PollingStationsContest {
    /// Identifier for the contest.
    pub identifier: ContestIdentifierGeen,

    /// Reporting unit for the contest.
    pub reporting_unit: PollingStationsReportingUnit,

    /// Voting method used in the contest.
    pub voting_method: StringValue<VotingMethod>,

    /// Maximum number of votes allowed.
    /// EML specifies the default value as 1, so an empty tag will be parsed as 1 and vice versa.
    pub max_votes: StringValue<NonZeroU64>,

    /// List of polling places in this contest.
    pub polling_places: Vec<PollingPlace>,
}

impl PollingStationsContest {
    /// Create a new builder for constructing a [`PollingStationsContest`].
    pub fn builder() -> PollingStationsContestBuilder {
        PollingStationsContestBuilder::new()
    }
}

/// Builder for the [`PollingStationsContest`] struct.
#[derive(Debug, Clone)]
pub struct PollingStationsContestBuilder {
    reporting_unit: Option<PollingStationsReportingUnit>,
    voting_method: Option<StringValue<VotingMethod>>,
    max_votes: Option<StringValue<NonZeroU64>>,
    polling_places: Vec<PollingPlace>,
}

impl PollingStationsContestBuilder {
    /// Create a new builder for constructing a [`PollingStationsContest`].
    pub fn new() -> Self {
        Self {
            reporting_unit: None,
            voting_method: None,
            max_votes: None,
            polling_places: vec![],
        }
    }

    /// Set the reporting unit for the contest.
    pub fn reporting_unit(
        mut self,
        reporting_unit: impl Into<PollingStationsReportingUnit>,
    ) -> Self {
        self.reporting_unit = Some(reporting_unit.into());
        self
    }

    /// Set the voting method used in the contest.
    pub fn voting_method(mut self, voting_method: impl Into<VotingMethod>) -> Self {
        self.voting_method = Some(StringValue::from_value(voting_method.into()));
        self
    }

    /// Set the maximum number of votes allowed in the contest.
    /// EML specifies the default value as 1, so providing the value of 1 will result in an empty xml tag.
    pub fn max_votes(mut self, max_votes: impl Into<NonZeroU64>) -> Self {
        self.max_votes = Some(StringValue::from_value(max_votes.into()));
        self
    }

    /// Set the list of polling places in the contest. This will replace any
    /// existing polling places set using this method or the [`Self::push_polling_place`]
    /// method.
    pub fn polling_places(mut self, polling_places: impl Into<Vec<PollingPlace>>) -> Self {
        self.polling_places = polling_places.into();
        self
    }

    /// Add a polling place to the contest.
    pub fn push_polling_place(mut self, polling_place: impl Into<PollingPlace>) -> Self {
        self.polling_places.push(polling_place.into());
        self
    }

    /// Build the [`PollingStationsContest`], returning any errors if required fields are missing.
    pub fn build(self) -> Result<PollingStationsContest, EMLError> {
        if self.polling_places.is_empty() {
            return Err(EMLErrorKind::MissingBuildProperty("polling_places").without_span());
        }

        let voting_method = self
            .voting_method
            .ok_or(EMLErrorKind::MissingBuildProperty("voting_method").without_span())?;
        if let Ok(vm) = voting_method.copied_value()
            && vm != VotingMethod::SPV
        {
            return Err(EMLErrorKind::UnsupportedVotingMethod).without_span();
        }

        Ok(PollingStationsContest {
            identifier: ContestIdentifierGeen::default(),
            reporting_unit: self
                .reporting_unit
                .ok_or(EMLErrorKind::MissingBuildProperty("reporting_unit").without_span())?,
            voting_method,
            max_votes: self
                .max_votes
                .ok_or(EMLErrorKind::MissingBuildProperty("max_votes").without_span())?,
            polling_places: self.polling_places,
        })
    }
}

impl Default for PollingStationsContestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EMLElement for PollingStationsContest {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Contest", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        struct PollingStationsContestInternal {
            pub identifier: Option<ContestIdentifierGeen>,
            pub reporting_unit: PollingStationsReportingUnit,
            pub voting_method: StringValue<VotingMethod>,
            pub max_votes: StringValue<NonZeroU64>,
            pub polling_places: Vec<PollingPlace>,
        }

        let data = collect_struct!(elem, PollingStationsContestInternal {
            identifier as Option: ContestIdentifierGeen::EML_NAME => |elem| ContestIdentifierGeen::read_eml(elem)?,
            reporting_unit: PollingStationsReportingUnit::EML_NAME => |elem| PollingStationsReportingUnit::read_eml(elem)?,
            voting_method: ("VotingMethod", NS_EML) => |elem| {
                let value = elem.string_value_opt()?;
                if let Some(value) = value {
                    value
                } else {
                    let err = EMLErrorKind::MissingElementValue(OwnedQualifiedName::from_static("VotingMethod", Some(NS_EML)))
                        .with_span(elem.full_span());
                    if elem.parsing_mode().is_strict() {
                        return Err(err);
                    } else {
                        elem.push_err(err);
                        StringValue::from_value(VotingMethod::SPV)
                    }
                }
            },
            max_votes: ("MaxVotes", NS_EML) => |elem| {
                // Default value of MaxVotes in EML is 1
                let text = elem.text_without_children_opt()?.unwrap_or_else(|| "1".into());
                elem.string_value_from_text(text, None, elem.full_span())?
            },
            polling_places as Vec: PollingPlace::EML_NAME => |elem| PollingPlace::read_eml(elem)?,
        });

        // Some municipalities omit the ContestIdentifier element, even though it is required.
        let identifier = if let Some(identifier) = data.identifier {
            identifier
        } else {
            let err = EMLErrorKind::MissingContenstIdentifier.with_span(elem.span());
            if elem.parsing_mode().is_strict() {
                return Err(err);
            } else {
                elem.push_err(err);
                ContestIdentifierGeen::default()
            }
        };

        // Technically there should be at least one polling place
        if data.polling_places.is_empty() {
            let err = EMLErrorKind::MissingElement(PollingPlace::EML_NAME.as_owned())
                .with_span(elem.full_span());
            if elem.parsing_mode().is_strict() {
                return Err(err);
            } else {
                elem.push_err(err);
            }
        }

        // Only SPV voting method is supported
        if let Ok(vm) = data.voting_method.copied_value()
            && vm != VotingMethod::SPV
        {
            let err = EMLErrorKind::UnsupportedVotingMethod.with_span(elem.full_span());
            if elem.parsing_mode().is_strict() {
                return Err(err);
            } else {
                elem.push_err(err);
            }
        }

        Ok(PollingStationsContest {
            identifier,
            reporting_unit: data.reporting_unit,
            voting_method: data.voting_method,
            max_votes: data.max_votes,
            polling_places: data.polling_places,
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(ContestIdentifier::EML_NAME, &self.identifier)?
            .child_elem(PollingStationsReportingUnit::EML_NAME, &self.reporting_unit)?
            .child(("VotingMethod", NS_EML), |elem| {
                elem.text(self.voting_method.raw().as_ref())?.finish()
            })?
            .child(("MaxVotes", NS_EML), |elem| {
                let raw_text = self.max_votes.raw();
                // Default value of MaxVotes in EML is 1
                if raw_text == "1" {
                    elem.empty()
                } else {
                    elem.text(raw_text.as_ref())?.finish()
                }
            })?
            .child_elems(PollingPlace::EML_NAME, &self.polling_places)?
            .finish()
    }
}

/// Reporting unit for the contest
#[derive(Debug, Clone)]
pub struct PollingStationsReportingUnit {
    /// Identifier of the reporting unit.
    pub identifier: ReportingUnitIdentifier,
}

impl PollingStationsReportingUnit {
    /// Create a new reporting unit for the polling stations document.
    pub fn new(identifier: impl Into<ReportingUnitIdentifier>) -> Self {
        PollingStationsReportingUnit {
            identifier: identifier.into(),
        }
    }
}

impl From<ReportingUnitIdentifier> for PollingStationsReportingUnit {
    fn from(value: ReportingUnitIdentifier) -> Self {
        PollingStationsReportingUnit::new(value)
    }
}

impl EMLElement for PollingStationsReportingUnit {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ReportingUnit", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, PollingStationsReportingUnit {
            identifier: ReportingUnitIdentifier::EML_NAME => |elem| ReportingUnitIdentifier::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(ReportingUnitIdentifier::EML_NAME, &self.identifier)?
            .finish()
    }
}

/// A polling place in the polling stations document.
#[derive(Debug, Clone)]
pub struct PollingPlace {
    /// Voting channel used at this polling place.
    pub channel: StringValue<VotingChannelType>,

    /// Physical location of the polling place.
    pub physical_location: PhysicalLocation,
}

impl PollingPlace {
    /// Create a new builder for constructing a [`PollingPlace`].
    pub fn builder() -> PollingPlaceBuilder {
        PollingPlaceBuilder::new()
    }
}

/// Builder for the [`PollingPlace`] struct.
#[derive(Debug, Clone)]
pub struct PollingPlaceBuilder {
    channel: Option<StringValue<VotingChannelType>>,
    polling_station_id: Option<StringValue<PhysicalLocationPollingStationId>>,
    polling_station_data: Option<Box<str>>,
    locality_name: Option<LocalityName>,
    postal_code: Option<PostalCode>,
}

impl PollingPlaceBuilder {
    /// Create a new builder for constructing a [`PollingPlace`].
    pub fn new() -> Self {
        Self {
            channel: None,
            polling_station_id: None,
            polling_station_data: None,
            locality_name: None,
            postal_code: None,
        }
    }

    /// Set the voting channel used at the polling place.
    pub fn channel(mut self, channel: impl Into<VotingChannelType>) -> Self {
        self.channel = Some(StringValue::from_value(channel.into()));
        self
    }

    /// Set the identifier of the polling station at the polling place.
    pub fn polling_station_id(mut self, id: impl Into<PhysicalLocationPollingStationId>) -> Self {
        self.polling_station_id = Some(StringValue::from_value(id.into()));
        self
    }

    /// Set the additional data of the polling station at the polling place.
    pub fn polling_station_data(self, data: impl Into<Box<str>>) -> Self {
        self.polling_station_data_option(Some(data))
    }

    /// Set the additional data of the polling station at the polling place, if present.
    pub fn polling_station_data_option(mut self, data: Option<impl Into<Box<str>>>) -> Self {
        self.polling_station_data = data.map(|d| d.into());
        self
    }

    /// Set the name of the locality of the polling place.
    pub fn locality_name(mut self, locality_name: impl Into<LocalityName>) -> Self {
        self.locality_name = Some(locality_name.into());
        self
    }

    /// Set the postal code of the locality of the polling place.
    pub fn postal_code(mut self, postal_code: impl Into<PostalCode>) -> Self {
        self.postal_code = Some(postal_code.into());
        self
    }

    /// Build the [`PollingPlace`], returning any errors if required fields are missing.
    pub fn build(self) -> Result<PollingPlace, EMLError> {
        Ok(PollingPlace {
            channel: self
                .channel
                .ok_or(EMLErrorKind::MissingBuildProperty("channel").without_span())?,
            physical_location: PhysicalLocation {
                address: PhysicalLocationAddress {
                    locality: PhysicalLocationLocality {
                        locality_name: self.locality_name.ok_or(
                            EMLErrorKind::MissingBuildProperty("locality_name").without_span(),
                        )?,
                        postal_code: self.postal_code,
                    },
                },
                polling_station: PhysicalLocationPollingStation {
                    id: self.polling_station_id.ok_or(
                        EMLErrorKind::MissingBuildProperty("polling_station_id").without_span(),
                    )?,
                    data: self.polling_station_data.ok_or(
                        EMLErrorKind::MissingBuildProperty("polling_station_data").without_span(),
                    )?,
                },
            },
        })
    }
}

impl Default for PollingPlaceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EMLElement for PollingPlace {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("PollingPlace", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, PollingPlace {
            physical_location: PhysicalLocation::EML_NAME => |elem| PhysicalLocation::read_eml(elem)?,
            channel: elem.string_value_attr("Channel", None)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr("Channel", self.channel.raw().as_ref())?
            .child_elem(PhysicalLocation::EML_NAME, &self.physical_location)?
            .finish()
    }
}

/// Physical location of a polling place.
#[derive(Debug, Clone)]
pub struct PhysicalLocation {
    /// Address of the physical location.
    pub address: PhysicalLocationAddress,

    /// Polling station information of the physical location.
    pub polling_station: PhysicalLocationPollingStation,
}

impl EMLElement for PhysicalLocation {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("PhysicalLocation", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, PhysicalLocation {
            address: PhysicalLocationAddress::EML_NAME => |elem| PhysicalLocationAddress::read_eml(elem)?,
            polling_station: PhysicalLocationPollingStation::EML_NAME => |elem| PhysicalLocationPollingStation::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(PhysicalLocationAddress::EML_NAME, &self.address)?
            .child_elem(
                PhysicalLocationPollingStation::EML_NAME,
                &self.polling_station,
            )?
            .finish()
    }
}

/// Address of a physical location.
#[derive(Debug, Clone)]
pub struct PhysicalLocationAddress {
    /// Locality of the physical location.
    pub locality: PhysicalLocationLocality,
}

impl EMLElement for PhysicalLocationAddress {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Address", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, PhysicalLocationAddress {
            locality: PhysicalLocationLocality::EML_NAME => |elem| PhysicalLocationLocality::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(PhysicalLocationLocality::EML_NAME, &self.locality)?
            .finish()
    }
}

/// Locality of a physical location.
#[derive(Debug, Clone)]
pub struct PhysicalLocationLocality {
    /// Name of the locality.
    pub locality_name: LocalityName,

    /// Postal code of the locality.
    pub postal_code: Option<PostalCode>,
}

impl EMLElement for PhysicalLocationLocality {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Locality", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, PhysicalLocationLocality {
            locality_name: LocalityName::EML_NAME => |elem| LocalityName::read_eml(elem)?,
            postal_code as Option: PostalCode::EML_NAME => |elem| PostalCode::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(LocalityName::EML_NAME, &self.locality_name)?
            .child_elem_option(PostalCode::EML_NAME, self.postal_code.as_ref())?
            .finish()
    }
}

/// Polling station information of a physical location.
#[derive(Debug, Clone)]
pub struct PhysicalLocationPollingStation {
    /// Identifier of the polling station.
    pub id: StringValue<PhysicalLocationPollingStationId>,

    /// Additional data of the polling station.
    pub data: Box<str>,
}

impl EMLElement for PhysicalLocationPollingStation {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("PollingStation", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(PhysicalLocationPollingStation {
            id: elem.string_value_attr("Id", None)?,
            data: elem.text_without_children()?,
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr("Id", self.id.raw().as_ref())?
            .text(self.data.as_ref())?
            .finish()
    }
}

/// Identifier for a physical location polling station.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicalLocationPollingStationId(u64);

impl PhysicalLocationPollingStationId {
    /// Create a new physical location polling station id from the given string, returning an error if the string is not a valid id.
    pub fn new(s: impl AsRef<str>) -> Result<Self, EMLError> {
        Self::parse_from_str(s.as_ref()).wrap_value_error()
    }

    /// Get the value of the physical location polling station id as a u64.
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl From<u64> for PhysicalLocationPollingStationId {
    fn from(value: u64) -> Self {
        PhysicalLocationPollingStationId(value)
    }
}

/// Error returned when a string could not be parsed as a PhysicalLocationPollingStationId
#[derive(Debug, Clone, Error)]
#[error("Invalid polling stations id: {0}")]
pub struct PhysicalLocationPollingStationIdError(String);

/// Regular expression for validating ContestId values.
static PHYSICAL_LOCATION_PS_ID: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\d+)$").expect("Failed to compile Physical Location Polling Station ID regex")
});

impl StringValueData for PhysicalLocationPollingStationId {
    type Error = PhysicalLocationPollingStationIdError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        if PHYSICAL_LOCATION_PS_ID.is_match(s) {
            Ok(PhysicalLocationPollingStationId(s.parse::<u64>().map_err(
                |_| PhysicalLocationPollingStationIdError(s.to_string()),
            )?))
        } else {
            Err(PhysicalLocationPollingStationIdError(s.to_string()))
        }
    }

    fn to_raw_value(&self) -> Box<str> {
        self.0.to_string().into()
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone as _;

    use crate::{
        common::AuthorityIdentifier,
        io::{EMLParsingMode, EMLRead as _, EMLWrite as _},
        utils::{AuthorityId, ReportingUnitIdentifierId},
    };

    use super::*;

    #[test]
    fn test_physical_location_ps_id_regex_compiles() {
        LazyLock::force(&PHYSICAL_LOCATION_PS_ID);
    }

    #[test]
    fn test_polling_stations_construction() {
        let ps = PollingStations::builder()
            .transaction_id(TransactionId::new(1))
            .managing_authority(
                AuthorityIdentifier::new(AuthorityId::new("1234").unwrap()).with_name("Test"),
            )
            .issue_date(XsDate::from_date(2024, 1, 1).unwrap())
            .creation_date_time(chrono::Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap())
            .election_identifier(
                PollingStationsElectionIdentifier::builder()
                    .id(ElectionId::new("TK2025").unwrap())
                    .name("Tweede Kamerverkiezingen 2025")
                    .category(ElectionCategory::TK)
                    .subcategory(ElectionSubcategory::TK)
                    .election_date(XsDate::from_date(2025, 3, 17).unwrap())
                    .build_for_polling_stations()
                    .unwrap(),
            )
            .contests([PollingStationsContest::builder()
                .reporting_unit(ReportingUnitIdentifier::new(
                    ReportingUnitIdentifierId::new("1234").unwrap(),
                    "Test",
                ))
                .max_votes(NonZeroU64::new(20).unwrap())
                .voting_method(VotingMethod::SPV)
                .polling_places([PollingPlace::builder()
                    .locality_name("Amsterdam")
                    .postal_code("1234 AB")
                    .channel(VotingChannelType::Polling)
                    .polling_station_data("123456")
                    .polling_station_id(PhysicalLocationPollingStationId::new("1234").unwrap())
                    .build()
                    .unwrap()])
                .build()
                .unwrap()])
            .build()
            .unwrap();

        let xml = ps.write_eml_root_str(true, true).unwrap();
        assert_eq!(
            xml,
            include_str!(
                "../../test-emls/polling_stations/eml110b_polling_stations_construction_output.eml.xml"
            )
        );

        // check if it still is the same after a second parse and write
        let parsed = PollingStations::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        let xml2 = parsed.write_eml_root_str(true, true).unwrap();
        assert_eq!(xml, xml2);
    }

    #[test]
    fn test_read_polling_stations_with_max_votes_empty() {
        let xml =
            include_str!("../../test-emls/polling_stations/eml110b_empty_number_of_voters.eml.xml");

        let parsed = PollingStations::parse_eml(xml, EMLParsingMode::Strict).unwrap();
        // empty MaxVotes should be parsed to 1, because 1 is the default value according to the EML standard
        assert_eq!(
            parsed.election_event.election.contests[0].max_votes,
            StringValue::Parsed(NonZeroU64::new(1).unwrap())
        );
    }

    #[test]
    fn test_write_polling_stations_with_max_votes_empty() {
        let ps = PollingStations::builder()
            .transaction_id(TransactionId::new(1))
            .managing_authority(
                AuthorityIdentifier::new(AuthorityId::new("1234").unwrap()).with_name("Test"),
            )
            .issue_date(XsDate::from_date(2024, 1, 1).unwrap())
            .creation_date_time(chrono::Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap())
            .election_identifier(
                PollingStationsElectionIdentifier::builder()
                    .id(ElectionId::new("TK2025").unwrap())
                    .name("Tweede Kamerverkiezingen 2025")
                    .category(ElectionCategory::TK)
                    .subcategory(ElectionSubcategory::TK)
                    .election_date(XsDate::from_date(2025, 3, 17).unwrap())
                    .build_for_polling_stations()
                    .unwrap(),
            )
            .contests([PollingStationsContest::builder()
                .reporting_unit(ReportingUnitIdentifier::new(
                    ReportingUnitIdentifierId::new("1234").unwrap(),
                    "Test",
                ))
                .max_votes(NonZeroU64::new(1).unwrap())
                .voting_method(VotingMethod::SPV)
                .polling_places([PollingPlace::builder()
                    .locality_name("Amsterdam")
                    .postal_code("1234 AB")
                    .channel(VotingChannelType::Polling)
                    .polling_station_data("123456")
                    .polling_station_id(PhysicalLocationPollingStationId::new("1234").unwrap())
                    .build()
                    .unwrap()])
                .build()
                .unwrap()])
            .build()
            .unwrap();

        let xml = ps.write_eml_root_str(true, true).unwrap();
        // max_votes of 1 should result in an empty MaxVotes tag, because 1 is the default value according to the EML standard
        assert!(xml.contains("<MaxVotes/>"))
    }

    #[test]
    fn test_empty_polling_stations() {
        assert!(
            PollingStations::parse_eml(
                include_str!(
                    "../../test-emls/polling_stations/eml110b_empty_polling_station.eml.xml"
                ),
                EMLParsingMode::Strict
            )
            .ok_with_errors()
            .is_err()
        )
    }

    #[test]
    fn test_invalid_number_of_voters() {
        assert!(
            PollingStations::parse_eml(
                include_str!(
                    "../../test-emls/polling_stations/eml110b_invalid_number_of_voters.eml.xml"
                ),
                EMLParsingMode::Strict
            )
            .ok_with_errors()
            .is_err()
        )
    }

    #[test]
    fn test_one_station() {
        let ps = PollingStations::parse_eml(
            include_str!("../../test-emls/polling_stations/eml110b_1_station.eml.xml"),
            EMLParsingMode::Strict,
        )
        .unwrap();

        assert_eq!(ps.election_event.election.contests.len(), 1);
        let contest = &ps.election_event.election.contests[0];
        assert_eq!(contest.polling_places.len(), 1);
    }

    #[test]
    fn test_less_than_10_stations() {
        let ps = PollingStations::parse_eml(
            include_str!("../../test-emls/polling_stations/eml110b_less_than_10_stations.eml.xml"),
            EMLParsingMode::Strict,
        )
        .unwrap();

        assert_eq!(ps.election_event.election.contests.len(), 1);
        let contest = &ps.election_event.election.contests[0];
        assert_eq!(contest.polling_places.len(), 9);
    }
}
