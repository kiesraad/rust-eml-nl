//! Document variant for the EML_NL Candidate List (`230b`) document.

use std::borrow::Cow;

use crate::{
    EML_SCHEMA_VERSION, EMLError, NS_EML, NS_KR, NS_XAL,
    common::{
        AffiliationIdentifier, CandidateIdentifier, CanonicalizationMethod, ContestIdentifier,
        CreationDateTime, ElectionDomain, IssueDate, ListData, ManagingAuthority,
        PersonNameStructure, TransactionId,
    },
    documents::accepted_root,
    error::{EMLErrorKind, EMLResultExt},
    io::{
        EMLElement, EMLElementReader, EMLElementWriter, EMLReadElement as _, QualifiedName,
        collect_struct, write_eml_element,
    },
    utils::{
        AffiliationType, ElectionCategory, ElectionIdType, ElectionSubcategory, GenderType,
        StringValue, XsDate, XsDateOrDateTime,
    },
};

pub(crate) const EML_CANDIDATE_LISTS_ID: &str = "230b";

/// Representing a `230b` document, containing the candidate lists.
#[derive(Debug, Clone)]
pub struct CandidateLists {
    /// Transaction id of the document.
    pub transaction_id: TransactionId,

    /// Managing authority of the document.
    pub managing_authority: ManagingAuthority,

    /// Issue date of the document.
    pub issue_date: IssueDate,

    /// Creation date and time of the document.
    pub creation_date_time: CreationDateTime,

    /// Canonicalization method used in this document, if present.
    pub canonicalization_method: Option<CanonicalizationMethod>,

    /// The candidate lists contained in this document.
    pub candidate_list: CandidateListsCandidateList,
}

