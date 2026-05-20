//! Document variant for the EML_NL Nomination (`210`) document.

use std::{fmt, str::FromStr};

use instant_xml::{FromXml, ToXml};
use thiserror::Error;

use crate::{
    EML_SCHEMA_VERSION, EMLError, EMLValueResultExt as _, NS_EML, NS_KR,
    common::{
        CandidateIdentifier, CanonicalizationMethod, CreationDateTime, ElectionDomain, IssueDate,
        ListData, ManagingAuthority, PersonNameStructure, TransactionId,
    },
    documents::ElectionIdentifierBuilder,
    eml_ns_context,
    error::EMLErrorKind,
    utils::{
        AffiliationType, ContestId, ElectionCategory, ElectionId, ElectionSubcategory, Gender,
        StringValue, StringValueData, XsDate, XsDateOrDateTime, XsDateTime,
    },
};

use super::candidate_lists::{
    QualifyingAddress, QualifyingAddressCountry, QualifyingAddressLocality,
};

/// EML document ID for nominations.
pub(crate) const EML_NOMINATION_ID: &str = "210";

/// Representing a `210` document, containing a nomination.
#[derive(Debug, Clone)]
pub struct Nomination {
    /// Transaction id of the document.
    pub transaction_id: TransactionId,

    /// Managing authority of the document, if present.
    pub managing_authority: Option<ManagingAuthority>,

    /// Issue date of the document.
    pub issue_date: IssueDate,

    /// Creation date and time of the document.
    pub creation_date_time: CreationDateTime,

    /// Canonicalization method used in this document, if present.
    pub canonicalization_method: Option<CanonicalizationMethod>,

    /// The nomination data contained in this document.
    pub nomination_data: NominationData,
}

impl Nomination {
    /// Create a new builder for the [`Nomination`] document.
    pub fn builder() -> NominationBuilder {
        NominationBuilder::new()
    }
}

impl FromStr for Nomination {
    type Err = EMLError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::io::EMLRead as _;
        Self::parse_eml(s, crate::io::EMLParsingMode::Strict).ok()
    }
}

impl TryFrom<&str> for Nomination {
    type Error = EMLError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use crate::io::EMLRead as _;
        Self::parse_eml(value, crate::io::EMLParsingMode::Strict).ok()
    }
}

impl TryFrom<Nomination> for String {
    type Error = EMLError;

    fn try_from(value: Nomination) -> Result<Self, Self::Error> {
        use crate::io::EMLWrite as _;
        value.write_eml_root_str(true)
    }
}

/// Builder for the [`Nomination`] document.
#[derive(Debug, Clone)]
pub struct NominationBuilder {
    transaction_id: Option<TransactionId>,
    managing_authority: Option<ManagingAuthority>,
    issue_date: Option<IssueDate>,
    creation_date_time: Option<CreationDateTime>,
    canonicalization_method: Option<CanonicalizationMethod>,
    nomination_data: Option<NominationData>,
    election_identifier: Option<NominationElectionIdentifier>,
    contest_identifier: Option<NominationContestIdentifier>,
    affiliation: Option<NominationAffiliation>,
    nominate: Option<NominationNominate>,
}

