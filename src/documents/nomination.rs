//! Document variant for the EML_NL Nomination (`210`) document.

use std::{borrow::Cow, str::FromStr};

use thiserror::Error;

use crate::{
    EML_SCHEMA_VERSION, EMLError, EMLValueResultExt as _, NS_EML, NS_KR,
    common::{
        CandidateIdentifier, CanonicalizationMethod, CreationDateTime, ElectionDomain, IssueDate,
        ListData, ManagingAuthority, PersonNameStructure, TransactionId,
    },
    documents::{
        ElectionIdentifierBuilder, accepted_root, validate_category_and_subcategory,
        validate_election_and_nomination_dates,
    },
    error::EMLErrorKind,
    io::{
        EMLElement, EMLElementReader, EMLElementWriter, EMLReadElement as _, QualifiedName,
        collect_struct, write_eml_element,
    },
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
        value.write_eml_root_str(true, true)
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

impl EMLElement for Nomination {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("EML", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        accepted_root(elem)?;

        let document_id = elem.attribute_value_req(("Id", None))?;
        if document_id.as_ref() != EML_NOMINATION_ID {
            return Err(
                EMLErrorKind::InvalidDocumentType("210", document_id.to_string())
                    .with_span(elem.span()),
            );
        }

        Ok(collect_struct!(elem, Nomination {
            transaction_id: TransactionId::EML_NAME => |elem| TransactionId::read_eml(elem)?,
            managing_authority as Option: ManagingAuthority::EML_NAME => |elem| ManagingAuthority::read_eml(elem)?,
            issue_date: IssueDate::EML_NAME => |elem| IssueDate::read_eml(elem)?,
            creation_date_time: CreationDateTime::EML_NAME => |elem| CreationDateTime::read_eml(elem)?,
            canonicalization_method as Option: CanonicalizationMethod::EML_NAME => |elem| CanonicalizationMethod::read_eml(elem)?,
            nomination_data: NominationData::EML_NAME => |elem| NominationData::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr(("Id", None), EML_NOMINATION_ID)?
            .attr(("SchemaVersion", None), EML_SCHEMA_VERSION)?
            .child_elem(TransactionId::EML_NAME, &self.transaction_id)?
            .child_elem_option(
                ManagingAuthority::EML_NAME,
                self.managing_authority.as_ref(),
            )?
            .child_elem(IssueDate::EML_NAME, &self.issue_date)?
            .child_elem(CreationDateTime::EML_NAME, &self.creation_date_time)?
            // Note: we don't output the CanonicalizationMethod because we aren't canonicalizing our output
            // .child_elem_option(
            //     CanonicalizationMethod::EML_NAME,
            //     self.canonicalization_method.as_ref(),
            // )?
            .child_elem(NominationData::EML_NAME, &self.nomination_data)?
            .finish()
    }
}

/// The `<Nomination>` element containing election, contest, affiliation and proposer data.
#[derive(Debug, Clone)]
pub struct NominationData {
    /// The election identifier.
    pub election_identifier: NominationElectionIdentifier,

    /// The contest identifier.
    pub contest_identifier: NominationContestIdentifier,

    /// The affiliation with its candidates.
    pub affiliation: NominationAffiliation,

    /// The proposers who nominate this list.
    pub nominate: NominationNominate,
}

impl EMLElement for NominationData {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Nomination", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, NominationData {
            election_identifier: NominationElectionIdentifier::EML_NAME => |elem| NominationElectionIdentifier::read_eml(elem)?,
            contest_identifier: NominationContestIdentifier::EML_NAME => |elem| NominationContestIdentifier::read_eml(elem)?,
            affiliation: NominationAffiliation::EML_NAME => |elem| NominationAffiliation::read_eml(elem)?,
            nominate: NominationNominate::EML_NAME => |elem| NominationNominate::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(
                NominationElectionIdentifier::EML_NAME,
                &self.election_identifier,
            )?
            .child_elem(
                NominationContestIdentifier::EML_NAME,
                &self.contest_identifier,
            )?
            .child_elem(NominationAffiliation::EML_NAME, &self.affiliation)?
            .child_elem(NominationNominate::EML_NAME, &self.nominate)?
            .finish()
    }
}

/// Identifier for the election in a nomination document.
#[derive(Debug, Clone)]
pub struct NominationElectionIdentifier {
    /// Id of the election
    pub id: StringValue<ElectionId>,

    /// Name of the election
    pub name: Option<Box<str>>,

    /// Category of the election
    pub category: StringValue<ElectionCategory>,

    /// Subcategory of the election
    pub subcategory: Option<StringValue<ElectionSubcategory>>,

    /// The (top level) region where the election takes place.
    pub domain: Option<ElectionDomain>,

    /// Date of the election
    pub election_date: StringValue<XsDate>,

    /// Nomination date for the election
    pub nomination_date: StringValue<XsDate>,
}

impl NominationElectionIdentifier {
    /// Create a new Election Identifier builder
    pub fn builder() -> ElectionIdentifierBuilder {
        ElectionIdentifierBuilder::new()
    }
}

impl EMLElement for NominationElectionIdentifier {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ElectionIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let data = collect_struct!(
            elem,
            NominationElectionIdentifier {
                id: elem.string_value_attr("Id", None)?,
                name as Option: ("ElectionName", NS_EML) => |elem| elem.text_without_children()?,
                category: ("ElectionCategory", NS_EML) => |elem| elem.string_value()?,
                subcategory as Option: ("ElectionSubcategory", NS_KR) => |elem| elem.string_value()?,
                domain as Option: ElectionDomain::EML_NAME => |elem| ElectionDomain::read_eml(elem)?,
                election_date: ("ElectionDate", NS_KR) => |elem| elem.string_value()?,
                nomination_date: ("NominationDate", NS_KR) => |elem| elem.string_value()?,
            }
        );

        if let Err(e) = validate_election_and_nomination_dates(
            Some(&data.election_date),
            Some(&data.nomination_date),
        ) {
            let e = e.into_kind().with_span(elem.full_span());
            if elem.parsing_mode().is_strict() {
                return Err(e);
            } else {
                elem.push_err(e);
            }
        }

        if let Err(e) = validate_category_and_subcategory(&data.category, data.subcategory.as_ref())
        {
            let e = e.into_kind().with_span(elem.full_span());
            if elem.parsing_mode().is_strict() {
                return Err(e);
            } else {
                elem.push_err(e);
            }
        }

        Ok(data)
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
            .child(("NominationDate", NS_KR), |elem| {
                elem.text(self.nomination_date.raw().as_ref())?.finish()
            })?
            .finish()
    }
}

/// Contest identifier for a nomination document (with mandatory ContestName).
#[derive(Debug, Clone)]
pub struct NominationContestIdentifier {
    /// Id of the contest.
    pub id: StringValue<ContestId>,

    /// Name of the contest (mandatory in 210).
    pub name: Box<str>,
}

impl NominationContestIdentifier {
    /// Create a new `NominationContestIdentifier`.
    pub fn new(id: ContestId, name: impl Into<Box<str>>) -> Self {
        NominationContestIdentifier {
            id: StringValue::Parsed(id),
            name: name.into(),
        }
    }
}

impl EMLElement for NominationContestIdentifier {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ContestIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(
            elem,
            NominationContestIdentifier {
                id: elem.string_value_attr("Id", None)?,
                name: ("ContestName", NS_EML) => |elem| elem.text_without_children()?,
            }
        ))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr("Id", self.id.raw().as_ref())?
            .child(("ContestName", NS_EML), |elem| {
                elem.text(self.name.as_ref())?.finish()
            })?
            .finish()
    }
}

