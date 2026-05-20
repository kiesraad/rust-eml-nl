//! Document variant for the EML_NL Election Definition (`110a`) document.

use std::{fmt, num::NonZeroU64, str::FromStr};

use instant_xml::{FromXml, ToXml};

use crate::{
    EML_SCHEMA_VERSION, EMLError, NS_EML, NS_KR,
    common::{
        CanonicalizationMethod, ContestIdentifier, CreationDateTime, ElectionDomain, ElectionTree,
        IssueDate, ManagingAuthority, TransactionId,
    },
    documents::ElectionIdentifierBuilder,
    eml_ns_context,
    error::EMLErrorKind,
    utils::{
        ElectionCategory, ElectionId, ElectionSubcategory, StringValue, VotingMethod, XsDate,
        XsDateOrDateTime, XsDateTime,
    },
};

pub(crate) const EML_ELECTION_DEFINITION_ID: &str = "110a";

/// Representing a `110a` document, containing an election definition.
#[derive(Debug, Clone)]
pub struct ElectionDefinition {
    /// Transaction id of the document.
    pub transaction_id: TransactionId,

    /// Managing authority of the election, if present.
    pub managing_authority: Option<ManagingAuthority>,

    /// Issue date of the election definition, if present.
    pub issue_date: Option<IssueDate>,

    /// Time this document was created.
    pub creation_date_time: CreationDateTime,

    /// Canonicalization method used in this document, if present.
    pub canonicalization_method: Option<CanonicalizationMethod>,

    /// The election event defined in this document.
    pub election_event: ElectionDefinitionElectionEvent,
}

impl ElectionDefinition {
    /// Create a new builder for [`ElectionDefinition`].
    pub fn builder() -> ElectionDefinitionBuilder {
        ElectionDefinitionBuilder::new()
    }
}

impl FromStr for ElectionDefinition {
    type Err = EMLError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::io::EMLRead as _;
        Self::parse_eml(s).ok()
    }
}

impl TryFrom<&str> for ElectionDefinition {
    type Error = EMLError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use crate::io::EMLRead as _;
        Self::parse_eml(value).ok()
    }
}

impl TryFrom<ElectionDefinition> for String {
    type Error = EMLError;

    fn try_from(value: ElectionDefinition) -> Result<Self, Self::Error> {
        use crate::io::EMLWrite as _;
        value.write_eml_root_str(true)
    }
}

/// Builder for [`ElectionDefinition`].
#[derive(Debug, Clone)]
pub struct ElectionDefinitionBuilder {
    transaction_id: Option<TransactionId>,
    managing_authority: Option<ManagingAuthority>,
    issue_date: Option<IssueDate>,
    creation_date_time: Option<CreationDateTime>,
    canonicalization_method: Option<CanonicalizationMethod>,
    election_event: Option<ElectionDefinitionElectionEvent>,
    election_identifier: Option<ElectionDefinitionElectionIdentifier>,
    number_of_seats: Option<StringValue<u64>>,
    preference_threshold: Option<StringValue<u64>>,
    election_tree: Option<ElectionTree>,
    registered_parties: Vec<ElectionDefinitionRegisteredParty>,
    contest_identifier: Option<ContestIdentifier>,
    voting_method: Option<StringValue<VotingMethod>>,
    max_votes: Option<StringValue<NonZeroU64>>,
}

