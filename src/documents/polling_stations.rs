//! Document variant for the EML_NL Polling Stations (`110b`) document.

use std::num::NonZeroU64;

use crate::{
    EML_SCHEMA_VERSION, EMLError, NS_EML, NS_KR,
    common::{
        CanonicalizationMethod, ContestIdentifier, ContestIdentifierGeen, CreationDateTime,
        ElectionDomain, IssueDate, LocalityName, ManagingAuthority, PostalCode,
        ReportingUnitIdentifier, TransactionId,
    },
    documents::accepted_root,
    error::{EMLErrorKind, EMLResultExt},
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName, collect_struct},
    utils::{
        ElectionCategory, ElectionIdType, ElectionSubcategory, StringValue, VotingChannelType,
        VotingMethod, XsDate,
    },
};

pub(crate) const EML_POLLING_STATIONS_ID: &str = "110b";

/// Representing a `110b` document, containing polling stations.
#[derive(Debug, Clone)]
pub struct PollingStations {
    /// Transaction id of the document.
    pub transaction_id: TransactionId,

    /// Canonicalization method used in this document, if present.
    pub canonicalization_method: Option<CanonicalizationMethod>,

    /// Creation date and time of the document.
    pub creation_date_time: CreationDateTime,

    /// Issue date of the document.
    pub issue_date: Option<IssueDate>,

    /// Managing authority of the document.
    pub managing_authority: ManagingAuthority,

    /// Election event containing the polling stations.
    pub election_event: PollingStationsElectionEvent,
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
            canonicalization_method as Option: CanonicalizationMethod::EML_NAME => |elem| CanonicalizationMethod::read_eml(elem)?,
            creation_date_time: CreationDateTime::EML_NAME => |elem| CreationDateTime::read_eml(elem)?,
            issue_date as Option: IssueDate::EML_NAME => |elem| IssueDate::read_eml(elem)?,
            managing_authority: ManagingAuthority::EML_NAME => |elem| ManagingAuthority::read_eml(elem)?,
            election_event: PollingStationsElectionEvent::EML_NAME => |elem| PollingStationsElectionEvent::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr(("Id", None), EML_POLLING_STATIONS_ID)?
            .attr(("SchemaVersion", None), EML_SCHEMA_VERSION)?
            .child_elem(TransactionId::EML_NAME, &self.transaction_id)?
            // Note: we don't output the CanonicalizationMethod because we aren't canonicalizing our output
            // .child_elem_option(
            //     CanonicalizationMethod::EML_NAME,
            //     self.canonicalization_method.as_ref(),
            // )?
            .child_elem(CreationDateTime::EML_NAME, &self.creation_date_time)?
            .child_elem_option(IssueDate::EML_NAME, self.issue_date.as_ref())?
            .child_elem(ManagingAuthority::EML_NAME, &self.managing_authority)?
            .child_elem(PollingStationsElectionEvent::EML_NAME, &self.election_event)?
            .finish()?;

        Ok(())
    }
}

/// Election event containing polling stations.
#[derive(Debug, Clone)]
pub struct PollingStationsElectionEvent {
    /// Identifier for this election event.
    pub id: PollingStationsElectionEventIdentifier,

    /// Election details.
    pub election: PollingStationsElection,
}