/// An affiliation in a nomination document.
///
/// In EML 210, the affiliation identifier has no `Id` attribute (it is prohibited),
/// and the `RegisteredName` is mandatory.
#[derive(Debug, Clone)]
pub struct NominationAffiliation {
    /// The registered name of the affiliation (Id is prohibited in 210).
    pub registered_name: Box<str>,

    /// The affiliation type.
    pub affiliation_type: StringValue<AffiliationType>,

    /// The list data of the affiliation.
    pub list_data: ListData,

    /// The candidates of the affiliation.
    pub candidates: Vec<NominationCandidate>,
}

impl EMLElement for NominationAffiliation {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Affiliation", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        // The AffiliationIdentifier in 210 has no Id attribute and a required RegisteredName
        struct NominationAffiliationIdentifier {
            registered_name: Box<str>,
        }

        impl EMLElement for NominationAffiliationIdentifier {
            const EML_NAME: QualifiedName<'_, '_> =
                QualifiedName::from_static("AffiliationIdentifier", Some(NS_EML));

            fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
                Ok(collect_struct!(
                    elem,
                    NominationAffiliationIdentifier {
                        registered_name: ("RegisteredName", NS_EML) => |elem| elem.text_without_children()?,
                    }
                ))
            }

            fn write_eml(&self, _writer: EMLElementWriter) -> Result<(), EMLError> {
                unreachable!()
            }
        }

        let data = collect_struct!(elem, NominationAffiliation {
            registered_name: NominationAffiliationIdentifier::EML_NAME => |elem| {
                let id = NominationAffiliationIdentifier::read_eml(elem)?;
                id.registered_name
            },
            affiliation_type: ("Type", NS_EML) => |elem| elem.string_value()?,
            list_data: ListData::EML_NAME => |elem| ListData::read_eml(elem)?,
            candidates as Vec: NominationCandidate::EML_NAME => |elem| NominationCandidate::read_eml(elem)?,
        });

        if data.candidates.is_empty() {
            let err = EMLErrorKind::MissingElement(NominationCandidate::EML_NAME.as_owned())
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
            .child(("AffiliationIdentifier", NS_EML), |w| {
                w.child(("RegisteredName", NS_EML), |w| {
                    w.text(self.registered_name.as_ref())?.finish()
                })?
                .finish()
            })?
            .child(("Type", NS_EML), |elem| {
                elem.text(self.affiliation_type.raw().as_ref())?.finish()
            })?
            .child_elem(ListData::EML_NAME, &self.list_data)?
            .child_elems(NominationCandidate::EML_NAME, &self.candidates)?
            .finish()
    }
}