impl EMLElement for CandidateLists {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("EML", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        accepted_root(elem)?;

        let document_id = elem.attribute_value_req(("Id", None))?;
        if document_id != EML_CANDIDATE_LISTS_ID {
            return Err(EMLErrorKind::InvalidDocumentType(
                EML_CANDIDATE_LISTS_ID,
                document_id.to_string(),
            ))
            .with_span(elem.span());
        }

        Ok(collect_struct!(elem, CandidateLists {
            transaction_id: TransactionId::EML_NAME => |elem| TransactionId::read_eml(elem)?,
            managing_authority: ManagingAuthority::EML_NAME => |elem| ManagingAuthority::read_eml(elem)?,
            issue_date: IssueDate::EML_NAME => |elem| IssueDate::read_eml(elem)?,
            creation_date_time: CreationDateTime::EML_NAME => |elem| CreationDateTime::read_eml(elem)?,
            canonicalization_method as Option: CanonicalizationMethod::EML_NAME => |elem| CanonicalizationMethod::read_eml(elem)?,
            candidate_list: CandidateListsCandidateList::EML_NAME => |elem| CandidateListsCandidateList::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr(("Id", None), EML_CANDIDATE_LISTS_ID)?
            .attr(("SchemaVersion", None), EML_SCHEMA_VERSION)?
            .child_elem(TransactionId::EML_NAME, &self.transaction_id)?
            .child_elem(ManagingAuthority::EML_NAME, &self.managing_authority)?
            .child_elem(IssueDate::EML_NAME, &self.issue_date)?
            .child_elem(CreationDateTime::EML_NAME, &self.creation_date_time)?
            // Note: we don't output the CanonicalizationMethod because we aren't canonicalizing our output
            // .child_elem_option(
            //     CanonicalizationMethod::EML_NAME,
            //     self.canonicalization_method.as_ref(),
            // )?
            .child_elem(CandidateListsCandidateList::EML_NAME, &self.candidate_list)?
            .finish()?;

        Ok(())
    }
}

/// The root candidate list element.
#[derive(Debug, Clone)]
pub struct CandidateListsCandidateList {
    /// The date of the candidate list, if present.
    pub list_date: Option<CandidateListsListDate>,
    /// The election information.
    pub election: CandidateListsElection,
}

impl EMLElement for CandidateListsCandidateList {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("CandidateList", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, CandidateListsCandidateList {
            list_date as Option: CandidateListsListDate::EML_NAME => |elem| CandidateListsListDate::read_eml(elem)?,
            election: CandidateListsElection::EML_NAME => |elem| CandidateListsElection::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem_option(CandidateListsListDate::EML_NAME, self.list_date.as_ref())?
            .child_elem(CandidateListsElection::EML_NAME, &self.election)?
            .finish()
    }
}

/// The date of the candidate list.
#[derive(Debug, Clone)]
pub struct CandidateListsListDate(pub StringValue<XsDateOrDateTime>);

impl EMLElement for CandidateListsListDate {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("ListDate", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let value = elem.string_value()?;
        Ok(CandidateListsListDate(value))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer.text(self.0.raw().as_ref())?.finish()
    }
}

/// The election information in the candidate lists.
#[derive(Debug, Clone)]
pub struct CandidateListsElection {
    /// Identifier for the election.
    pub identifier: CandidateListsElectionIdentifier,
    /// Election contest details.
    pub contest: CandidateListsContest,
}

impl EMLElement for CandidateListsElection {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Election", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, CandidateListsElection {
            identifier: CandidateListsElectionIdentifier::EML_NAME => |elem| CandidateListsElectionIdentifier::read_eml(elem)?,
            contest: CandidateListsContest::EML_NAME => |elem| CandidateListsContest::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(CandidateListsElectionIdentifier::EML_NAME, &self.identifier)?
            .child_elem(CandidateListsContest::EML_NAME, &self.contest)?
            .finish()
    }
}

/// Identifier for the election.
#[derive(Debug, Clone)]
pub struct CandidateListsElectionIdentifier {
    /// Id of the election
    pub id: StringValue<ElectionIdType>,
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
    /// Nomination date for the election
    pub nomination_date: StringValue<XsDate>,
}

impl EMLElement for CandidateListsElectionIdentifier {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ElectionIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(
            elem,
            CandidateListsElectionIdentifier {
                id: elem.string_value_attr("Id", None)?,
                name as Option: ("ElectionName", NS_EML) => |elem| elem.text_without_children()?,
                category: ("ElectionCategory", NS_EML) => |elem| elem.string_value()?,
                subcategory as Option: ("ElectionSubcategory", NS_KR) => |elem| elem.string_value()?,
                domain as Option: ElectionDomain::EML_NAME => |elem| ElectionDomain::read_eml(elem)?,
                election_date: ("ElectionDate", NS_KR) => |elem| elem.string_value()?,
                nomination_date: ("NominationDate", NS_KR) => |elem| elem.string_value()?,
            }
        ))
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

/// Election contest details.
#[derive(Debug, Clone)]
pub struct CandidateListsContest {
    /// Identifier for the contest.
    pub identifier: ContestIdentifier,
    /// Affiliations participating in the contest.
    pub affiliations: Vec<CandidateListsAffiliation>,
}

impl EMLElement for CandidateListsContest {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Contest", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, CandidateListsContest {
            identifier: ContestIdentifier::EML_NAME => |elem| ContestIdentifier::read_eml(elem)?,
            affiliations as Vec: CandidateListsAffiliation::EML_NAME => |elem| CandidateListsAffiliation::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        let mut writer = writer.child_elem(ContestIdentifier::EML_NAME, &self.identifier)?;
        for affiliation in &self.affiliations {
            writer = writer.child_elem(CandidateListsAffiliation::EML_NAME, affiliation)?;
        }
        writer.finish()
    }
}

/// An affiliation participating in the contest.
#[derive(Debug, Clone)]
pub struct CandidateListsAffiliation {
    /// The affiliation identifier.
    pub identifier: AffiliationIdentifier,

    /// The affiliation type.
    pub affiliation_type: StringValue<AffiliationType>,

    /// The list data of the affiliation.
    pub list_data: ListData,

    /// The candidates of the affiliation.
    pub candidates: Vec<CandidateListsCandidate>,
}

impl EMLElement for CandidateListsAffiliation {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Affiliation", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, CandidateListsAffiliation {
            identifier: AffiliationIdentifier::EML_NAME => |elem| AffiliationIdentifier::read_eml(elem)?,
            affiliation_type: ("Type", NS_EML) => |elem| elem.string_value()?,
            list_data: ListData::EML_NAME => |elem| ListData::read_eml(elem)?,
            candidates as Vec: CandidateListsCandidate::EML_NAME => |elem| CandidateListsCandidate::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        let mut writer = writer
            .child_elem(AffiliationIdentifier::EML_NAME, &self.identifier)?
            .child(("Type", NS_EML), |elem| {
                elem.text(self.affiliation_type.raw().as_ref())?.finish()
            })?
            .child_elem(ListData::EML_NAME, &self.list_data)?;

        for candidate in &self.candidates {
            writer = writer.child_elem(CandidateListsCandidate::EML_NAME, candidate)?;
        }
        writer.finish()
    }
}

/// A candidate in an affiliation.
#[derive(Debug, Clone)]
pub struct CandidateListsCandidate {
    /// The candidate identifier.
    pub identifier: CandidateIdentifier,

    /// The full name of the candidate.
    pub full_name: PersonNameStructure,

    /// The date of birth of the candidate, if present.
    pub date_of_birth: Option<StringValue<XsDate>>,

    /// The gender of the candidate, if present.
    pub gender: Option<StringValue<GenderType>>,