impl NominationBuilder {
    /// Create a new builder for the [`Nomination`] document.
    pub fn new() -> Self {
        NominationBuilder {
            transaction_id: None,
            managing_authority: None,
            issue_date: None,
            creation_date_time: None,
            canonicalization_method: None,
            nomination_data: None,
            election_identifier: None,
            contest_identifier: None,
            affiliation: None,
            nominate: None,
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

    /// Set the nomination data for the document directly.
    ///
    /// You may either set the entire nomination data at once using this
    /// method, or use any of [`Self::election_identifier`],
    /// [`Self::contest_identifier`], [`Self::affiliation`] and/or
    /// [`Self::nominate`] to construct the individual components.
    pub fn nomination_data(mut self, nomination_data: impl Into<NominationData>) -> Self {
        self.nomination_data = Some(nomination_data.into());
        self
    }

    /// Set the election identifier for the contained Nomination element.
    ///
    /// This only has effect if the nomination data was not set directly using
    /// [`Self::nomination_data`].
    pub fn election_identifier(
        mut self,
        election_identifier: impl Into<NominationElectionIdentifier>,
    ) -> Self {
        self.election_identifier = Some(election_identifier.into());
        self
    }

    /// Set the contest identifier for the contained Nomination element.
    ///
    /// This only has effect if the nomination data was not set directly using
    /// [`Self::nomination_data`].
    pub fn contest_identifier(
        mut self,
        contest_identifier: impl Into<NominationContestIdentifier>,
    ) -> Self {
        self.contest_identifier = Some(contest_identifier.into());
        self
    }

    /// Set the affiliation for the contained Nomination element.
    ///
    /// This only has effect if the nomination data was not set directly using
    /// [`Self::nomination_data`].
    pub fn affiliation(mut self, affiliation: impl Into<NominationAffiliation>) -> Self {
        self.affiliation = Some(affiliation.into());
        self
    }

    /// Set the nominate element for the contained Nomination element.
    ///
    /// This only has effect if the nomination data was not set directly using
    /// [`Self::nomination_data`].
    pub fn nominate(mut self, nominate: impl Into<NominationNominate>) -> Self {
        self.nominate = Some(nominate.into());
        self
    }

    /// Build the `Nomination` document, returning an error if any required fields are missing.
    pub fn build(self) -> Result<Nomination, EMLError> {
        Ok(Nomination {
            transaction_id: self
                .transaction_id
                .ok_or(EMLErrorKind::MissingBuildProperty("transaction_id").without_span())?,
            managing_authority: self.managing_authority,
            issue_date: self
                .issue_date
                .ok_or(EMLErrorKind::MissingBuildProperty("issue_date").without_span())?,
            creation_date_time: self
                .creation_date_time
                .ok_or(EMLErrorKind::MissingBuildProperty("creation_date_time").without_span())?,
            canonicalization_method: self.canonicalization_method,
            nomination_data: self.nomination_data.map_or_else(
                || {
                    Ok(NominationData {
                        election_identifier: self.election_identifier.ok_or(
                            EMLErrorKind::MissingBuildProperty("election_identifier")
                                .without_span(),
                        )?,
                        contest_identifier: self.contest_identifier.ok_or(
                            EMLErrorKind::MissingBuildProperty("contest_identifier").without_span(),
                        )?,
                        affiliation: self.affiliation.ok_or(
                            EMLErrorKind::MissingBuildProperty("affiliation").without_span(),
                        )?,
                        nominate: self
                            .nominate
                            .ok_or(EMLErrorKind::MissingBuildProperty("nominate").without_span())?,
                    })
                },
                Ok,
            )?,
        })
    }
}

impl Default for NominationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Custom: root EML element with Id/SchemaVersion attributes and full namespace context.
impl<'xml> FromXml<'xml> for Nomination {
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
        let mut nomination_data = <NominationData as FromXml>::Accumulator::default();

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
            } else if NominationData::matches(id, None) {
                let mut nested = deserializer.nested(element);
                NominationData::deserialize(&mut nomination_data, field, &mut nested)?;
                nested.ignore()?;
            } else {
                let mut nested = deserializer.nested(element);
                nested.ignore()?;
            }
        }

        *into = Some(Nomination {
            transaction_id: transaction_id.try_done(field)?,
            managing_authority,
            issue_date: issue_date.try_done(field)?,
            creation_date_time: creation_date_time.try_done(field)?,
            canonicalization_method,
            nomination_data: nomination_data.try_done(field)?,
        });
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Element;
}

// Custom: root EML element with Id/SchemaVersion attributes and full namespace context.
impl ToXml for Nomination {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix = serializer.write_start("EML", NS_EML, Some(eml_ns_context()))?;
        serializer.write_attr("Id", "", EML_NOMINATION_ID)?;
        serializer.write_attr("SchemaVersion", "", EML_SCHEMA_VERSION)?;
        serializer.end_start()?;

        self.transaction_id.serialize(None, serializer)?;
        if let Some(ma) = &self.managing_authority {
            ma.serialize(None, serializer)?;
        }

        self.issue_date.serialize(None, serializer)?;
        self.creation_date_time.serialize(None, serializer)?;
        self.nomination_data.serialize(None, serializer)?;

        serializer.write_close(prefix)
    }
}

/// The `<Nomination>` element containing election, contest, affiliation and proposer data.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "Nomination", ns(NS_EML))]
pub struct NominationData {
    /// The election identifier.
    #[xml(rename = "ElectionIdentifier")]
    pub election_identifier: NominationElectionIdentifier,

    /// The contest identifier.
    #[xml(rename = "ContestIdentifier")]
    pub contest_identifier: NominationContestIdentifier,

    /// The affiliation with its candidates.
    #[xml(rename = "Affiliation")]
    pub affiliation: NominationAffiliation,

    /// The proposers who nominate this list.
    #[xml(rename = "Nominate")]
    pub nominate: NominationNominate,
}

/// Identifier for the election in a nomination document.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "ElectionIdentifier", ns(NS_EML, kr = NS_KR))]
pub struct NominationElectionIdentifier {
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