impl ElectionDefinitionBuilder {
    /// Create a new builder for [`ElectionDefinition`].
    pub fn new() -> Self {
        Self {
            transaction_id: None,
            managing_authority: None,
            issue_date: None,
            creation_date_time: None,
            canonicalization_method: None,
            election_event: None,
            election_identifier: None,
            number_of_seats: None,
            preference_threshold: None,
            // TODO: election tree not fully supported, defaulting to empty for now
            election_tree: Some(ElectionTree::new(vec![])),
            registered_parties: vec![],
            contest_identifier: None,
            voting_method: None,
            max_votes: None,
        }
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

    /// Set the issue date for the document.
    pub fn issue_date(mut self, issue_date: impl Into<XsDateOrDateTime>) -> Self {
        self.issue_date = Some(IssueDate::new(issue_date.into()));
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

    /// Set the election event containing the election definition
    pub fn election_event(
        mut self,
        election_event: impl Into<ElectionDefinitionElectionEvent>,
    ) -> Self {
        self.election_event = Some(election_event.into());
        self
    }

    /// Set the election identifier for the election event.
    pub fn election_identifier(
        mut self,
        election_identifier: impl Into<ElectionDefinitionElectionIdentifier>,
    ) -> Self {
        self.election_identifier = Some(election_identifier.into());
        self
    }

    /// Set the number of seats for the election.
    pub fn number_of_seats(mut self, number_of_seats: impl Into<u64>) -> Self {
        self.number_of_seats = Some(StringValue::from_value(number_of_seats.into()));
        self
    }

    /// Set the preference threshold for the election.
    pub fn preference_threshold(mut self, preference_threshold: impl Into<u64>) -> Self {
        self.preference_threshold = Some(StringValue::from_value(preference_threshold.into()));
        self
    }

    /// Set the election tree for the election.
    pub fn election_tree(mut self, election_tree: impl Into<ElectionTree>) -> Self {
        self.election_tree = Some(election_tree.into());
        self
    }

    /// Set the registered parties for the election.
    pub fn registered_parties(
        mut self,
        registered_parties: impl Into<Vec<ElectionDefinitionRegisteredParty>>,
    ) -> Self {
        self.registered_parties = registered_parties.into();
        self
    }

    /// Add a registered party to the election.
    pub fn push_registered_party(
        mut self,
        party: impl Into<ElectionDefinitionRegisteredParty>,
    ) -> Self {
        self.registered_parties.push(party.into());
        self
    }

    /// Set the contest identifier for the election.
    pub fn contest_identifier(mut self, contest_identifier: impl Into<ContestIdentifier>) -> Self {
        self.contest_identifier = Some(contest_identifier.into());
        self
    }

    /// Set the voting method for the election.
    pub fn voting_method(mut self, voting_method: impl Into<VotingMethod>) -> Self {
        self.voting_method = Some(StringValue::from_value(voting_method.into()));
        self
    }

    /// Set the maximum number of votes for the election (number of eligible voters).
    pub fn max_votes(mut self, max_votes: impl Into<NonZeroU64>) -> Self {
        self.max_votes = Some(StringValue::from_value(max_votes.into()));
        self
    }

    /// Build the [`ElectionDefinition`] from the provided values, returning an error if any required fields are missing.
    pub fn build(self) -> Result<ElectionDefinition, EMLError> {
        Ok(ElectionDefinition {
            transaction_id: self
                .transaction_id
                .ok_or(EMLErrorKind::MissingBuildProperty("transaction_id").without_span())?,
            managing_authority: self.managing_authority,
            issue_date: self.issue_date,
            creation_date_time: self
                .creation_date_time
                .ok_or(EMLErrorKind::MissingBuildProperty("creation_date_time").without_span())?,
            canonicalization_method: self.canonicalization_method,
            election_event: self.election_event.map_or_else(
                || {
                    let election_identifier = self.election_identifier.ok_or(
                        EMLErrorKind::MissingBuildProperty("election_identifier").without_span(),
                    )?;

                    let election_details = ElectionDefinitionElection {
                        identifier: election_identifier,
                        contest: ElectionDefinitionContest::new(
                            self.contest_identifier.ok_or(
                                EMLErrorKind::MissingBuildProperty("contest_identifier")
                                    .without_span(),
                            )?,
                            self.voting_method.ok_or(
                                EMLErrorKind::MissingBuildProperty("voting_method").without_span(),
                            )?,
                            self.max_votes.ok_or(
                                EMLErrorKind::MissingBuildProperty("max_votes").without_span(),
                            )?,
                        ),
                        number_of_seats: self.number_of_seats.ok_or(
                            EMLErrorKind::MissingBuildProperty("number_of_seats").without_span(),
                        )?,
                        preference_threshold: self.preference_threshold.ok_or(
                            EMLErrorKind::MissingBuildProperty("preference_threshold")
                                .without_span(),
                        )?,
                        election_tree: self.election_tree.ok_or(
                            EMLErrorKind::MissingBuildProperty("election_tree").without_span(),
                        )?,
                        registered_parties: self.registered_parties,
                    };

                    validate_election_details(&election_details)?;

                    Ok(election_details.into())
                },
                Ok,
            )?,
        })
    }
}

fn validate_election_details(election: &ElectionDefinitionElection) -> Result<(), EMLError> {
    let subcategory = election.identifier.subcategory.copied_value().ok();
    let preference_threshold = election.preference_threshold.copied_value().ok();
    let number_of_seats = election.number_of_seats.copied_value().ok();
    let voting_method = election.contest.voting_method.copied_value().ok();
    let mut errors = vec![];
    if let Some(voting_method) = voting_method
        && voting_method != VotingMethod::SPV
    {
        // All election supported should use the SPV voting method
        errors.push(EMLErrorKind::UnsupportedVotingMethod.without_span());
    }

    match (subcategory, preference_threshold, number_of_seats) {
        (Some(ElectionSubcategory::GR1), pt, seats) => {
            if let Some(pt_num) = pt
                && pt_num != 50
            {
                // For GR1 elections: preference threshold should be 50

                errors.push(EMLErrorKind::InvalidPreferenceThreshold.without_span());
            }

            if let Some(seat_count) = seats
                && seat_count >= 19
            {
                // For GR1 elections: number of seats should be less than 19

                errors.push(EMLErrorKind::InvalidNumberOfSeats.without_span());
            }

            // Valid for GR1
        }
        (Some(ElectionSubcategory::GR2), _, Some(seats)) if seats < 19 => {
            // For GR2 elections: number of seats should be 19 or more

            errors.push(EMLErrorKind::InvalidNumberOfSeats.without_span());
        }
        (Some(ElectionSubcategory::AB1), _, Some(seats)) if seats >= 19 => {
            // For AB1 elections: number of seats should be less than 19

            errors.push(EMLErrorKind::InvalidNumberOfSeats.without_span());
        }
        (Some(ElectionSubcategory::AB2), _, Some(seats)) if seats < 19 => {
            // For AB2 elections: number of seats should be 19 or more

            errors.push(EMLErrorKind::InvalidNumberOfSeats.without_span());
        }
        (_, Some(pt), _) if pt != 25 => {
            // For all elections: if preference threshold is provided,
            // it should be 25 (except for GR1 where it should be 50)

            errors.push(EMLErrorKind::InvalidPreferenceThreshold.without_span());
        }
        _ => {
            // Anything else is valid
        }
    }

    if !errors.is_empty() {
        return Err(EMLError::from_vec(errors));
    }

    Ok(())
}

impl Default for ElectionDefinitionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Custom: root EML element with Id/SchemaVersion attributes and full namespace context.
impl<'xml> FromXml<'xml> for ElectionDefinition {
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
        let mut issue_date = <IssueDate as FromXml>::Accumulator::default();
        let mut creation_date_time = <CreationDateTime as FromXml>::Accumulator::default();
        let mut canonicalization_method =
            <CanonicalizationMethod as FromXml>::Accumulator::default();
        let mut election_event =
            <ElectionDefinitionElectionEvent as FromXml>::Accumulator::default();

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
            } else if IssueDate::matches(id, None) {
                let mut nested = deserializer.nested(element);
                IssueDate::deserialize(&mut issue_date, field, &mut nested)?;
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
            } else if ElectionDefinitionElectionEvent::matches(id, None) {
                let mut nested = deserializer.nested(element);
                ElectionDefinitionElectionEvent::deserialize(
                    &mut election_event,
                    field,
                    &mut nested,
                )?;
                nested.ignore()?;
            } else {
                let mut nested = deserializer.nested(element);
                nested.ignore()?;
            }
        }

