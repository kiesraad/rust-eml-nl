//! Document variant for the EML_NL Election Definition (`110a`) document.

use crate::{
    EML_SCHEMA_VERSION, EMLError, NS_EML, NS_KR,
    common::{CreationDateTime, ElectionDomain, IssueDate, ManagingAuthority, TransactionId},
    documents::accepted_root,
    error::{EMLErrorKind, EMLResultExt},
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName, collect_struct},
    utils::{
        ContestIdType, ElectionCategory, ElectionIdType, ElectionSubcategory, StringValue,
        VotingMethod, XsDate,
    },
};

pub(crate) const EML_ELECTION_DEFINITION_ID: &str = "110a";

/// Representing a `110a` document, containing an election definition.
#[derive(Debug, Clone)]
pub struct ElectionDefinition {
    /// Transaction id of the document.
    pub transaction_id: TransactionId,
    /// Time this document was created.
    pub creation_date_time: CreationDateTime,
    /// Issue date of the election definition, if present.
    pub issue_date: Option<IssueDate>,
    /// Managing authority of the election, if present.
    pub managing_authority: Option<ManagingAuthority>,
    /// The election event defined in this document.
    pub election_event: ElectionDefinitionElectionEvent,
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
            creation_date_time: CreationDateTime::EML_NAME => |elem| CreationDateTime::read_eml(elem)?,
            issue_date as Option: IssueDate::EML_NAME => |elem| IssueDate::read_eml(elem)?,
            managing_authority as Option: ManagingAuthority::EML_NAME => |elem| ManagingAuthority::read_eml(elem)?,
            election_event: ElectionDefinitionElectionEvent::EML_NAME => |elem| ElectionDefinitionElectionEvent::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr(("Id", None), EML_ELECTION_DEFINITION_ID)?
            .attr(("SchemaVersion", None), EML_SCHEMA_VERSION)?
            .child_elem(TransactionId::EML_NAME, &self.transaction_id)?
            .child_elem(CreationDateTime::EML_NAME, &self.creation_date_time)?
            .child_elem_option(IssueDate::EML_NAME, self.issue_date.as_ref())?
            .child_elem_option(
                ManagingAuthority::EML_NAME,
                self.managing_authority.as_ref(),
            )?
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

impl EMLElement for ElectionDefinitionElectionEvent {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ElectionEvent", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, ElectionDefinitionElectionEvent {
            election: ElectionDefinitionElection::EML_NAME => |elem| ElectionDefinitionElection::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child(("EventIdentifier", NS_EML), |elem| elem.empty())?
            .child_elem(ElectionDefinitionElection::EML_NAME, &self.election)?
            .finish()
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
    /// A list of registered parties.
    pub registered_parties: Vec<ElectionDefinitionRegisteredParty>,
}

impl EMLElement for ElectionDefinitionElection {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Election", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, ElectionDefinitionElection {
            identifier: ElectionDefinitionElectionIdentifier::EML_NAME => |elem| ElectionDefinitionElectionIdentifier::read_eml(elem)?,
            contest: ElectionDefinitionContest::EML_NAME => |elem| ElectionDefinitionContest::read_eml(elem)?,
            number_of_seats: ("NumberOfSeats", NS_KR) => |elem| StringValue::<u64>::from_maybe_read_parsed_err(elem, ("NumberOfSeats", NS_KR))?,
            preference_threshold: ("PreferenceThreshold", NS_KR) => |elem| StringValue::<u64>::from_maybe_read_parsed_err(elem, ("PreferenceThreshold", NS_KR))?,
            registered_parties: ("RegisteredParties", NS_KR) => |elem| ElectionDefinitionRegisteredParty::read_list(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(
                ElectionDefinitionElectionIdentifier::EML_NAME,
                &self.identifier,
            )?
            .child_elem(ElectionDefinitionContest::EML_NAME, &self.contest)?
            .child(("NumberOfSeats", NS_KR), |elem| {
                elem.text(self.number_of_seats.raw().as_ref())?.finish()
            })?
            .child(("PreferenceThreshold", NS_KR), |elem| {
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
    pub id: StringValue<ElectionIdType>,
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

impl EMLElement for ElectionDefinitionElectionIdentifier {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ElectionIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(
            elem,
            ElectionDefinitionElectionIdentifier {
                id: StringValue::from_maybe_parsed_err(
                    elem.attribute_value_req("Id")?.into_owned(),
                    elem.strict_value_parsing(),
                    "Id",
                    Some(elem.span()),
                )?,
                name: ("ElectionName", NS_EML) => |elem| elem.text_without_children()?,
                category: ("ElectionCategory", NS_EML) => |elem| StringValue::from_maybe_read_parsed_err(elem, ("ElectionCategory", NS_EML))?,
                subcategory: ("ElectionSubcategory", NS_KR) => |elem| StringValue::from_maybe_read_parsed_err(elem, ("ElectionSubcategory", NS_KR))?,
                domain as Option: ElectionDomain::EML_NAME => |elem| ElectionDomain::read_eml(elem)?,
                election_date: ("ElectionDate", NS_KR) => |elem| StringValue::from_maybe_read_parsed_err(elem, ("ElectionDate", NS_KR))?,
                nomination_date: ("NominationDate", NS_KR) => |elem| StringValue::from_maybe_read_parsed_err(elem, ("NominationDate", NS_KR))?,
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
    pub max_votes: StringValue<u64>,
}

impl EMLElement for ElectionDefinitionContest {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Contest", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, ElectionDefinitionContest {
            identifier: ContestIdentifier::EML_NAME => |elem| ContestIdentifier::read_eml(elem)?,
            voting_method: ("VotingMethod", NS_EML) => |elem| StringValue::from_maybe_read_parsed_err(elem, ("VotingMethod", NS_EML))?,
            max_votes: ("MaxVotes", NS_EML) => |elem| {
                let text = elem.text_without_children()?;
                let text = if !text.is_empty() {
                    Some(text)
                } else {
                    None
                };

                // If MaxVotes value is not present, default to "1"
                let text = text.unwrap_or_else(|| "1".to_string());

                StringValue::from_maybe_parsed_err(text, elem.strict_value_parsing(), ("MaxVotes", NS_KR), Some(elem.span()))?
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

/// Identifier for the contest.
#[derive(Debug, Clone)]
pub struct ContestIdentifier {
    /// Id of the contest.
    pub id: StringValue<ContestIdType>,
}

impl EMLElement for ContestIdentifier {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ContestIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let id_str = elem.attribute_value_req("Id")?;
        let id = StringValue::<ContestIdType>::from_maybe_parsed_err(
            id_str.into_owned(),
            elem.strict_value_parsing(),
            "Id",
            Some(elem.span()),
        )?;
        Ok(ContestIdentifier { id })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer.attr("Id", self.id.raw().as_ref())?.empty()
    }
}

/// A registered party in the election definition.
///
/// In election definitions this is just a party name, for full party details and
/// candidates see the [`CandidateList`](crate::documents::candidate_list::CandidateList)
/// document.
#[derive(Debug, Clone)]
pub struct ElectionDefinitionRegisteredParty {
    /// Name of the registered party (as registered at the CSB)
    pub registered_appellation: String,
}

impl ElectionDefinitionRegisteredParty {
    pub(crate) fn read_list(
        elem: &mut EMLElementReader<'_, '_>,
    ) -> Result<Vec<ElectionDefinitionRegisteredParty>, EMLError> {
        let mut parties = Vec::new();
        while let Some(mut child) = elem.next_child()? {
            if child.has_name(ElectionDefinitionRegisteredParty::EML_NAME)? {
                let party = ElectionDefinitionRegisteredParty::read_eml(&mut child)?;
                parties.push(party);
            } else {
                return Err(EMLErrorKind::UnexpectedElement(child.name()?.as_owned()))
                    .with_span(child.span());
            }
        }
        Ok(parties)
    }

    pub(crate) fn write_list(
        parties: &[ElectionDefinitionRegisteredParty],
        writer: EMLElementWriter,
    ) -> Result<(), EMLError> {
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