    /// Nomination date for the election
    #[xml(rename = "NominationDate", ns(NS_KR))]
    pub nomination_date: StringValue<XsDate>,
}

impl NominationElectionIdentifier {
    /// Create a new Election Identifier builder
    pub fn builder() -> ElectionIdentifierBuilder {
        ElectionIdentifierBuilder::new()
    }
}

/// Contest identifier for a nomination document (with mandatory ContestName).
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "ContestIdentifier", ns(NS_EML))]
pub struct NominationContestIdentifier {
    /// Id of the contest.
    #[xml(attribute, rename = "Id")]
    pub id: StringValue<ContestId>,

    /// Name of the contest (mandatory in 210).
    #[xml(rename = "ContestName")]
    pub name: String,
}

impl NominationContestIdentifier {
    /// Create a new `NominationContestIdentifier`.
    pub fn new(id: ContestId, name: impl Into<String>) -> Self {
        NominationContestIdentifier {
            id: StringValue::Parsed(id),
            name: name.into(),
        }
    }
}

/// An affiliation in a nomination document.
///
/// In EML 210, the affiliation identifier has no `Id` attribute (it is prohibited),
/// and the `RegisteredName` is mandatory.
#[derive(Debug, Clone, FromXml)]
#[xml(rename = "Affiliation", ns(NS_EML))]
pub struct NominationAffiliation {
    /// The registered name of the affiliation (Id is prohibited in 210),
    /// wrapped in an `AffiliationIdentifier` element.
    #[xml(rename = "AffiliationIdentifier")]
    pub identifier: NominationAffiliationId,

    /// The affiliation type.
    #[xml(rename = "Type")]
    pub affiliation_type: StringValue<AffiliationType>,

    /// The list data of the affiliation.
    #[xml(rename = "ListData", ns(NS_KR))]
    pub list_data: ListData,

    /// The candidates of the affiliation.
    pub candidates: Vec<NominationCandidate>,
}

impl NominationAffiliation {
    /// Get the registered name.
    pub fn registered_name(&self) -> &str {
        &self.identifier.registered_name
    }
}

/// The affiliation identifier wrapper for nominations (no Id attribute, required RegisteredName).
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "AffiliationIdentifier", ns(NS_EML))]
pub struct NominationAffiliationId {
    /// The registered name.
    #[xml(rename = "RegisteredName")]
    pub registered_name: String,
}

// Custom: wraps identifier fields in `<AffiliationIdentifier>/<RegisteredName>` sub-elements.
impl ToXml for NominationAffiliation {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix =
            serializer.write_start("Affiliation", NS_EML, None::<instant_xml::ser::Context<0>>)?;
        serializer.end_start()?;
        self.identifier.serialize(None, serializer)?;

        self.affiliation_type.serialize(
            Some(instant_xml::Id {
                ns: NS_EML,
                name: "Type",
            }),
            serializer,
        )?;

        self.list_data.serialize(None, serializer)?;
        for candidate in &self.candidates {
            candidate.serialize(None, serializer)?;
        }

        serializer.write_close(prefix)
    }
}

/// A candidate in a nomination document.
///
/// In EML 210, `Gender` and `QualifyingAddress` are required (unlike in 230b/230c
/// where they are optional). Additional fields like `Contact`, `Agent`,
/// `DateOfBirthAnnex` and `NationalIdentificationNumber` are also supported.
#[derive(Debug, Clone, FromXml)]
#[xml(rename = "Candidate", ns(NS_EML, kr = NS_KR))]
pub struct NominationCandidate {
    /// The candidate identifier.
    #[xml(rename = "CandidateIdentifier")]
    pub identifier: CandidateIdentifier,

    /// The full name of the candidate.
    #[xml(rename = "CandidateFullName")]
    pub full_name: PersonNameStructure,

    /// The date of birth of the candidate, if present.
    #[xml(rename = "DateOfBirth")]
    pub date_of_birth: Option<StringValue<XsDate>>,

    /// The gender of the candidate (required in 210).
    #[xml(rename = "Gender")]
    pub gender: StringValue<Gender>,

    /// The qualifying address of the candidate (required in 210).
    #[xml(rename = "QualifyingAddress")]
    pub qualifying_address: QualifyingAddress,

    /// Contact details for the candidate, if present.
    #[xml(rename = "Contact")]
    pub contact: Option<NominationContact>,

    /// Agent details for the candidate, if present.
    #[xml(rename = "Agent")]
    pub agent: Option<NominationAgent>,

    /// Alternative date of birth representation when exact date is unknown.
    #[xml(rename = "DateOfBirthAnnex", ns(NS_KR))]
    pub date_of_birth_annex: Option<String>,