/// A candidate in a nomination document.
///
/// In EML 210, `Gender` and `QualifyingAddress` are required (unlike in 230b/230c
/// where they are optional). Additional fields like `Contact`, `Agent`,
/// `DateOfBirthAnnex` and `NationalIdentificationNumber` are also supported.
#[derive(Debug, Clone)]
pub struct NominationCandidate {
    /// The candidate identifier.
    pub identifier: CandidateIdentifier,

    /// The full name of the candidate.
    pub full_name: PersonNameStructure,

    /// The date of birth of the candidate, if present.
    pub date_of_birth: Option<StringValue<XsDate>>,

    /// The gender of the candidate (required in 210).
    pub gender: StringValue<Gender>,

    /// The qualifying address of the candidate (required in 210).
    pub qualifying_address: QualifyingAddress,

    /// Contact details for the candidate, if present.
    pub contact: Option<NominationContact>,

    /// Agent details for the candidate, if present.
    pub agent: Option<NominationAgent>,

    /// Alternative date of birth representation when exact date is unknown.
    pub date_of_birth_annex: Option<Box<str>>,

    /// National identification number (e.g. BSN in the Netherlands).
    pub national_identification_number: Option<Box<str>>,
}

impl EMLElement for NominationCandidate {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Candidate", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, NominationCandidate {
            identifier: CandidateIdentifier::EML_NAME => |elem| CandidateIdentifier::read_eml(elem)?,
            full_name: ("CandidateFullName", NS_EML) => |elem| PersonNameStructure::read_eml_element(elem)?,
            date_of_birth as Option: ("DateOfBirth", NS_EML) => |elem| elem.string_value()?,
            gender: ("Gender", NS_EML) => |elem| elem.string_value()?,
            qualifying_address: QualifyingAddress::EML_NAME => |elem| QualifyingAddress::read_eml(elem)?,
            contact as Option: NominationContact::EML_NAME => |elem| NominationContact::read_eml(elem)?,
            agent as Option: NominationAgent::EML_NAME => |elem| NominationAgent::read_eml(elem)?,
            date_of_birth_annex as Option: ("DateOfBirthAnnex", NS_KR) => |elem| elem.text_without_children()?,
            national_identification_number as Option: ("NationalIdentificationNumber", NS_KR) => |elem| elem.text_without_children()?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(CandidateIdentifier::EML_NAME, &self.identifier)?
            .child(
                ("CandidateFullName", NS_EML),
                write_eml_element(&self.full_name),
            )?
            .child_option(
                ("DateOfBirth", NS_EML),
                self.date_of_birth.as_ref(),
                |elem, value| elem.text(value.raw().as_ref())?.finish(),
            )?
            .child(("Gender", NS_EML), |elem| {
                elem.text(self.gender.raw().as_ref())?.finish()
            })?
            .child_elem(QualifyingAddress::EML_NAME, &self.qualifying_address)?
            .child_elem_option(NominationContact::EML_NAME, self.contact.as_ref())?
            .child_elem_option(NominationAgent::EML_NAME, self.agent.as_ref())?
            .child_option(
                ("DateOfBirthAnnex", NS_KR),
                self.date_of_birth_annex.as_ref(),
                |elem, value| elem.text(value.as_ref())?.finish(),
            )?
            .child_option(
                ("NationalIdentificationNumber", NS_KR),
                self.national_identification_number.as_ref(),
                |elem, value| elem.text(value.as_ref())?.finish(),
            )?
            .finish()
    }
}