impl EMLElement for PollingStationsElectionEvent {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ElectionEvent", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError>
    where
        Self: Sized,
    {
        Ok(collect_struct!(elem, PollingStationsElectionEvent {
            id: PollingStationsElectionEventIdentifier::EML_NAME => |elem| PollingStationsElectionEventIdentifier::read_eml(elem)?,
            election: PollingStationsElection::EML_NAME => |elem| PollingStationsElection::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(PollingStationsElectionEventIdentifier::EML_NAME, &self.id)?
            .child_elem(PollingStationsElection::EML_NAME, &self.election)?
            .finish()
    }
}

/// Identifier for a polling stations election event.
#[derive(Debug, Clone)]
pub struct PollingStationsElectionEventIdentifier;

impl EMLElement for PollingStationsElectionEventIdentifier {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("EventIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError>
    where
        Self: Sized,
    {
        elem.skip()?;
        Ok(PollingStationsElectionEventIdentifier)
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer.empty()
    }
}

/// Election definition containing polling stations.
#[derive(Debug, Clone)]
pub struct PollingStationsElection {
    /// Identifier of the election.
    pub identifier: PollingStationsElectionIdentifier,

    /// Contest containing the polling stations.
    pub contest: PollingStationsContest,
}

impl EMLElement for PollingStationsElection {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Election", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError>
    where
        Self: Sized,
    {
        Ok(collect_struct!(elem, PollingStationsElection {
            identifier: PollingStationsElectionIdentifier::EML_NAME => |elem| PollingStationsElectionIdentifier::read_eml(elem)?,
            contest: PollingStationsContest::EML_NAME => |elem| PollingStationsContest::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(
                PollingStationsElectionIdentifier::EML_NAME,
                &self.identifier,
            )?
            .child_elem(PollingStationsContest::EML_NAME, &self.contest)?
            .finish()
    }
}

/// Identifier of an election in the polling stations document.
#[derive(Debug, Clone)]
pub struct PollingStationsElectionIdentifier {
    /// Election id.
    pub id: StringValue<ElectionIdType>,

    /// Election name, if present.
    pub name: Option<String>,

    /// Election category.
    pub category: StringValue<ElectionCategory>,

    /// Election subcategory, if present.
    pub subcategory: Option<StringValue<ElectionSubcategory>>,

    /// The (top level) region where the election takes place.
    pub domain: Option<ElectionDomain>,

    /// Date of the election
    pub election_date: StringValue<XsDate>,
}

impl EMLElement for PollingStationsElectionIdentifier {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ElectionIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError>
    where
        Self: Sized,
    {
        Ok(collect_struct!(
            elem,
            PollingStationsElectionIdentifier {
                id: elem.string_value_attr("Id", None)?,
                name as Option: ("ElectionName", NS_EML) => |elem| elem.text_without_children()?,
                category: ("ElectionCategory", NS_EML) => |elem| elem.string_value()?,
                subcategory as Option: ("ElectionSubcategory", NS_KR) => |elem| elem.string_value()?,
                domain as Option: ElectionDomain::EML_NAME => |elem| ElectionDomain::read_eml(elem)?,
                election_date: ("ElectionDate", NS_KR) => |elem| elem.string_value()?,
            }
        ))
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
    pub max_votes: StringValue<NonZeroU64>,
    /// List of polling places in this contest.
    pub polling_places: Vec<PollingPlace>,
}

impl EMLElement for PollingStationsContest {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Contest", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, PollingStationsContest {
            identifier: ContestIdentifierGeen::EML_NAME => |elem| ContestIdentifierGeen::read_eml(elem)?,
            reporting_unit: PollingStationsReportingUnit::EML_NAME => |elem| PollingStationsReportingUnit::read_eml(elem)?,
            voting_method: ("VotingMethod", NS_EML) => |elem| elem.string_value()?,
            max_votes: ("MaxVotes", NS_EML) => |elem| {
                let text = elem.text_without_children_opt()?.unwrap_or_else(|| "1".to_string());
                elem.string_value_from_text(text, None, elem.full_span())?
            },
            polling_places as Vec: PollingPlace::EML_NAME => |elem| PollingPlace::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        let mut writer = writer
            .child_elem(ContestIdentifier::EML_NAME, &self.identifier)?
            .child_elem(PollingStationsReportingUnit::EML_NAME, &self.reporting_unit)?
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
            })?;

        for polling_place in &self.polling_places {
            writer = writer.child_elem(PollingPlace::EML_NAME, polling_place)?;
        }

        writer.finish()
    }
}

/// Reporting unit for the contest
#[derive(Debug, Clone)]
pub struct PollingStationsReportingUnit {
    /// Identifier of the reporting unit.
    pub identifier: ReportingUnitIdentifier,
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
    locality_name: LocalityName,
    postal_code: Option<PostalCode>,
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
pub struct PhysicalLocationPollingStation {}

impl EMLElement for PhysicalLocationPollingStation {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("PollingStation", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        // TODO
        elem.skip()?;
        Ok(PhysicalLocationPollingStation {})
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        // TODO
        writer.empty()
    }
}