    /// National identification number (e.g. BSN in the Netherlands).
    #[xml(rename = "NationalIdentificationNumber", ns(NS_KR))]
    pub national_identification_number: Option<String>,
}

// Custom: wraps full_name in `<CandidateFullName>` wrapper; inline kr: elements for annex/NIN.
impl ToXml for NominationCandidate {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix =
            serializer.write_start("Candidate", NS_EML, None::<instant_xml::ser::Context<0>>)?;
        serializer.end_start()?;

        self.identifier.serialize(None, serializer)?;
        let cfn_prefix = serializer.write_start(
            "CandidateFullName",
            NS_EML,
            None::<instant_xml::ser::Context<0>>,
        )?;

        self.full_name.serialize(None, serializer)?;
        serializer.write_close(cfn_prefix)?;
        if let Some(dob) = &self.date_of_birth {
            dob.serialize(
                Some(instant_xml::Id {
                    ns: NS_EML,
                    name: "DateOfBirth",
                }),
                serializer,
            )?;
        }

        self.gender.serialize(
            Some(instant_xml::Id {
                ns: NS_EML,
                name: "Gender",
            }),
            serializer,
        )?;

        self.qualifying_address.serialize(None, serializer)?;

        if let Some(contact) = &self.contact {
            contact.serialize(None, serializer)?;
        }
        if let Some(agent) = &self.agent {
            agent.serialize(None, serializer)?;
        }

        if let Some(dob_annex) = &self.date_of_birth_annex {
            let dba_prefix = serializer.write_start(
                "DateOfBirthAnnex",
                NS_KR,
                None::<instant_xml::ser::Context<0>>,
            )?;
            serializer.end_start()?;
            serializer.write_str(dob_annex.as_str())?;
            serializer.write_close(dba_prefix)?;
        }

        if let Some(nin) = &self.national_identification_number {
            let nin_prefix = serializer.write_start(
                "NationalIdentificationNumber",
                NS_KR,
                None::<instant_xml::ser::Context<0>>,
            )?;
            serializer.end_start()?;
            serializer.write_str(nin.as_str())?;
            serializer.write_close(nin_prefix)?;
        }

        serializer.write_close(prefix)
    }
}

/// Contact details (containing a mailing address).
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "Contact", ns(NS_EML))]
pub struct NominationContact {
    /// The mailing address.
    #[xml(rename = "MailingAddress")]
    pub mailing_address: MailingAddress,
}

/// A mailing address, structured as a qualifying address (Locality or Country).
#[derive(Debug, Clone)]
pub struct MailingAddress {
    /// The address content (Locality or Country).
    pub address: QualifyingAddress,
}

impl MailingAddress {
    /// Create a new mailing address with a locality.
    pub fn new(address: impl Into<QualifyingAddress>) -> Self {
        MailingAddress {
            address: address.into(),
        }
    }
}