/// Contact details (containing a mailing address).
#[derive(Debug, Clone)]
pub struct NominationContact {
    /// The mailing address.
    pub mailing_address: MailingAddress,
}

impl EMLElement for NominationContact {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Contact", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, NominationContact {
            mailing_address: MailingAddress::EML_NAME => |elem| MailingAddress::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(MailingAddress::EML_NAME, &self.mailing_address)?
            .finish()
    }
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

impl EMLElement for MailingAddress {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("MailingAddress", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let parent_name = elem.name()?.as_owned();
        let mut found_value = None;
        while let Some(mut next_child) = elem.next_child()? {
            let name = next_child.name()?;
            if found_value.is_some()
                || name != QualifyingAddressLocality::EML_NAME
                    && name != QualifyingAddressCountry::EML_NAME
            {
                let err = EMLErrorKind::UnexpectedElement(name.as_owned(), parent_name.clone())
                    .with_span(next_child.span());
                if next_child.parsing_mode().is_strict() {
                    return Err(err);
                } else {
                    next_child.push_err(err);
                    next_child.skip()?;
                }
            } else {
                match name {
                    name if name == QualifyingAddressLocality::EML_NAME => {
                        let locality = QualifyingAddressLocality::read_eml(&mut next_child)?;
                        found_value = Some(QualifyingAddress::Locality(locality));
                    }
                    name if name == QualifyingAddressCountry::EML_NAME => {
                        let country = QualifyingAddressCountry::read_eml(&mut next_child)?;
                        found_value = Some(QualifyingAddress::Country(country));
                    }
                    _ => unreachable!(),
                }
            }
        }
        let Some(value) = found_value else {
            return Err(EMLErrorKind::MissingChoiceElements(vec![
                QualifyingAddressLocality::EML_NAME.as_owned(),
                QualifyingAddressCountry::EML_NAME.as_owned(),
            ])
            .with_span(elem.span()));
        };
        Ok(MailingAddress { address: value })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        match &self.address {
            QualifyingAddress::Locality(locality) => {
                writer.child_elem(QualifyingAddressLocality::EML_NAME, locality)?
            }
            QualifyingAddress::Country(country) => {
                writer.child_elem(QualifyingAddressCountry::EML_NAME, country)?
            }
        }
        .finish()
    }
}

/// An agent for a candidate.
#[derive(Debug, Clone)]
pub struct NominationAgent {
    /// The role of the agent (e.g. "H10" or "H10a").
    pub role: Option<String>,

