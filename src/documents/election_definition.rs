//! Document variant for the EML_NL Election Definition (`110a`) document.

use std::{num::NonZeroU64, str::FromStr};

use crate::{
    EML_SCHEMA_VERSION, EMLError, NS_EML, NS_KR,
    common::{
        CanonicalizationMethod, ContestIdentifier, CreationDateTime, ElectionDomain, ElectionTree,
        IssueDate, ManagingAuthority, TransactionId,
    },
    documents::{ElectionIdentifierBuilder, accepted_root},
    error::{EMLErrorKind, EMLResultExt},
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName, collect_struct},
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
        Self::parse_eml(s, crate::io::EMLParsingMode::Strict).ok()
    }
}

impl TryFrom<&str> for ElectionDefinition {
    type Error = EMLError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use crate::io::EMLRead as _;
        Self::parse_eml(value, crate::io::EMLParsingMode::Strict).ok()
    }
}

impl TryFrom<ElectionDefinition> for String {
    type Error = EMLError;

    fn try_from(value: ElectionDefinition) -> Result<Self, Self::Error> {
        use crate::io::EMLWrite as _;
        value.write_eml_root_str(true, true)
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

impl EMLElement for ElectionDefinition {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("EML", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        accepted_root(elem)?;

        let document_id = elem.attribute_value_req(("Id", None))?;
        if document_id != EML_ELECTION_DEFINITION_ID {
            return Err(EMLErrorKind::InvalidDocumentType(
                EML_ELECTION_DEFINITION_ID,
                document_id.to_string(),
            ))
            .with_span(elem.span());
        }

        Ok(collect_struct!(elem, ElectionDefinition {
            transaction_id: TransactionId::EML_NAME => |elem| TransactionId::read_eml(elem)?,
            managing_authority as Option: ManagingAuthority::EML_NAME => |elem| ManagingAuthority::read_eml(elem)?,
            issue_date as Option: IssueDate::EML_NAME => |elem| IssueDate::read_eml(elem)?,
            creation_date_time: CreationDateTime::EML_NAME => |elem| CreationDateTime::read_eml(elem)?,
            canonicalization_method as Option: CanonicalizationMethod::EML_NAME => |elem| CanonicalizationMethod::read_eml(elem)?,
            election_event: ElectionDefinitionElectionEvent::EML_NAME => |elem| ElectionDefinitionElectionEvent::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr(("Id", None), EML_ELECTION_DEFINITION_ID)?
            .attr(("SchemaVersion", None), EML_SCHEMA_VERSION)?
            .child_elem(TransactionId::EML_NAME, &self.transaction_id)?
            .child_elem_option(
                ManagingAuthority::EML_NAME,
                self.managing_authority.as_ref(),
            )?
            .child_elem(CreationDateTime::EML_NAME, &self.creation_date_time)?
            .child_elem_option(IssueDate::EML_NAME, self.issue_date.as_ref())?
            // Note: we don't output the CanonicalizationMethod because we aren't canonicalizing our output
            // .child_elem_option(
            //     CanonicalizationMethod::EML_NAME,
            //     self.canonicalization_method.as_ref(),
            // )?
            .child_elem(
                ElectionDefinitionElectionEvent::EML_NAME,
                &self.election_event,
            )?
            .finish()
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

impl EMLElement for ElectionDefinitionElectionEvent {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ElectionEvent", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, ElectionDefinitionElectionEvent {
            id as None: ("EventIdentifier", NS_EML) => |elem| elem.skip().map(|_| ())?,
            election: ElectionDefinitionElection::EML_NAME => |elem| ElectionDefinitionElection::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child(("EventIdentifier", NS_EML), |w| w.empty())?
            .child_elem(ElectionDefinitionElection::EML_NAME, &self.election)?
            .finish()
    }
}

/// Name for the number of seats element
const EML_NAME_NUMBER_OF_SEATS: QualifiedName<'_, '_> =
    QualifiedName::from_static("NumberOfSeats", Some(NS_KR));

/// Name for the preference threshold element
const EML_NAME_PREFERENCE_THRESHOLD: QualifiedName<'_, '_> =
    QualifiedName::from_static("PreferenceThreshold", Some(NS_KR));

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

impl EMLElement for ElectionDefinitionElection {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Election", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let data = collect_struct!(elem, ElectionDefinitionElection {
            identifier: ElectionDefinitionElectionIdentifier::EML_NAME => |elem| ElectionDefinitionElectionIdentifier::read_eml(elem)?,
            contest: ElectionDefinitionContest::EML_NAME => |elem| ElectionDefinitionContest::read_eml(elem)?,
            number_of_seats: EML_NAME_NUMBER_OF_SEATS => |elem| elem.string_value()?,
            preference_threshold: EML_NAME_PREFERENCE_THRESHOLD => |elem| elem.string_value()?,
            election_tree: ElectionTree::EML_NAME => |elem| ElectionTree::read_eml(elem)?,
            registered_parties: ("RegisteredParties", NS_KR) => |elem| ElectionDefinitionRegisteredParty::read_list(elem)?,
        });

        if let Err(errors) = validate_election_details(&data) {
            if elem.parsing_mode().is_strict() {
                return Err(errors);
            } else {
                elem.push_err(errors);
            }
        }

        Ok(data)
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(
                ElectionDefinitionElectionIdentifier::EML_NAME,
                &self.identifier,
            )?
            .child_elem(ElectionDefinitionContest::EML_NAME, &self.contest)?
            .child_elem(ElectionTree::EML_NAME, &self.election_tree)?
            .child(EML_NAME_NUMBER_OF_SEATS, |elem| {
                elem.text(self.number_of_seats.raw().as_ref())?.finish()
            })?
            .child(EML_NAME_PREFERENCE_THRESHOLD, |elem| {
                elem.text(self.preference_threshold.raw().as_ref())?
                    .finish()
            })?
            .child(("RegisteredParties", NS_KR), |elem| {
                ElectionDefinitionRegisteredParty::write_list(&self.registered_parties, elem)
            })?
            .finish()
    }
}

/// Identifier for the election.
#[derive(Debug, Clone)]
pub struct ElectionDefinitionElectionIdentifier {
    /// Id of the election
    pub id: StringValue<ElectionId>,

    /// Name of the election
    pub name: String,

    /// Category of the election
    pub category: StringValue<ElectionCategory>,

    /// Subcategory of the election
    pub subcategory: StringValue<ElectionSubcategory>,

    /// The (top level) region where the election takes place.
    pub domain: Option<ElectionDomain>,

    /// Date of the election
    pub election_date: StringValue<XsDate>,

    /// Nomination date for the election
    pub nomination_date: StringValue<XsDate>,
}

impl ElectionDefinitionElectionIdentifier {
    /// Create a builder for the [`ElectionDefinitionElectionIdentifier`].
    /// Note: this is a generic builder, use the [`ElectionIdentifierBuilder::build_for_definition`] method for an election definition specific builder that will fill in the correct category and subcategory.
    pub fn builder() -> ElectionIdentifierBuilder {
        ElectionIdentifierBuilder::new()
    }
}

impl EMLElement for ElectionDefinitionElectionIdentifier {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ElectionIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(
            elem,
            ElectionDefinitionElectionIdentifier {
                id: elem.string_value_attr("Id", None)?,
                name: ("ElectionName", NS_EML) => |elem| elem.text_without_children()?,
                category: ("ElectionCategory", NS_EML) => |elem| elem.string_value()?,
                subcategory: ("ElectionSubcategory", NS_KR) => |elem| elem.string_value()?,
                domain as Option: ElectionDomain::EML_NAME => |elem| ElectionDomain::read_eml(elem)?,
                election_date: ("ElectionDate", NS_KR) => |elem| elem.string_value()?,
                nomination_date: ("NominationDate", NS_KR) => |elem| elem.string_value()?,
            }
        ))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr("Id", self.id.raw().as_ref())?
            .child(("ElectionName", NS_EML), |elem| {
                elem.text(self.name.as_ref())?.finish()
            })?
            .child(("ElectionCategory", NS_EML), |elem| {
                elem.text(self.category.raw().as_ref())?.finish()
            })?
            .child(("ElectionSubcategory", NS_KR), |elem| {
                elem.text(self.subcategory.raw().as_ref())?.finish()
            })?
            .child_elem_option(ElectionDomain::EML_NAME, self.domain.as_ref())?
            .child(("ElectionDate", NS_KR), |elem| {
                elem.text(self.election_date.raw().as_ref())?.finish()
            })?
            .child(("NominationDate", NS_KR), |elem| {
                elem.text(self.nomination_date.raw().as_ref())?.finish()
            })?
            .finish()
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

impl EMLElement for ElectionDefinitionContest {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Contest", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, ElectionDefinitionContest {
            identifier: ContestIdentifier::EML_NAME => |elem| ContestIdentifier::read_eml(elem)?,
            voting_method: ("VotingMethod", NS_EML) => |elem| elem.string_value()?,
            max_votes: ("MaxVotes", NS_EML) => |elem| {
                let text = elem.text_without_children_opt()?.unwrap_or_else(|| "1".to_string());
                elem.string_value_from_text(text, None, elem.full_span())?
            },
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(ContestIdentifier::EML_NAME, &self.identifier)?
            .child(("VotingMethod", NS_EML), |elem| {
                elem.text(self.voting_method.raw().as_ref())?.finish()
            })?
            .child(("MaxVotes", NS_EML), |elem| {
                let raw_text = self.max_votes.raw();
                if raw_text == "1" {
                    elem.empty()
                } else {
                    elem.text(raw_text.as_ref())?.finish()
                }
            })?
            .finish()
    }
}

/// A registered party in the election definition.
///
/// In election definitions this is just a party name, for full party details and
/// candidates see the [`CandidateLists`](crate::documents::candidate_lists::CandidateLists)
/// document.
#[derive(Debug, Clone)]
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

impl ElectionDefinitionRegisteredParty {
    pub(crate) fn read_list(
        elem: &mut EMLElementReader<'_, '_>,
    ) -> Result<Vec<ElectionDefinitionRegisteredParty>, EMLError> {
        let mut parties = Vec::new();
        let elem_name = elem.name()?.as_owned();
        while let Some(mut child) = elem.next_child()? {
            if child.has_name(ElectionDefinitionRegisteredParty::EML_NAME)? {
                let party = ElectionDefinitionRegisteredParty::read_eml(&mut child)?;
                parties.push(party);
            } else {
                return Err(EMLErrorKind::UnexpectedElement(
                    child.name()?.as_owned(),
                    elem_name,
                ))
                .with_span(child.span());
            }
        }
        Ok(parties)
    }

    pub(crate) fn write_list(
        parties: &[ElectionDefinitionRegisteredParty],
        writer: EMLElementWriter,
    ) -> Result<(), EMLError> {
        if parties.is_empty() {
            return writer.empty();
        }

        let mut content = writer.content()?;
        for party in parties {
            content = content.child_elem(ElectionDefinitionRegisteredParty::EML_NAME, party)?;
        }
        content.finish()
    }
}

impl EMLElement for ElectionDefinitionRegisteredParty {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("RegisteredParty", Some(NS_KR));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, ElectionDefinitionRegisteredParty {
            registered_appellation: ("RegisteredAppellation", NS_KR) => |elem| elem.text_without_children()?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child(("RegisteredAppellation", NS_KR), |elem| {
                elem.text(self.registered_appellation.as_ref())?.finish()
            })?
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone as _;

    use crate::{
        io::{EMLParsingMode, EMLRead as _, EMLWrite},
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

        let xml = election_definition.write_eml_root_str(true, true).unwrap();
        assert_eq!(
            xml,
            include_str!(
                "../../test-emls/election_definition/eml110a_election_definition_construction_output.eml.xml"
            )
        );

        // check if it still is the same after a second parse and write
        let parsed = ElectionDefinition::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        let xml2 = parsed.write_eml_root_str(true, true).unwrap();
        assert_eq!(xml, xml2);
    }

    #[test]
    fn test_election_definition_with_max_votes_empty() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_test.eml.xml"
        );

        let parsed = ElectionDefinition::parse_eml(xml, EMLParsingMode::Strict).unwrap();
        // empty MaxVotes should be parsed to 1, because that's the default value according to the EML standard
        assert_eq!(parsed.election_event.election.contest.max_votes, StringValue::Parsed(NonZeroU64::new(1).unwrap()));
    }

    #[test]
    fn test_invalid_election_date_format() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_date_format.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml, EMLParsingMode::Strict).ok_with_errors();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_election_date_nomination_format() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_date_nomination_format.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml, EMLParsingMode::Strict).ok_with_errors();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_election_mismatch_preference_threshold() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_mismatch_preference_threshold.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml, EMLParsingMode::Strict).ok_with_errors();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_election_mismatch_preference_threshold_small_election() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_mismatch_preference_threshold_small_election.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml, EMLParsingMode::Strict).ok_with_errors();
        assert!(result.is_err());
    }

    #[test]
    fn test_election_missing_election_domain() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_election_missing_election_domain.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml, EMLParsingMode::Strict).ok_with_errors();
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_election_missing_election_tree() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_missing_election_tree.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml, EMLParsingMode::Strict).ok_with_errors();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_election_missing_nomination_date() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_missing_nomination_date.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml, EMLParsingMode::Strict).ok_with_errors();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_election_missing_number_of_seats() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_missing_number_of_seats.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml, EMLParsingMode::Strict).ok_with_errors();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_election_missing_preference_threshold() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_missing_preference_threshold.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml, EMLParsingMode::Strict).ok_with_errors();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_election_missing_subcategory() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_missing_subcategory.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml, EMLParsingMode::Strict).ok_with_errors();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_election_number_of_seats() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_number_of_seats.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml, EMLParsingMode::Strict).ok_with_errors();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_election_subcategory() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_subcategory.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml, EMLParsingMode::Strict).ok_with_errors();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_election_voting_method() {
        let xml = include_str!(
            "../../test-emls/election_definition/eml110a_invalid_election_voting_method.eml.xml"
        );

        let result = ElectionDefinition::parse_eml(xml, EMLParsingMode::Strict).ok_with_errors();
        assert!(result.is_err());
    }
}