// Custom: enum dispatch (Locality/Country variants) inside a `<MailingAddress>` element.
impl<'xml> FromXml<'xml> for MailingAddress {
    fn matches(id: instant_xml::Id<'_>, field: Option<instant_xml::Id<'_>>) -> bool {
        match field {
            Some(field) => id == field,
            None => {
                id == instant_xml::Id {
                    ns: NS_EML,
                    name: "MailingAddress",
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

        while let Some(node) = deserializer.next() {
            let element = match node? {
                Node::Open(element) => element,
                Node::Text(s) if s.trim().is_empty() => continue,
                node => return Err(Error::UnexpectedNode(format!("{node:?}"))),
            };

            let id = deserializer.element_id(&element)?;
            if QualifyingAddressLocality::matches(id, None) {
                let mut acc = <QualifyingAddressLocality as FromXml<'xml>>::Accumulator::default();
                let mut nested = deserializer.nested(element);
                QualifyingAddressLocality::deserialize(&mut acc, field, &mut nested)?;
                nested.ignore()?;
                *into = Some(MailingAddress {
                    address: QualifyingAddress::Locality(acc.try_done(field)?),
                });
            } else if QualifyingAddressCountry::matches(id, None) {
                let mut acc = <QualifyingAddressCountry as FromXml<'xml>>::Accumulator::default();
                let mut nested = deserializer.nested(element);
                QualifyingAddressCountry::deserialize(&mut acc, field, &mut nested)?;
                nested.ignore()?;
                *into = Some(MailingAddress {
                    address: QualifyingAddress::Country(acc.try_done(field)?),
                });
            } else {
                let mut nested = deserializer.nested(element);
                nested.ignore()?;
            }
        }

        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Element;
}

// Custom: enum dispatch (QualifyingAddress variants) inside a `<MailingAddress>` element.
impl ToXml for MailingAddress {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix = serializer.write_start(
            "MailingAddress",
            NS_EML,
            None::<instant_xml::ser::Context<0>>,
        )?;
        serializer.end_start()?;

        match &self.address {
            QualifyingAddress::Locality(inner) => inner.serialize(None, serializer)?,
            QualifyingAddress::Country(inner) => inner.serialize(None, serializer)?,
        }

        serializer.write_close(prefix)
    }
}

/// An agent for a candidate.
#[derive(Debug, Clone, FromXml)]
#[xml(rename = "Agent", ns(NS_EML))]
pub struct NominationAgent {
    /// The role of the agent (e.g. "H10" or "H10a").
    #[xml(attribute, rename = "Role")]
    pub role: Option<String>,

    /// The agent's name.
    #[xml(rename = "AgentIdentifier")]
    pub agent_identifier: AgentIdentifier,

    /// Contact details for the agent, if present.
    #[xml(rename = "Contact")]
    pub contact: Option<NominationContact>,

    /// The living address of the agent.
    #[xml(rename = "LivingAddress")]
    pub living_address: LivingAddress,
}

// Custom: optional Role attribute written conditionally before `end_start`.
impl ToXml for NominationAgent {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix =
            serializer.write_start("Agent", NS_EML, None::<instant_xml::ser::Context<0>>)?;
        if let Some(role) = &self.role {
            serializer.write_attr("Role", "", role.as_str())?;
        }
        serializer.end_start()?;

        self.agent_identifier.serialize(None, serializer)?;
        if let Some(contact) = &self.contact {
            contact.serialize(None, serializer)?;
        }
        self.living_address.serialize(None, serializer)?;

        serializer.write_close(prefix)
    }
}

/// Job title used for a proposer in a nomination document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromXml, ToXml)]
#[xml(scalar)]
pub enum NominationJobTitle {
    /// inleveraar
    #[xml(rename = "inleveraar")]
    Submitter,
    /// plaatsvervanger van de inleveraar
    #[xml(rename = "plaatsvervanger van de inleveraar")]
    DeputySubmitter,
    /// gemachtigde voor het aangaan van lijstencombinaties
    #[xml(rename = "gemachtigde voor het aangaan van lijstencombinaties")]
    CombinationRepresentative,
    /// plaatsvervanger voor het aangaan van lijstencombinaties
    #[xml(rename = "plaatsvervanger voor het aangaan van lijstencombinaties")]
    DeputyCombinationRepresentative,
}

impl NominationJobTitle {
    /// Create a new NominationJobTitle from a string, validating its format.
    pub fn new(s: impl AsRef<str>) -> Result<Self, EMLError> {
        Self::from_eml_value(s).wrap_value_error()
    }

    /// Create a [`NominationJobTitle`] from a `&str`, if possible.
    pub fn from_eml_value(s: impl AsRef<str>) -> Result<Self, UnknownNominationJobTitleError> {
        let data = s.as_ref();
        match data {
            "inleveraar" => Ok(NominationJobTitle::Submitter),
            "plaatsvervanger van de inleveraar" => Ok(NominationJobTitle::DeputySubmitter),
            "gemachtigde voor het aangaan van lijstencombinaties" => {
                Ok(NominationJobTitle::CombinationRepresentative)
            }
            "plaatsvervanger voor het aangaan van lijstencombinaties" => {
                Ok(NominationJobTitle::DeputyCombinationRepresentative)
            }
            _ => Err(UnknownNominationJobTitleError(data.to_string())),
        }
    }

    /// Get the `&str` representation of this [`NominationJobTitle`].
    pub fn to_eml_value(&self) -> &'static str {
        match self {
            NominationJobTitle::Submitter => "inleveraar",
            NominationJobTitle::DeputySubmitter => "plaatsvervanger van de inleveraar",
            NominationJobTitle::CombinationRepresentative => {
                "gemachtigde voor het aangaan van lijstencombinaties"
            }
            NominationJobTitle::DeputyCombinationRepresentative => {
                "plaatsvervanger voor het aangaan van lijstencombinaties"
            }
        }
    }
}

/// Error returned when an unknown nomination job title string is encountered.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("Unknown nomination job title: {0}")]
pub struct UnknownNominationJobTitleError(String);

impl StringValueData for NominationJobTitle {
    type Error = UnknownNominationJobTitleError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Self::from_eml_value(s)
    }

    fn to_raw_value(&self) -> String {
        self.to_eml_value().to_string()
    }
}