    /// The agent's name.
    pub agent_identifier: AgentIdentifier,

    /// Contact details for the agent, if present.
    pub contact: Option<NominationContact>,

    /// The living address of the agent.
    pub living_address: LivingAddress,
}

impl EMLElement for NominationAgent {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Agent", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, NominationAgent {
            role: elem.attribute_value("Role")?.map(Cow::into_owned),
            agent_identifier: AgentIdentifier::EML_NAME => |elem| AgentIdentifier::read_eml(elem)?,
            contact as Option: NominationContact::EML_NAME => |elem| NominationContact::read_eml(elem)?,
            living_address: LivingAddress::EML_NAME => |elem| LivingAddress::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr_opt("Role", self.role.as_ref())?
            .child_elem(AgentIdentifier::EML_NAME, &self.agent_identifier)?
            .child_elem_option(NominationContact::EML_NAME, self.contact.as_ref())?
            .child_elem(LivingAddress::EML_NAME, &self.living_address)?
            .finish()
    }
}

/// Job title used for a proposer in a nomination document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NominationJobTitle {
    /// inleveraar
    Submitter,
    /// plaatsvervanger van de inleveraar
    DeputySubmitter,
    /// gemachtigde voor het aangaan van lijstencombinaties
    CombinationRepresentative,
    /// plaatsvervanger voor het aangaan van lijstencombinaties
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

    fn to_raw_value(&self) -> Box<str> {
        self.to_eml_value().into()
    }
}

/// Agent identifier containing the agent's name.
#[derive(Debug, Clone)]
pub struct AgentIdentifier {
    /// The agent's name.
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

impl EMLElement for AgentIdentifier {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("AgentIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, AgentIdentifier {
            agent_name: ("AgentName", NS_EML) => |elem| PersonNameStructure::read_eml_element(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child(("AgentName", NS_EML), write_eml_element(&self.agent_name))?
            .finish()
    }
}

/// A living address (kr:LivingAddress).
#[derive(Debug, Clone)]
pub struct LivingAddress {
    /// The locality name.
    pub locality_name: Box<str>,

    /// The country name code, if present.
    pub country_name_code: Option<Box<str>>,
}

impl LivingAddress {
    /// Create a new `LivingAddress`.
    pub fn new(locality_name: impl Into<Box<str>>) -> Self {
        LivingAddress {
            locality_name: locality_name.into(),
            country_name_code: None,
        }
    }

    /// Set the country name code.
    pub fn with_country_name_code(mut self, code: impl Into<Box<str>>) -> Self {
        self.country_name_code = Some(code.into());
        self
    }
}

impl EMLElement for LivingAddress {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("LivingAddress", Some(NS_KR));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, LivingAddress {
            locality_name: ("LocalityName", NS_KR) => |elem| elem.text_without_children()?,
            country_name_code as Option: ("CountryNameCode", NS_KR) => |elem| elem.text_without_children()?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child(("LocalityName", NS_KR), |elem| {
                elem.text(self.locality_name.as_ref())?.finish()
            })?
            .child_option(
                ("CountryNameCode", NS_KR),
                self.country_name_code.as_ref(),
                |elem, value| elem.text(value.as_ref())?.finish(),
            )?
            .finish()
    }
}

/// The `<Nominate>` element containing proposers.
#[derive(Debug, Clone)]
pub struct NominationNominate {
    /// The proposers (minimum 2 required by schema).
    pub proposers: Vec<NominationProposer>,
}

impl NominationNominate {
    /// Create a new `NominationNominate` with the given proposers.
    pub fn new(proposers: Vec<NominationProposer>) -> Self {
        NominationNominate { proposers }
    }
}

impl EMLElement for NominationNominate {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Nominate", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let data = collect_struct!(elem, NominationNominate {
            proposers as Vec: NominationProposer::EML_NAME => |elem| NominationProposer::read_eml(elem)?,
        });

        if data.proposers.len() < 2 {
            let err = EMLErrorKind::MissingElement(NominationProposer::EML_NAME.as_owned())
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
            .child_elems(NominationProposer::EML_NAME, &self.proposers)?
            .finish()
    }
}