        let election_event: ElectionDefinitionElectionEvent = election_event.try_done(field)?;

        // Validate date formats are parseable
        let ident = &election_event.election.identifier;
        ident
            .election_date
            .copied_value_err()
            .map_err(|e| Error::Other(e.to_string()))?;
        ident
            .nomination_date
            .copied_value_err()
            .map_err(|e| Error::Other(e.to_string()))?;

        // Validate dates
        crate::documents::validate_election_and_nomination_dates(
            Some(&ident.election_date),
            Some(&ident.nomination_date),
        )
        .map_err(|e| Error::Other(e.to_string()))?;

        // Validate category and subcategory
        crate::documents::validate_category_and_subcategory(
            &ident.category,
            Some(&ident.subcategory),
        )
        .map_err(|e| Error::Other(e.to_string()))?;

        // Validate voting method is parseable
        election_event
            .election
            .contest
            .voting_method
            .copied_value_err()
            .map_err(|e| Error::Other(e.to_string()))?;

        // Validate election details (voting method, seats, preference threshold)
        validate_election_details(&election_event.election)
            .map_err(|e| Error::Other(e.to_string()))?;

        *into = Some(ElectionDefinition {
            transaction_id: transaction_id.try_done(field)?,
            managing_authority,
            issue_date,
            creation_date_time: creation_date_time.try_done(field)?,
            canonicalization_method,
            election_event,
        });
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Element;
}