/// Agent identifier containing the agent's name.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "AgentIdentifier", ns(NS_EML))]
pub struct AgentIdentifier {
    /// The agent's name.
    #[xml(rename = "AgentName")]
    pub agent_name: PersonNameStructure,
}

impl AgentIdentifier {
    /// Create a new `AgentIdentifier`.
    pub fn new(agent_name: impl Into<PersonNameStructure>) -> Self {
        AgentIdentifier {
            agent_name: agent_name.into(),
        }
    }
}

/// A living address (kr:LivingAddress).
#[derive(Debug, Clone, FromXml)]
#[xml(rename = "LivingAddress", ns(NS_KR), force_prefix)]
pub struct LivingAddress {
    /// The locality name.
    #[xml(rename = "LocalityName", ns(NS_KR))]
    pub locality_name: String,

    /// The country name code, if present.
    #[xml(rename = "CountryNameCode", ns(NS_KR))]
    pub country_name_code: Option<String>,
}

// Custom: uses `display_to_xml` for scalar children in the kr: namespace with force_prefix.
impl ToXml for LivingAddress {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let mut cx = instant_xml::ser::Context::<0>::default();
        cx.default_ns = NS_KR;
        cx.force_prefix = true;

        let prefix = serializer.write_start("LivingAddress", NS_KR, Some(cx))?;
        serializer.end_start()?;

        instant_xml::display_to_xml(
            &self.locality_name,
            Some(instant_xml::Id {
                ns: NS_KR,
                name: "LocalityName",
            }),
            serializer,
        )?;

        if let Some(code) = &self.country_name_code {
            instant_xml::display_to_xml(
                code,
                Some(instant_xml::Id {
                    ns: NS_KR,
                    name: "CountryNameCode",
                }),
                serializer,
            )?;
        }

        serializer.write_close(prefix)
    }
}

impl LivingAddress {
    /// Create a new `LivingAddress`.
    pub fn new(locality_name: impl Into<String>) -> Self {
        LivingAddress {
            locality_name: locality_name.into(),
            country_name_code: None,
        }
    }

    /// Set the country name code.
    pub fn with_country_name_code(mut self, code: impl Into<String>) -> Self {
        self.country_name_code = Some(code.into());
        self
    }
}

/// The `<Nominate>` element containing proposers.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "Nominate", ns(NS_EML))]
pub struct NominationNominate {
    /// The proposers (minimum 2 required by schema).
    #[xml(rename = "Proposer")]
    pub proposers: Vec<NominationProposer>,
}

impl NominationNominate {
    /// Create a new `NominationNominate` with the given proposers.
    pub fn new(proposers: Vec<NominationProposer>) -> Self {
        NominationNominate { proposers }
    }
}

/// A proposer in a nomination document.
#[derive(Debug, Clone, FromXml)]
#[xml(rename = "Proposer", ns(NS_EML))]
pub struct NominationProposer {
    /// The proposer's name.
    #[xml(rename = "Name")]
    pub name: PersonNameStructure,

    /// Contact details for the proposer (required).
    #[xml(rename = "Contact")]
    pub contact: NominationContact,

    /// The job title of the proposer.
    ///
    /// Valid values: "inleveraar", "plaatsvervanger van de inleveraar",
    /// "gemachtigde voor het aangaan van lijstencombinaties",
    /// "plaatsvervanger voor het aangaan van lijstencombinaties"
    #[xml(rename = "JobTitle")]
    pub job_title: StringValue<NominationJobTitle>,

    /// Optional identifier for the proposer (mandatory if deputy).
    #[xml(rename = "Id")]
    pub id: Option<String>,

    /// The living address of the proposer, if present.
    #[xml(rename = "LivingAddress")]
    pub living_address: Option<LivingAddress>,
}

// Custom: wraps name in `<Name>` wrapper; inline `<Id>` element for optional proposer id.
impl ToXml for NominationProposer {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix =
            serializer.write_start("Proposer", NS_EML, None::<instant_xml::ser::Context<0>>)?;
        serializer.end_start()?;

        let name_prefix =
            serializer.write_start("Name", NS_EML, None::<instant_xml::ser::Context<0>>)?;
        self.name.serialize(None, serializer)?;
        serializer.write_close(name_prefix)?;
        self.contact.serialize(None, serializer)?;
        self.job_title.serialize(
            Some(instant_xml::Id {
                ns: NS_EML,
                name: "JobTitle",
            }),
            serializer,
        )?;

        if let Some(id) = &self.id {
            let id_prefix =
                serializer.write_start("Id", NS_EML, None::<instant_xml::ser::Context<0>>)?;
            serializer.end_start()?;
            serializer.write_str(id.as_str())?;
            serializer.write_close(id_prefix)?;
        }