    /// The qualifying address of the candidate.
    pub qualifying_address: QualifyingAddress,
}

impl EMLElement for CandidateListsCandidate {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Candidate", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, CandidateListsCandidate {
            identifier: CandidateIdentifier::EML_NAME => |elem| CandidateIdentifier::read_eml(elem)?,
            full_name: ("CandidateFullName", NS_EML) => |elem| PersonNameStructure::read_eml_element(elem)?,
            date_of_birth as Option: ("DateOfBirth", NS_EML) => |elem| elem.string_value()?,
            gender as Option: ("Gender", NS_EML) => |elem| elem.string_value()?,
            qualifying_address: QualifyingAddress::EML_NAME => |elem| QualifyingAddress::read_eml(elem)?,
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
            .child_option(("Gender", NS_EML), self.gender.as_ref(), |elem, value| {
                elem.text(value.raw().as_ref())?.finish()
            })?
            .child_elem(QualifyingAddress::EML_NAME, &self.qualifying_address)?
            .finish()
    }
}

/// The qualifying address of a candidate.
#[derive(Debug, Clone)]
pub enum QualifyingAddress {
    /// Qualifying address is a locality only.
    Locality(QualifyingAddressLocality),
    /// Qualifying address is a locality in a specific country.
    Country(QualifyingAddressCountry),
}

impl EMLElement for QualifyingAddress {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("QualifyingAddress", Some(NS_EML));

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
                    .add_span(next_child.span());
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
            .add_span(elem.span()));
        };
        Ok(value)
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        match self {
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

/// Qualifying address locality.
#[derive(Debug, Clone)]
pub struct QualifyingAddressLocality {
    /// The address line, if present.
    pub address_line: Option<AddressLine>,
    /// The locality name.
    pub locality_name: LocalityName,
    /// The postal code, if present.
    pub postal_code: Option<PostalCode>,
    /// The Type attribute, if present.
    pub locality_type: Option<String>,
    /// The UsageType attribute, if present.
    pub usage_type: Option<String>,
    /// The Indicator attribute, if present.
    pub indicator: Option<String>,
}

impl QualifyingAddressLocality {
    /// Create a new QualifyingAddressLocality.
    pub fn new(locality_name: impl Into<String>) -> Self {
        QualifyingAddressLocality {
            address_line: None,
            locality_name: LocalityName::new(locality_name),
            postal_code: None,
            locality_type: None,
            usage_type: None,
            indicator: None,
        }
    }
}

impl EMLElement for QualifyingAddressLocality {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Locality", Some(NS_XAL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, QualifyingAddressLocality {
            address_line as Option: AddressLine::EML_NAME => |elem| AddressLine::read_eml(elem)?,
            locality_name: LocalityName::EML_NAME => |elem| LocalityName::read_eml(elem)?,
            postal_code as Option: PostalCode::EML_NAME => |elem| PostalCode::read_eml(elem)?,
            locality_type: elem.attribute_value("Type")?.map(Cow::into_owned),
            usage_type: elem.attribute_value("UsageType")?.map(Cow::into_owned),
            indicator: elem.attribute_value("Indicator")?.map(Cow::into_owned),
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr_opt("Type", self.locality_type.as_ref())?
            .attr_opt("UsageType", self.usage_type.as_ref())?
            .attr_opt("Indicator", self.indicator.as_ref())?
            .child_elem_option(AddressLine::EML_NAME, self.address_line.as_ref())?
            .child_elem(LocalityName::EML_NAME, &self.locality_name)?
            .child_elem_option(PostalCode::EML_NAME, self.postal_code.as_ref())?
            .finish()
    }
}

/// Address line information.
#[derive(Debug, Clone)]
pub struct AddressLine {
    /// The address line value.
    pub value: String,
    /// The Type attribute, if present.
    pub address_line_type: Option<String>,
    /// The Code attribute, if present.
    pub code: Option<String>,
}

impl AddressLine {
    /// Create a new AddressLine.
    pub fn new(value: impl Into<String>) -> Self {
        AddressLine {
            value: value.into(),
            address_line_type: None,
            code: None,
        }
    }
}