// Custom: root EML element with Id/SchemaVersion attributes and full namespace context.
impl ToXml for ElectionDefinition {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix = serializer.write_start("EML", NS_EML, Some(eml_ns_context()))?;
        serializer.write_attr("Id", "", EML_ELECTION_DEFINITION_ID)?;
        serializer.write_attr("SchemaVersion", "", EML_SCHEMA_VERSION)?;
        serializer.end_start()?;
        self.transaction_id.serialize(None, serializer)?;

        if let Some(ma) = &self.managing_authority {
            ma.serialize(None, serializer)?;
        }

        self.creation_date_time.serialize(None, serializer)?;
        if let Some(id) = &self.issue_date {
            id.serialize(None, serializer)?;
        }

        self.election_event.serialize(None, serializer)?;
        serializer.write_close(prefix)
    }
}

/// Election event defined in the election definition document.
#[derive(Debug, Clone)]
pub struct ElectionDefinitionElectionEvent {
    /// Election details.
    pub election: ElectionDefinitionElection,
}

impl ElectionDefinitionElectionEvent {
    /// Create a new election event with the provided election details.
    pub fn new(election: impl Into<ElectionDefinitionElection>) -> Self {
        Self {
            election: election.into(),
        }
    }
}

impl From<ElectionDefinitionElection> for ElectionDefinitionElectionEvent {
    fn from(election: ElectionDefinitionElection) -> Self {
        Self::new(election)
    }
}

// Custom: skips `<EventIdentifier/>` element not present in the struct.
impl<'xml> FromXml<'xml> for ElectionDefinitionElectionEvent {
    fn matches(id: instant_xml::Id<'_>, _field: Option<instant_xml::Id<'_>>) -> bool {
        id == instant_xml::Id {
            ns: NS_EML,
            name: "ElectionEvent",
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

        let mut election = <ElectionDefinitionElection as FromXml>::Accumulator::default();

        while let Some(node) = deserializer.next() {
            let element = match node? {
                Node::Open(element) => element,
                Node::Text(s) if s.trim().is_empty() => continue,
                node => return Err(Error::UnexpectedNode(format!("{node:?}"))),
            };

            let id = deserializer.element_id(&element)?;
            if ElectionDefinitionElection::matches(id, None) {
                let mut nested = deserializer.nested(element);
                ElectionDefinitionElection::deserialize(&mut election, field, &mut nested)?;
                nested.ignore()?;
            } else {
                // Skip EventIdentifier and other unknown elements
                let mut nested = deserializer.nested(element);
                nested.ignore()?;
            }
        }

        *into = Some(ElectionDefinitionElectionEvent {
            election: election.try_done(field)?,
        });
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Element;
}

// Custom: injects empty `<EventIdentifier/>` element not present in the struct.
impl ToXml for ElectionDefinitionElectionEvent {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix = serializer.write_start(
            "ElectionEvent",
            NS_EML,
            None::<instant_xml::ser::Context<0>>,
        )?;

        serializer.end_start()?;
        let _ = serializer.write_start(
            "EventIdentifier",
            NS_EML,
            None::<instant_xml::ser::Context<0>>,
        )?;

        serializer.end_empty()?;
        self.election.serialize(None, serializer)?;
        serializer.write_close(prefix)
    }
}

/// Election details for an election definition.
#[derive(Debug, Clone)]
pub struct ElectionDefinitionElection {
    /// Identifier
    pub identifier: ElectionDefinitionElectionIdentifier,