        if let Some(la) = &self.living_address {
            la.serialize(None, serializer)?;
        }

        serializer.write_close(prefix)
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use chrono::{NaiveDate, NaiveDateTime};

    use super::*;
    use crate::{
        common::{AuthorityIdentifier, CandidateIdentifier, ElectionDomain, ListData, PersonName},
        documents::EML,
        io::{EMLParsingMode, EMLRead as _, EMLWrite as _},
        utils::{
            AffiliationType, AuthorityId, CandidateId, ContestId, ElectionCategory,
            ElectionDomainId, ElectionId, ElectionSubcategory, Gender, StringValue, XsDate,
            XsDateTime,
        },
    };

    #[test]
    fn nomination_construction() {
        let nomination = Nomination::builder()
            .transaction_id(TransactionId::new(1))
            .managing_authority(ManagingAuthority::new(
                AuthorityIdentifier::new(AuthorityId::new("0000").unwrap()).with_name("Test"),
            ))
            .issue_date(XsDate::from_date(2024, 6, 10).unwrap())
            .creation_date_time(XsDateTime::new_without_tz(NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2024, 6, 10).unwrap(),
                chrono::NaiveTime::from_hms_milli_opt(12, 0, 0, 0).unwrap(),
            )))
            .election_identifier(
                NominationElectionIdentifier::builder()
                    .id(ElectionId::new("GR2026_Test").unwrap())
                    .category(ElectionCategory::GR)
                    .subcategory(ElectionSubcategory::GR2)
                    .domain(ElectionDomain::new(
                        Some(ElectionDomainId::new("0000").unwrap()),
                        "Test",
                    ))
                    .election_date(XsDate::from_date(2026, 3, 18).unwrap())
                    .nomination_date(XsDate::from_date(2026, 2, 2).unwrap())
                    .build_for_nomination()
                    .unwrap(),
            )
            .contest_identifier(NominationContestIdentifier::new(
                ContestId::new("geen").unwrap(),
                "Test Contest",
            ))
            .affiliation(NominationAffiliation {
                identifier: NominationAffiliationId {
                    registered_name: "Test Party".to_string(),
                },
                affiliation_type: StringValue::from_value(AffiliationType::StandAloneList),
                list_data: ListData::new(true),
                candidates: vec![
                    NominationCandidate {
                        identifier: CandidateIdentifier::new(CandidateId::new(
                            NonZeroU64::new(1).unwrap(),
                        )),
                        full_name: PersonName::new("Tansen")
                            .with_initials("J.")
                            .with_first_name("Jan")
                            .with_name_prefix("van")
                            .into(),
                        date_of_birth: Some(StringValue::from_value(
                            XsDate::from_date(1980, 1, 15).unwrap(),
                        )),
                        gender: StringValue::from_value(Gender::Male),
                        qualifying_address: QualifyingAddress::Locality(
                            QualifyingAddressLocality::new("Amsterdam"),
                        ),
                        contact: None,
                        agent: None,
                        date_of_birth_annex: None,
                        national_identification_number: None,
                    },
                    NominationCandidate {
                        identifier: CandidateIdentifier::new(CandidateId::new(
                            NonZeroU64::new(2).unwrap(),
                        )),
                        full_name: PersonName::new("Bakker")
                            .with_initials("A.B.")
                            .with_first_name("Anna")
                            .into(),
                        date_of_birth: Some(StringValue::from_value(
                            XsDate::from_date(1990, 7, 22).unwrap(),
                        )),
                        gender: StringValue::from_value(Gender::Female),
                        qualifying_address: QualifyingAddress::Country(
                            QualifyingAddressCountry::new(Some("NL"), "Rotterdam"),
                        ),
                        contact: Some(NominationContact {
                            mailing_address: MailingAddress::new(QualifyingAddress::Locality(
                                QualifyingAddressLocality::new("Rotterdam"),
                            )),
                        }),
                        agent: Some(NominationAgent {
                            role: Some("H10".to_string()),
                            agent_identifier: AgentIdentifier::new(
                                PersonName::new("Groot")
                                    .with_initials("P.")
                                    .with_first_name("Pieter"),
                            ),
                            contact: None,
                            living_address: LivingAddress::new("Den Haag"),
                        }),
                        date_of_birth_annex: Some("XX-07-1990".to_string()),
                        national_identification_number: Some("123456789".to_string()),
                    },
                ],
            })
            .nominate(NominationNominate::new(vec![
                NominationProposer {
                    name: PersonName::new("Janssen")
                        .with_initials("K.")
                        .with_first_name("Karel")
                        .into(),
                    contact: NominationContact {
                        mailing_address: MailingAddress::new(QualifyingAddress::Locality(
                            QualifyingAddressLocality::new("Amsterdam"),
                        )),
                    },
                    job_title: StringValue::from_value(NominationJobTitle::Submitter),
                    id: None,
                    living_address: None,
                },
                NominationProposer {
                    name: PersonName::new("Vries")
                        .with_initials("M.")
                        .with_first_name("Maria")
                        .with_name_prefix("de")
                        .into(),
                    contact: NominationContact {
                        mailing_address: MailingAddress::new(QualifyingAddress::Locality(
                            QualifyingAddressLocality::new("Utrecht"),
                        )),
                    },
                    job_title: StringValue::from_value(NominationJobTitle::DeputySubmitter),
                    id: Some("PV001".to_string()),
                    living_address: Some(
                        LivingAddress::new("Utrecht").with_country_name_code("NL"),
                    ),
                },
            ]))
            .build()
            .unwrap();

        let xml = nomination.write_eml_root_str(true).unwrap();
        eprintln!("DEBUG XML:\n{}", &xml);

        let eml = EML::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        let parsed = eml
            .as_nomination_doc()
            .expect("expected nomination variant");
        let xml2 = parsed.write_eml_root_str(true).unwrap();
        assert_eq!(xml, xml2);
    }

    #[test]
    fn test_nomination_parse_and_write_roundtrip() {
        let doc = include_str!("../../test-emls/nomination/eml210_test.eml.xml");
        let eml = EML::parse_eml(doc, EMLParsingMode::Strict)
            .ok()
            .expect("Failed to parse EML 210 document");
        let nomination = eml
            .as_nomination_doc()
            .expect("expected nomination variant");

        assert_eq!(nomination.transaction_id.raw(), "1");
        assert!(nomination.managing_authority.is_some());
        assert_eq!(
            nomination.nomination_data.contest_identifier.name,
            "Test Contest"
        );
        assert_eq!(
            nomination.nomination_data.affiliation.registered_name(),
            "Test Party"
        );
        assert!(!nomination.nomination_data.affiliation.candidates.is_empty());
        assert!(nomination.nomination_data.nominate.proposers.len() >= 2);

        let xml_output = nomination
            .write_eml_root_str(true)
            .expect("Failed to write EML 210 document");
        let eml2 = EML::parse_eml(&xml_output, EMLParsingMode::Strict)
            .ok()
            .expect("Failed to re-parse written EML 210 document");
        let reparsed = eml2
            .as_nomination_doc()
            .expect("expected nomination variant");

        assert_eq!(
            reparsed.nomination_data.affiliation.registered_name(),
            nomination.nomination_data.affiliation.registered_name()
        );
        assert_eq!(
            reparsed.nomination_data.affiliation.candidates.len(),
            nomination.nomination_data.affiliation.candidates.len()
        );
        assert_eq!(
            reparsed.nomination_data.nominate.proposers.len(),
            nomination.nomination_data.nominate.proposers.len()
        );
    }

    #[test]
    fn test_nomination_job_title_from_str() {
        assert_eq!(
            NominationJobTitle::from_eml_value("inleveraar"),
            Ok(NominationJobTitle::Submitter)
        );
        assert_eq!(
            NominationJobTitle::from_eml_value("plaatsvervanger van de inleveraar"),
            Ok(NominationJobTitle::DeputySubmitter)
        );
        assert_eq!(
            NominationJobTitle::from_eml_value(
                "gemachtigde voor het aangaan van lijstencombinaties"
            ),
            Ok(NominationJobTitle::CombinationRepresentative)
        );
        assert_eq!(
            NominationJobTitle::from_eml_value(
                "plaatsvervanger voor het aangaan van lijstencombinaties"
            ),
            Ok(NominationJobTitle::DeputyCombinationRepresentative)
        );
        assert_eq!(
            NominationJobTitle::from_eml_value("UNKNOWN"),
            Err(UnknownNominationJobTitleError("UNKNOWN".to_string()))
        );
    }

    #[test]
    fn test_nomination_job_title_to_str() {
        assert_eq!(NominationJobTitle::Submitter.to_eml_value(), "inleveraar");
        assert_eq!(
            NominationJobTitle::DeputySubmitter.to_eml_value(),
            "plaatsvervanger van de inleveraar"
        );
        assert_eq!(
            NominationJobTitle::CombinationRepresentative.to_eml_value(),
            "gemachtigde voor het aangaan van lijstencombinaties"
        );
        assert_eq!(
            NominationJobTitle::DeputyCombinationRepresentative.to_eml_value(),
            "plaatsvervanger voor het aangaan van lijstencombinaties"
        );
    }
}
