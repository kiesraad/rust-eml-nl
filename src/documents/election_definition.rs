//! Document variant for the EML_NL Election Definition (`110a`) document.

use std::num::NonZeroU64;

use crate::{
    EML_SCHEMA_VERSION, EMLError, NS_EML, NS_KR,
    common::{
        CanonicalizationMethod, ContestIdentifier, CreationDateTime, ElectionDomain, ElectionTree,
        IssueDate, ManagingAuthority, TransactionId,
    },
    documents::accepted_root,
    error::{EMLErrorKind, EMLResultExt},
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName, collect_struct},
    utils::{
        ElectionCategory, ElectionIdType, ElectionSubcategory, StringValue, VotingMethod, XsDate,
    },
};

pub(crate) const EML_ELECTION_DEFINITION_ID: &str = "110a";

/// Representing a `110a` document, containing an election definition.
#[derive(Debug, Clone)]
pub struct ElectionDefinition {
    /// Transaction id of the document.
    pub transaction_id: TransactionId,
    /// Canonicalization method used in this document, if present.
    pub canonicalization_method: Option<CanonicalizationMethod>,
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
            canonicalization_method as Option: CanonicalizationMethod::EML_NAME => |elem| CanonicalizationMethod::read_eml(elem)?,
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
            // Note: we don't output the CanonicalizationMethod because we aren't canonicalizing our output
            // .child_elem_option(
            //     CanonicalizationMethod::EML_NAME,
            //     self.canonicalization_method.as_ref(),
            // )?
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
    /// Identifier for this election event.
    pub id: ElectionDefinitionElectionEventIdentifier,

    /// Election details.
    pub election: ElectionDefinitionElection,
}

impl EMLElement for ElectionDefinitionElectionEvent {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ElectionEvent", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, ElectionDefinitionElectionEvent {
            id: ElectionDefinitionElectionEventIdentifier::EML_NAME => |elem| ElectionDefinitionElectionEventIdentifier::read_eml(elem)?,
            election: ElectionDefinitionElection::EML_NAME => |elem| ElectionDefinitionElection::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(
                ElectionDefinitionElectionEventIdentifier::EML_NAME,
                &self.id,
            )?
            .child_elem(ElectionDefinitionElection::EML_NAME, &self.election)?
            .finish()
    }
}

/// Event identifier for an election event, is an empty element.
#[derive(Debug, Clone)]
pub struct ElectionDefinitionElectionEventIdentifier;

impl EMLElement for ElectionDefinitionElectionEventIdentifier {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("EventIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError>
    where
        Self: Sized,
    {
        elem.skip()?;
        Ok(ElectionDefinitionElectionEventIdentifier)
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer.empty()
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
        Ok(collect_struct!(elem, ElectionDefinitionElection {
            identifier: ElectionDefinitionElectionIdentifier::EML_NAME => |elem| ElectionDefinitionElectionIdentifier::read_eml(elem)?,
            contest: ElectionDefinitionContest::EML_NAME => |elem| ElectionDefinitionContest::read_eml(elem)?,
            number_of_seats: EML_NAME_NUMBER_OF_SEATS => |elem| elem.string_value()?,
            preference_threshold: EML_NAME_PREFERENCE_THRESHOLD => |elem| elem.string_value()?,
            election_tree: ElectionTree::EML_NAME => |elem| ElectionTree::read_eml(elem)?,
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