    /// Election voting method details
    pub contest: ElectionDefinitionContest,

    /// Number of seats in the election
    pub number_of_seats: StringValue<u64>,

    /// The preference threshold percentage
    pub preference_threshold: StringValue<u64>,

    /// Election tree for this election
    pub election_tree: ElectionTree,

    /// A list of registered parties.
    pub registered_parties: Vec<ElectionDefinitionRegisteredParty>,
}

// Custom: unwraps `<RegisteredParties>` wrapper element not present in the struct.
impl<'xml> FromXml<'xml> for ElectionDefinitionElection {
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

        let mut identifier =
            <ElectionDefinitionElectionIdentifier as FromXml>::Accumulator::default();
        let mut contest = <ElectionDefinitionContest as FromXml>::Accumulator::default();
        let mut number_of_seats = <StringValue<u64> as FromXml>::Accumulator::default();
        let mut preference_threshold = <StringValue<u64> as FromXml>::Accumulator::default();
        let mut election_tree = <ElectionTree as FromXml>::Accumulator::default();
        let mut registered_parties =
            <Vec<ElectionDefinitionRegisteredParty> as FromXml>::Accumulator::default();

        while let Some(node) = deserializer.next() {
            let element = match node? {
                Node::Open(element) => element,
                Node::Text(s) if s.trim().is_empty() => continue,
                node => return Err(Error::UnexpectedNode(format!("{node:?}"))),
            };

            let id = deserializer.element_id(&element)?;
            if ElectionDefinitionElectionIdentifier::matches(id, None) {
                let mut nested = deserializer.nested(element);
                ElectionDefinitionElectionIdentifier::deserialize(
                    &mut identifier,
                    field,
                    &mut nested,
                )?;
                nested.ignore()?;
            } else if ElectionDefinitionContest::matches(id, None) {
                let mut nested = deserializer.nested(element);
                ElectionDefinitionContest::deserialize(&mut contest, field, &mut nested)?;
                nested.ignore()?;
            } else if id
                == (instant_xml::Id {
                    ns: NS_KR,
                    name: "NumberOfSeats",
                })
            {
                let mut nested = deserializer.nested(element);
                <StringValue<u64>>::deserialize(&mut number_of_seats, field, &mut nested)?;
                nested.ignore()?;
            } else if id
                == (instant_xml::Id {
                    ns: NS_KR,
                    name: "PreferenceThreshold",
                })
            {
                let mut nested = deserializer.nested(element);
                <StringValue<u64>>::deserialize(&mut preference_threshold, field, &mut nested)?;
                nested.ignore()?;
            } else if ElectionTree::matches(id, None) {
                let mut nested = deserializer.nested(element);
                ElectionTree::deserialize(&mut election_tree, field, &mut nested)?;
                nested.ignore()?;
            } else if id
                == (instant_xml::Id {
                    ns: NS_KR,
                    name: "RegisteredParties",
                })
            {
                // Unwrap the <RegisteredParties> wrapper
                let mut nested = deserializer.nested(element);
                while let Some(inner) = nested.next() {
                    let inner = match inner? {
                        Node::Open(inner) => inner,
                        Node::Text(s) if s.trim().is_empty() => continue,
                        node => return Err(Error::UnexpectedNode(format!("{node:?}"))),
                    };

                    let inner_id = nested.element_id(&inner)?;
                    if <Vec<ElectionDefinitionRegisteredParty> as FromXml>::matches(inner_id, None)
                    {
                        let mut inner_nested = nested.nested(inner);
                        <Vec<ElectionDefinitionRegisteredParty> as FromXml>::deserialize(
                            &mut registered_parties,
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

        let result = ElectionDefinitionElection {
            identifier: identifier.try_done(field)?,
            contest: contest.try_done(field)?,
            number_of_seats: number_of_seats.try_done(field)?,
            preference_threshold: preference_threshold.try_done(field)?,
            election_tree: election_tree.try_done(field)?,
            registered_parties: registered_parties.try_done(field)?,
        };

        // Validate election details
        validate_election_details(&result).map_err(|e| instant_xml::Error::Other(e.to_string()))?;

        *into = Some(result);
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Element;
}

// Custom: wraps parties in `<RegisteredParties>` wrapper; empty-element when list is empty.
impl ToXml for ElectionDefinitionElection {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix =
            serializer.write_start("Election", NS_EML, None::<instant_xml::ser::Context<0>>)?;
        serializer.end_start()?;
        self.identifier.serialize(None, serializer)?;
        self.contest.serialize(None, serializer)?;
        self.election_tree.serialize(None, serializer)?;

        self.number_of_seats.serialize(
            Some(instant_xml::Id {
                ns: NS_KR,
                name: "NumberOfSeats",
            }),
            serializer,
        )?;

        self.preference_threshold.serialize(
            Some(instant_xml::Id {
                ns: NS_KR,
                name: "PreferenceThreshold",
            }),
            serializer,
        )?;

        let rp_prefix = serializer.write_start(
            "RegisteredParties",
            NS_KR,
            None::<instant_xml::ser::Context<0>>,
        )?;

        if self.registered_parties.is_empty() {
            serializer.end_empty()?;
        } else {
            serializer.end_start()?;
            for party in &self.registered_parties {
                party.serialize(None, serializer)?;
            }
            serializer.write_close(rp_prefix)?;
        }

        serializer.write_close(prefix)
    }
}

/// Identifier for the election.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "ElectionIdentifier", ns(NS_EML, kr = NS_KR))]
pub struct ElectionDefinitionElectionIdentifier {
    /// Id of the election
    #[xml(attribute, rename = "Id")]
    pub id: StringValue<ElectionId>,

    /// Name of the election
    #[xml(rename = "ElectionName")]
    pub name: String,

    /// Category of the election
    #[xml(rename = "ElectionCategory")]
    pub category: StringValue<ElectionCategory>,

    /// Subcategory of the election
    #[xml(rename = "ElectionSubcategory", ns(NS_KR))]
    pub subcategory: StringValue<ElectionSubcategory>,

    /// The (top level) region where the election takes place.
    #[xml(rename = "ElectionDomain")]
    pub domain: Option<ElectionDomain>,

    /// Date of the election
    #[xml(rename = "ElectionDate", ns(NS_KR))]
    pub election_date: StringValue<XsDate>,

    /// Nomination date for the election
    #[xml(rename = "NominationDate", ns(NS_KR))]
    pub nomination_date: StringValue<XsDate>,
}

impl ElectionDefinitionElectionIdentifier {
    /// Create a builder for the [`ElectionDefinitionElectionIdentifier`].
    /// Note: this is a generic builder, use the [`ElectionIdentifierBuilder::build_for_definition`] method for an election definition specific builder that will fill in the correct category and subcategory.
    pub fn builder() -> ElectionIdentifierBuilder {
        ElectionIdentifierBuilder::new()
    }
}

/// Contains details about the voting methods for the election.
#[derive(Debug, Clone)]
pub struct ElectionDefinitionContest {
    /// Identifier for the contest.
    pub identifier: ContestIdentifier,

    /// Voting method used in the contest.
    pub voting_method: StringValue<VotingMethod>,

    /// Maximum number of votes allowed.
    pub max_votes: StringValue<NonZeroU64>,
}

impl ElectionDefinitionContest {
    /// Create a new contest with the provided details.
    pub fn new(
        identifier: impl Into<ContestIdentifier>,
        voting_method: impl Into<StringValue<VotingMethod>>,
        max_votes: impl Into<StringValue<NonZeroU64>>,
    ) -> Self {
        Self {
            identifier: identifier.into(),
            voting_method: voting_method.into(),
            max_votes: max_votes.into(),
        }
    }
}

// Custom: empty `<MaxVotes/>` element parsed as value "1".
impl<'xml> FromXml<'xml> for ElectionDefinitionContest {
    fn matches(id: instant_xml::Id<'_>, _field: Option<instant_xml::Id<'_>>) -> bool {
        id == instant_xml::Id {
            ns: NS_EML,
            name: "Contest",
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

        let mut identifier = <ContestIdentifier as FromXml>::Accumulator::default();
        let mut voting_method = <StringValue<VotingMethod> as FromXml>::Accumulator::default();
        let mut max_votes = <StringValue<NonZeroU64> as FromXml>::Accumulator::default();

        while let Some(node) = deserializer.next() {
            let element = match node? {
                Node::Open(element) => element,
                Node::Text(s) if s.trim().is_empty() => continue,
                node => return Err(Error::UnexpectedNode(format!("{node:?}"))),
            };

            let id = deserializer.element_id(&element)?;
            if ContestIdentifier::matches(id, None) {
                let mut nested = deserializer.nested(element);
                ContestIdentifier::deserialize(&mut identifier, field, &mut nested)?;
                nested.ignore()?;
            } else if id
                == (instant_xml::Id {
                    ns: NS_EML,
                    name: "VotingMethod",
                })
            {
                let mut nested = deserializer.nested(element);
                <StringValue<VotingMethod>>::deserialize(&mut voting_method, field, &mut nested)?;
                nested.ignore()?;
            } else if id
                == (instant_xml::Id {
                    ns: NS_EML,
                    name: "MaxVotes",
                })
            {
                let mut nested = deserializer.nested(element);
                // MaxVotes: empty element means "1"
                let mut has_content = false;
                while let Some(inner) = nested.next() {
                    let inner = inner?;
                    match inner {
                        Node::Text(text) => {
                            has_content = true;
                            let sv = StringValue::Raw(text.to_string());
                            max_votes = Some(sv);
                        }
                        Node::Open(el) => {
                            let mut n = nested.nested(el);
                            n.ignore()?;
                        }
                        _ => {}
                    }
                }
                if !has_content {
                    max_votes = Some(StringValue::Raw("1".to_string()));
                }
            } else {
                let mut nested = deserializer.nested(element);
                nested.ignore()?;
            }
        }

        *into = Some(ElectionDefinitionContest {
            identifier: identifier.try_done(field)?,
            voting_method: voting_method.try_done(field)?,
            max_votes: max_votes.try_done(field)?,
        });
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Element;
}

// Custom: `<MaxVotes>` uses empty-element when value is "1", otherwise with content.
impl ToXml for ElectionDefinitionContest {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix =
            serializer.write_start("Contest", NS_EML, None::<instant_xml::ser::Context<0>>)?;
        serializer.end_start()?;
        self.identifier.serialize(None, serializer)?;

        self.voting_method.serialize(
            Some(instant_xml::Id {
                ns: NS_EML,
                name: "VotingMethod",
            }),
            serializer,
        )?;

        let raw_text = self.max_votes.raw();
        let mv_prefix =
            serializer.write_start("MaxVotes", NS_EML, None::<instant_xml::ser::Context<0>>)?;
        if raw_text == "1" {
            serializer.end_empty()?;
        } else {
            serializer.end_start()?;
            serializer.write_str(raw_text.as_ref())?;
            serializer.write_close(mv_prefix)?;
        }

        serializer.write_close(prefix)
    }
}

/// A registered party in the election definition.
///
/// In election definitions this is just a party name, for full party details and
/// candidates see the [`CandidateLists`](crate::documents::candidate_lists::CandidateLists)
/// document.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(
    rename = "RegisteredParty",
    rename_all = "PascalCase",
    ns(NS_KR),
    force_prefix
)]
pub struct ElectionDefinitionRegisteredParty {
    /// Name of the registered party (as registered at the CSB)
    pub registered_appellation: String,
}

impl ElectionDefinitionRegisteredParty {
    /// Create a new registered party with the provided name.
    pub fn new(registered_appellation: impl Into<String>) -> Self {
        Self {
            registered_appellation: registered_appellation.into(),
        }
    }
}

impl From<String> for ElectionDefinitionRegisteredParty {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for ElectionDefinitionRegisteredParty {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone as _;

    use crate::{
        documents::EML,
        io::{EMLRead as _, EMLWrite},
        utils::AuthorityId,
    };

    use super::*;

    #[test]
    fn test_election_definition_construction() {
        let election_definition = ElectionDefinition::builder()
            .transaction_id(TransactionId::new(1))
            .managing_authority(ManagingAuthority::new(AuthorityId::new("1234").unwrap()))
            .issue_date(XsDate::from_date(2024, 6, 10).unwrap())
            .creation_date_time(
                chrono::Utc
                    .with_ymd_and_hms(2014, 11, 28, 12, 0, 9)
                    .unwrap(),
            )
            .election_identifier(
                ElectionDefinitionElectionIdentifier::builder()
                    .id(ElectionId::new("GR2026_Test").unwrap())
                    .name("Test election")
                    .category(ElectionCategory::GR)
                    .subcategory(ElectionSubcategory::GR1)
                    .election_date(XsDate::from_date(2024, 11, 5).unwrap())
                    .nomination_date(XsDate::from_date(2024, 10, 1).unwrap())
                    .build_for_definition()
                    .unwrap(),
            )
            .contest_identifier(ContestIdentifier::geen())
            .voting_method(VotingMethod::SPV)
            .max_votes(NonZeroU64::new(100).unwrap())
            .number_of_seats(10u32)
            .preference_threshold(50u32)
            .push_registered_party("Party a")
            .push_registered_party("Party one")
            .build()
            .unwrap();

        let xml = election_definition.write_eml_root_str(true).unwrap();

        // check if it still is the same after a second parse and write
        let eml = EML::parse_eml(&xml).unwrap();
        let parsed = eml
            .as_election_definition_doc()
            .expect("expected election definition variant");
        let xml2 = parsed.write_eml_root_str(true).unwrap();
        assert_eq!(xml, xml2);
    }

    #[test]
    fn test_invalid_election_date_format() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_date_format.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml).ok_with_errors();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_election_date_nomination_format() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_date_nomination_format.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml).ok_with_errors();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_election_mismatch_preference_threshold() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_mismatch_preference_threshold.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml).ok_with_errors();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_election_mismatch_preference_threshold_small_election() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_mismatch_preference_threshold_small_election.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml).ok_with_errors();
        assert!(result.is_err());
    }

    #[test]
    fn test_election_missing_election_domain() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_election_missing_election_domain.eml.xml"
        );

        let eml = EML::parse_eml(xml).ok_with_errors();
        assert!(eml.unwrap().0.as_election_definition_doc().is_some());
    }

    #[test]
    fn test_invalid_election_missing_election_tree() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_missing_election_tree.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml).ok_with_errors();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_election_missing_nomination_date() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_missing_nomination_date.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml).ok_with_errors();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_election_missing_number_of_seats() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_missing_number_of_seats.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml).ok_with_errors();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_election_missing_preference_threshold() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_missing_preference_threshold.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml).ok_with_errors();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_election_missing_subcategory() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_missing_subcategory.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml).ok_with_errors();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_election_number_of_seats() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_number_of_seats.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml).ok_with_errors();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_election_subcategory() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_subcategory.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml).ok_with_errors();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_election_voting_method() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_voting_method.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml).ok_with_errors();
        assert!(result.is_err());
    }
}