/// A proposer in a nomination document.
#[derive(Debug, Clone)]
pub struct NominationProposer {
    /// The proposer's name.
    pub name: PersonNameStructure,

    /// Contact details for the proposer (required).
    pub contact: NominationContact,

    /// The job title of the proposer.
    ///
    /// Valid values: "inleveraar", "plaatsvervanger van de inleveraar",
    /// "gemachtigde voor het aangaan van lijstencombinaties",
    /// "plaatsvervanger voor het aangaan van lijstencombinaties"
    pub job_title: StringValue<NominationJobTitle>,

    /// Optional identifier for the proposer (mandatory if deputy).
    pub id: Option<Box<str>>,

    /// The living address of the proposer, if present.
    pub living_address: Option<LivingAddress>,
}

impl EMLElement for NominationProposer {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Proposer", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, NominationProposer {
            name: ("Name", NS_EML) => |elem| PersonNameStructure::read_eml_element(elem)?,
            contact: NominationContact::EML_NAME => |elem| NominationContact::read_eml(elem)?,
            job_title: ("JobTitle", NS_EML) => |elem| elem.string_value()?,
            id as Option: ("Id", NS_EML) => |elem| elem.text_without_children()?,
            living_address as Option: LivingAddress::EML_NAME => |elem| LivingAddress::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child(("Name", NS_EML), write_eml_element(&self.name))?
            .child_elem(NominationContact::EML_NAME, &self.contact)?
            .child(("JobTitle", NS_EML), |elem| {
                elem.text(self.job_title.raw().as_ref())?.finish()
            })?
            .child_option(("Id", NS_EML), self.id.as_ref(), |elem, value| {
                elem.text(value.as_ref())?.finish()
            })?
            .child_elem_option(LivingAddress::EML_NAME, self.living_address.as_ref())?
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use chrono::{NaiveDate, NaiveDateTime};

    use super::*;
    use crate::{
        common::{AuthorityIdentifier, CandidateIdentifier, ElectionDomain, ListData, PersonName},
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
                registered_name: "Test Party".into(),
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
                        date_of_birth_annex: Some("XX-07-1990".into()),
                        national_identification_number: Some("123456789".into()),
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
                    id: Some("PV001".into()),
                    living_address: Some(
                        LivingAddress::new("Utrecht").with_country_name_code("NL"),
                    ),
                },
            ]))
            .build()
            .unwrap();

        let xml = nomination.write_eml_root_str(true, true).unwrap();
        assert_eq!(
            xml,
            include_str!("../../test-files/nomination/eml210_construction_output.eml.xml")
        );

        let parsed = Nomination::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        let xml2 = parsed.write_eml_root_str(true, true).unwrap();
        assert_eq!(xml, xml2);
    }

    #[test]
    fn test_nomination_parse_and_write_roundtrip() {
        let doc = include_str!("../../test-files/nomination/eml210_test.eml.xml");
        let nomination = Nomination::parse_eml(doc, EMLParsingMode::Strict)
            .ok()
            .expect("Failed to parse EML 210 document");

        assert_eq!(nomination.transaction_id.raw(), "1");
        assert!(nomination.managing_authority.is_some());
        assert_eq!(
            nomination.nomination_data.contest_identifier.name.as_ref(),
            "Test Contest"
        );
        assert_eq!(
            nomination
                .nomination_data
                .affiliation
                .registered_name
                .as_ref(),
            "Test Party"
        );
        assert!(!nomination.nomination_data.affiliation.candidates.is_empty());
        assert!(nomination.nomination_data.nominate.proposers.len() >= 2);

        let xml_output = nomination
            .write_eml_root_str(true, true)
            .expect("Failed to write EML 210 document");
        let reparsed = Nomination::parse_eml(&xml_output, EMLParsingMode::Strict)
            .ok()
            .expect("Failed to re-parse written EML 210 document");

        assert_eq!(
            reparsed.nomination_data.affiliation.registered_name,
            nomination.nomination_data.affiliation.registered_name
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