impl EMLElement for AddressLine {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("AddressLine", Some(NS_XAL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(AddressLine {
            value: elem.text_without_children()?,
            address_line_type: elem.attribute_value("Type")?.map(Cow::into_owned),
            code: elem.attribute_value("Code")?.map(Cow::into_owned),
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr_opt("Type", self.address_line_type.as_ref())?
            .attr_opt("Code", self.code.as_ref())?
            .text(self.value.as_ref())?
            .finish()
    }
}

/// The locality name.
#[derive(Debug, Clone)]
pub struct LocalityName {
    /// The locality name.
    pub value: String,
    /// The Type attribute, if present.
    pub locality_name_type: Option<String>,
    /// The Code attribute, if present.
    pub code: Option<String>,
}

impl LocalityName {
    /// Create a new LocalityName.
    pub fn new(value: impl Into<String>) -> Self {
        LocalityName {
            value: value.into(),
            locality_name_type: None,
            code: None,
        }
    }
}

impl EMLElement for LocalityName {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("LocalityName", Some(NS_XAL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(LocalityName {
            value: elem.text_without_children()?,
            locality_name_type: elem.attribute_value("Type")?.map(Cow::into_owned),
            code: elem.attribute_value("Code")?.map(Cow::into_owned),
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr_opt("Type", self.locality_name_type.as_ref())?
            .attr_opt("Code", self.code.as_ref())?
            .text(self.value.as_ref())?
            .finish()
    }
}

/// Postal code information.
#[derive(Debug, Clone)]
pub struct PostalCode {
    /// Number of the postal code.
    pub postal_code_number: PostalCodeNumber,
}

impl PostalCode {
    /// Create a new PostalCode.
    pub fn new(postal_code_number: impl Into<String>) -> Self {
        PostalCode {
            postal_code_number: PostalCodeNumber::new(postal_code_number),
        }
    }
}

impl EMLElement for PostalCode {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("PostalCode", Some(NS_XAL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, PostalCode {
            postal_code_number: PostalCodeNumber::EML_NAME => |elem| PostalCodeNumber::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(PostalCodeNumber::EML_NAME, &self.postal_code_number)?
            .finish()
    }
}

/// The postal code number.
#[derive(Debug, Clone)]
pub struct PostalCodeNumber {
    /// The postal code number value.
    pub value: String,
    /// The Type attribute, if present.
    pub postal_code_number_type: Option<String>,
    /// The Code attribute, if present.
    pub code: Option<String>,
}

impl EMLElement for PostalCodeNumber {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("PostalCodeNumber", Some(NS_XAL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(PostalCodeNumber {
            value: elem.text_without_children()?,
            postal_code_number_type: elem.attribute_value("Type")?.map(Cow::into_owned),
            code: elem.attribute_value("Code")?.map(Cow::into_owned),
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr_opt("Type", self.postal_code_number_type.as_ref())?
            .attr_opt("Code", self.code.as_ref())?
            .text(self.value.as_ref())?
            .finish()
    }
}

impl PostalCodeNumber {
    /// Create a new PostalCodeNumber.
    pub fn new(value: impl Into<String>) -> Self {
        PostalCodeNumber {
            value: value.into(),
            postal_code_number_type: None,
            code: None,
        }
    }
}

/// Qualifying address country.
#[derive(Debug, Clone)]
pub struct QualifyingAddressCountry {
    /// The country name code, if present.
    pub country_name_code: Option<CountryNameCode>,
    /// The locality within the country.
    pub locality: QualifyingAddressLocality,
}

impl QualifyingAddressCountry {
    /// Create a new QualifyingAddressCountry.
    pub fn new(
        country_code: Option<impl Into<String>>,
        locality: QualifyingAddressLocality,
    ) -> Self {
        Self {
            country_name_code: country_code.map(|code| CountryNameCode::new(code)),
            locality,
        }
    }
}

impl EMLElement for QualifyingAddressCountry {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Country", Some(NS_XAL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, QualifyingAddressCountry {
            country_name_code as Option: CountryNameCode::EML_NAME => |elem| CountryNameCode::read_eml(elem)?,
            locality: QualifyingAddressLocality::EML_NAME => |elem| QualifyingAddressLocality::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem_option(CountryNameCode::EML_NAME, self.country_name_code.as_ref())?
            .child_elem(QualifyingAddressLocality::EML_NAME, &self.locality)?
            .finish()
    }
}

/// Country name code information.
#[derive(Debug, Clone)]
pub struct CountryNameCode {
    /// The country name code value.
    pub value: String,
    /// The Scheme attribute, if present.
    pub scheme: Option<String>,
    /// The Code attribute, if present.
    pub code: Option<String>,
}

impl CountryNameCode {
    /// Create a new CountryNameCode.
    pub fn new(value: impl Into<String>) -> Self {
        CountryNameCode {
            value: value.into(),
            scheme: None,
            code: None,
        }
    }
}

impl EMLElement for CountryNameCode {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("CountryNameCode", Some(NS_XAL));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(CountryNameCode {
            value: elem.text_without_children()?,
            scheme: elem.attribute_value("Scheme")?.map(Cow::into_owned),
            code: elem.attribute_value("Code")?.map(Cow::into_owned),
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr_opt("Scheme", self.scheme.as_ref())?
            .attr_opt("Code", self.code.as_ref())?
            .text(self.value.as_ref())?
            .finish()
    }
}
