//! Document variant for the EML_NL Candidate List (`230b`) document.

use std::{borrow::Cow, num::NonZeroU64, str::FromStr};

use crate::{
    EML_SCHEMA_VERSION, EMLError, NS_EML, NS_KR, NS_XAL,
    common::{
        CandidateIdentifier, CanonicalizationMethod, ContestIdentifier, CountryNameCode,
        CreationDateTime, ElectionDomain, IssueDate, ListData, ListDataBelongsToCombination,
        LocalityName, ManagingAuthority, PersonNameStructure, TransactionId,
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
        AffiliationId, AffiliationType, ElectionCategory, ElectionId, ElectionSubcategory, Gender,
        PublicationLanguage, StringValue, XsDate, XsDateOrDateTime, XsDateTime,
    },
};

/// Representing a `230b` document, containing the candidate lists.
#[derive(Debug, Clone)]
pub struct CandidateLists {
    /// The type of the candidate lists document.
    pub lists_type: CandidateListsType,

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

impl CandidateLists {
    /// Create a new builder for the [`CandidateLists`] document.
    pub fn builder() -> CandidateListsBuilder {
        CandidateListsBuilder::new()
    }
}

impl FromStr for CandidateLists {
    type Err = EMLError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::io::EMLRead as _;
        Self::parse_eml(s, crate::io::EMLParsingMode::Strict).ok()
    }
}

impl TryFrom<&str> for CandidateLists {
    type Error = EMLError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use crate::io::EMLRead as _;
        Self::parse_eml(value, crate::io::EMLParsingMode::Strict).ok()
    }
}

impl TryFrom<CandidateLists> for String {
    type Error = EMLError;

    fn try_from(value: CandidateLists) -> Result<Self, Self::Error> {
        use crate::io::EMLWrite as _;
        value.write_eml_root_str(true)
    }
}

/// Builder for the [`CandidateLists`] document.
#[derive(Debug, Clone)]
pub struct CandidateListsBuilder {
    lists_type: Option<CandidateListsType>,
    transaction_id: Option<TransactionId>,
    managing_authority: Option<ManagingAuthority>,
    issue_date: Option<IssueDate>,
    creation_date_time: Option<CreationDateTime>,
    canonicalization_method: Option<CanonicalizationMethod>,
    candidate_list: Option<CandidateListsCandidateList>,
    election_identifier: Option<CandidateListsElectionIdentifier>,
    list_date: Option<CandidateListsListDate>,
    contests: Vec<CandidateListsContest>,
}

impl CandidateListsBuilder {
    /// Create a new builder for the [`CandidateLists`] document.
    pub fn new() -> Self {
        CandidateListsBuilder {
            lists_type: None,
            transaction_id: None,
            managing_authority: None,
            issue_date: None,
            creation_date_time: None,
            canonicalization_method: None,
            candidate_list: None,
            election_identifier: None,
            list_date: None,
            contests: vec![],
        }
    }

    /// Set the list type for the document.
    pub fn lists_type(mut self, list_type: impl Into<CandidateListsType>) -> Self {
        self.lists_type = Some(list_type.into());
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

    /// Set the candidate list for the document.
    ///
    /// You may either set the entire candidate list at once using this
    /// method, or use any of [`Self::election_identifier`], [`Self::list_date`],
    /// [`Self::contests`] and/or [`Self::push_contest`] to construct the
    /// individual components of the CandidateList and Election elements to
    /// allow this builder to construct them for you.
    pub fn candidate_list(
        mut self,
        candidate_list: impl Into<CandidateListsCandidateList>,
    ) -> Self {
        self.candidate_list = Some(candidate_list.into());
        self
    }

    /// Set the list date for the contained CandidateList element.
    ///
    /// This only has effect if the candidate list was not set directly using
    /// [`Self::candidate_list`].
    pub fn list_date(mut self, list_date: impl Into<XsDateOrDateTime>) -> Self {
        self.list_date = Some(CandidateListsListDate::from(list_date.into()));
        self
    }

    /// Set the election identifier for the contained Election element.
    ///
    /// This only has effect if the candidate list was not set directly using
    /// [`Self::candidate_list`].
    pub fn election_identifier(
        mut self,
        election_identifier: impl Into<CandidateListsElectionIdentifier>,
    ) -> Self {
        self.election_identifier = Some(election_identifier.into());
        self
    }

    /// Set the list of contests within the election. This will replace any
    /// existing contests set using this method or the [`Self::push_contest`]
    /// method.
    ///
    /// This only has effect if the candidate list was not set directly using
    /// [`Self::candidate_list`].
    pub fn contests(mut self, contests: impl Into<Vec<CandidateListsContest>>) -> Self {
        self.contests = contests.into();
        self
    }

    /// Add a contest to the election.
    ///
    /// This only has effect if the candidate list was not set directly using
    /// [`Self::candidate_list`].
    pub fn push_contest(mut self, contest: impl Into<CandidateListsContest>) -> Self {
        self.contests.push(contest.into());
        self
    }

    /// Build the `CandidateLists` document, returning an error if any required fields are missing.
    pub fn build(self) -> Result<CandidateLists, EMLError> {
        Ok(CandidateLists {
            lists_type: self
                .lists_type
                .ok_or_else(|| EMLErrorKind::MissingBuildProperty("lists_type").without_span())?,
            transaction_id: self
                .transaction_id
                .ok_or(EMLErrorKind::MissingBuildProperty("transaction_id").without_span())?,
            managing_authority: self
                .managing_authority
                .ok_or(EMLErrorKind::MissingBuildProperty("managing_authority").without_span())?,
            issue_date: self
                .issue_date
                .ok_or(EMLErrorKind::MissingBuildProperty("issue_date").without_span())?,
            creation_date_time: self
                .creation_date_time
                .ok_or(EMLErrorKind::MissingBuildProperty("creation_date_time").without_span())?,
            canonicalization_method: self.canonicalization_method,
            candidate_list: self.candidate_list.map_or_else(
                || {
                    if self.contests.is_empty() {
                        return Err(EMLErrorKind::MissingBuildProperty("contests").without_span());
                    }

                    let election = CandidateListsElection::new(self.election_identifier.ok_or(
                        EMLErrorKind::MissingBuildProperty("election_identifier").without_span(),
                    )?)
                    .with_contests(self.contests);
                    let list = CandidateListsCandidateList::new(election);
                    let list = if let Some(list_date) = self.list_date {
                        list.with_list_date(list_date)
                    } else {
                        list
                    };

                    Ok(list)
                },
                Ok,
            )?,
        })
    }
}

impl Default for CandidateListsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EMLElement for CandidateLists {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("EML", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        accepted_root(elem)?;

        let document_id = elem.attribute_value_req(("Id", None))?;
        let candidate_lists_type = CandidateListsType::from_eml_id(document_id.as_ref())
            .map_err(|e| e.into_kind().with_span(elem.span()))?;

        Ok(collect_struct!(elem, CandidateLists {
            lists_type: candidate_lists_type,
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
            .attr(("Id", None), self.lists_type.to_eml_id())?
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
            .finish()
    }
}

/// EML document ID for candidate lists of a single district.
pub(crate) const EML_CANDIDATE_LISTS_SINGLE_ID: &str = "230b";

/// EML document ID for candidate lists of multiple districts.
pub(crate) const EML_CANDIDATE_LISTS_MULTIPLE_ID: &str = "230c";

/// Type of CandidateLists document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CandidateListsType {
    /// Representing a `230b` document, containing the candidate lists for a single district.
    Single,
    /// Representing a `230c` document, containing the candidate lists for multiple districts.
    Multiple,
}

impl CandidateListsType {
    /// Create a CandidateListsType from an EML document ID string.
    pub fn from_eml_id(s: impl AsRef<str>) -> Result<Self, EMLError> {
        let data = s.as_ref();
        match data {
            EML_CANDIDATE_LISTS_SINGLE_ID => Ok(CandidateListsType::Single),
            EML_CANDIDATE_LISTS_MULTIPLE_ID => Ok(CandidateListsType::Multiple),
            _ => {
                Err(EMLErrorKind::InvalidDocumentType("230b/230c", data.to_string()).without_span())
            }
        }
    }

    /// Get the EML document ID string for this CandidateListsType.
    pub fn to_eml_id(&self) -> &'static str {
        match self {
            CandidateListsType::Single => EML_CANDIDATE_LISTS_SINGLE_ID,
            CandidateListsType::Multiple => EML_CANDIDATE_LISTS_MULTIPLE_ID,
        }
    }

    /// Get a friendly name for this CandidateListsType.
    pub fn to_friendly_name(&self) -> &'static str {
        match self {
            CandidateListsType::Single => "Candidate Lists",
            CandidateListsType::Multiple => "Candidate Lists Total",
        }
    }

    /// Returns if the given EML document ID string is a valid CandidateListsType ID.
    pub fn is_valid_eml_id(s: &str) -> bool {
        matches!(
            s,
            EML_CANDIDATE_LISTS_SINGLE_ID | EML_CANDIDATE_LISTS_MULTIPLE_ID
        )
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

impl CandidateListsCandidateList {
    /// Create a new CandidateList element for the given election
    pub fn new(election: CandidateListsElection) -> Self {
        Self {
            list_date: None,
            election,
        }
    }

    /// Set the list date for this CandidateList
    pub fn with_list_date(mut self, list_date: CandidateListsListDate) -> Self {
        self.list_date = Some(list_date);
        self
    }
}

impl From<CandidateListsElection> for CandidateListsCandidateList {
    fn from(election: CandidateListsElection) -> Self {
        Self::new(election)
    }
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

impl From<XsDateOrDateTime> for CandidateListsListDate {
    fn from(value: XsDateOrDateTime) -> Self {
        CandidateListsListDate(StringValue::from_value(value))
    }
}

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
    pub contests: Vec<CandidateListsContest>,
}

impl CandidateListsElection {
    /// Create a new Election element with the given identifier and contest
    pub fn new(identifier: impl Into<CandidateListsElectionIdentifier>) -> Self {
        Self {
            identifier: identifier.into(),
            contests: vec![],
        }
    }

    /// Set the list of contests within this election. This will replace any
    /// existing contests set using this method or the [`Self::push_contest`]
    /// method.
    pub fn with_contests(mut self, contests: impl Into<Vec<CandidateListsContest>>) -> Self {
        self.contests = contests.into();
        self
    }

    /// Add a contest to this election.
    pub fn push_contest(mut self, contest: impl Into<CandidateListsContest>) -> Self {
        self.contests.push(contest.into());
        self
    }
}

impl EMLElement for CandidateListsElection {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Election", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let data = collect_struct!(elem, CandidateListsElection {
            identifier: CandidateListsElectionIdentifier::EML_NAME => |elem| CandidateListsElectionIdentifier::read_eml(elem)?,
            contests as Vec: CandidateListsContest::EML_NAME => |elem| CandidateListsContest::read_eml(elem)?,
        });

        if data.contests.is_empty() {
            let err = EMLErrorKind::MissingElement(CandidateListsContest::EML_NAME.as_owned())
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
            .child_elem(CandidateListsElectionIdentifier::EML_NAME, &self.identifier)?
            .child_elems(CandidateListsContest::EML_NAME, &self.contests)?
            .finish()
    }
}

/// Identifier for the election.
#[derive(Debug, Clone)]
pub struct CandidateListsElectionIdentifier {
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

    /// Nomination date for the election
    pub nomination_date: StringValue<XsDate>,
}

impl CandidateListsElectionIdentifier {
    /// Create a new Election Identifier builder
    pub fn builder() -> ElectionIdentifierBuilder {
        ElectionIdentifierBuilder::new()
    }
}

impl EMLElement for CandidateListsElectionIdentifier {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ElectionIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let data = collect_struct!(
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

        // check that the election subcategory is valid for the election category, if both are present
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

/// Election contest details.
#[derive(Debug, Clone)]
pub struct CandidateListsContest {
    /// Identifier for the contest.
    pub identifier: ContestIdentifier,

    /// Affiliations participating in the contest.
    pub affiliations: Vec<CandidateListsAffiliation>,
}

/// Builder for the election contest details, see [`CandidateListsContest`].
pub struct CandidateListsContestBuilder {
    identifier: Option<ContestIdentifier>,
    affiliations: Vec<CandidateListsAffiliation>,
}

impl CandidateListsContestBuilder {
    /// Create a new builder for the election contest details.
    pub fn new() -> Self {
        Self {
            identifier: None,
            affiliations: vec![],
        }
    }

    /// Set the identifier for the contest.
    pub fn identifier(mut self, identifier: impl Into<ContestIdentifier>) -> Self {
        self.identifier = Some(identifier.into());
        self
    }

    /// Set the affiliations for the contest. This will replace any existing affiliations.
    pub fn affiliations(mut self, affiliations: impl Into<Vec<CandidateListsAffiliation>>) -> Self {
        self.affiliations = affiliations.into();
        self
    }

    /// Add an affiliation to the contest.
    pub fn push_affiliation(mut self, affiliation: impl Into<CandidateListsAffiliation>) -> Self {
        self.affiliations.push(affiliation.into());
        self
    }

    /// Build the contest, returning an error if any required fields are missing.
    pub fn build(self) -> Result<CandidateListsContest, EMLError> {
        if self.affiliations.is_empty() {
            return Err(EMLErrorKind::MissingBuildProperty("affiliations").without_span());
        }

        Ok(CandidateListsContest {
            identifier: self
                .identifier
                .ok_or_else(|| EMLErrorKind::MissingBuildProperty("identifier").without_span())?,
            affiliations: self.affiliations,
        })
    }
}

impl Default for CandidateListsContestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CandidateListsContest {
    /// Create a new builder for building a contest for the candidate lists document.
    pub fn builder() -> CandidateListsContestBuilder {
        CandidateListsContestBuilder::new()
    }
}

impl EMLElement for CandidateListsContest {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Contest", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let data = collect_struct!(elem, CandidateListsContest {
            identifier: ContestIdentifier::EML_NAME => |elem| ContestIdentifier::read_eml(elem)?,
            affiliations as Vec: CandidateListsAffiliation::EML_NAME => |elem| CandidateListsAffiliation::read_eml(elem)?,
        });

        if data.affiliations.is_empty() {
            let err = EMLErrorKind::MissingElement(CandidateListsAffiliation::EML_NAME.as_owned())
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
            .child_elem(ContestIdentifier::EML_NAME, &self.identifier)?
            .child_elems(CandidateListsAffiliation::EML_NAME, &self.affiliations)?
            .finish()
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

impl CandidateListsAffiliation {
    /// Create a new builder for building an affiliation for the candidate lists document.
    pub fn builder() -> CandidateListsAffiliationBuilder {
        CandidateListsAffiliationBuilder::new()
    }
}

/// Builder for an affiliation participating in the contest.
pub struct CandidateListsAffiliationBuilder {
    id: Option<AffiliationId>,
    registered_name: Option<String>,
    affiliation_type: Option<AffiliationType>,
    publish_gender: Option<bool>,
    publication_language: Option<PublicationLanguage>,
    belongs_to_set: Option<NonZeroU64>,
    belongs_to_combination: Option<ListDataBelongsToCombination>,
    candidates: Vec<CandidateListsCandidate>,
}

impl CandidateListsAffiliationBuilder {
    /// Create a new builder for building an affiliation for the candidate lists document.
    pub fn new() -> Self {
        CandidateListsAffiliationBuilder {
            id: None,
            registered_name: None,
            affiliation_type: None,
            publish_gender: None,
            publication_language: None,
            belongs_to_set: None,
            belongs_to_combination: None,
            candidates: vec![],
        }
    }

    /// Set the affiliation id for the affiliation.
    pub fn id(mut self, id: impl Into<AffiliationId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the registered name for the affiliation.
    pub fn registered_name(mut self, registered_name: impl Into<String>) -> Self {
        self.registered_name = Some(registered_name.into());
        self
    }

    /// Set the affiliation type for the affiliation.
    pub fn affiliation_type(mut self, affiliation_type: impl Into<AffiliationType>) -> Self {
        self.affiliation_type = Some(affiliation_type.into());
        self
    }

    /// Set whether to publish genders
    pub fn publish_gender(mut self, publish_gender: bool) -> Self {
        self.publish_gender = Some(publish_gender);
        self
    }

    /// Set the publication language for the affiliation.
    pub fn publication_language(
        mut self,
        publication_language: impl Into<PublicationLanguage>,
    ) -> Self {
        self.publication_language = Some(publication_language.into());
        self
    }

    /// Set the set that this affiliation belongs to.
    pub fn belongs_to_set(mut self, belongs_to_set: NonZeroU64) -> Self {
        self.belongs_to_set = Some(belongs_to_set);
        self
    }

    /// Set the combination that this affiliation belongs to.
    pub fn belongs_to_combination(
        mut self,
        belongs_to_combination: impl Into<ListDataBelongsToCombination>,
    ) -> Self {
        self.belongs_to_combination = Some(belongs_to_combination.into());
        self
    }

    /// Set the candidates for this affiliation.
    pub fn candidates(mut self, candidates: impl Into<Vec<CandidateListsCandidate>>) -> Self {
        self.candidates = candidates.into();
        self
    }

    /// Add a candidate to the list of candidates for this affiliation.
    pub fn push_candidate(mut self, candidate: impl Into<CandidateListsCandidate>) -> Self {
        self.candidates.push(candidate.into());
        self
    }

    /// Build the affiliation, returning an error if any required fields are missing.
    pub fn build(self) -> Result<CandidateListsAffiliation, EMLError> {
        if self.candidates.is_empty() {
            return Err(EMLErrorKind::MissingBuildProperty("candidates").without_span());
        }

        Ok(CandidateListsAffiliation {
            identifier: AffiliationIdentifier::new(
                self.id
                    .ok_or_else(|| EMLErrorKind::MissingBuildProperty("id").without_span())?,
                self.registered_name,
            ),
            affiliation_type: StringValue::from_value(self.affiliation_type.ok_or_else(|| {
                EMLErrorKind::MissingBuildProperty("affiliation_type").without_span()
            })?),
            list_data: ListData {
                publish_gender: StringValue::from_value(self.publish_gender.ok_or_else(|| {
                    EMLErrorKind::MissingBuildProperty("publish_gender").without_span()
                })?),
                publication_language: self.publication_language.map(StringValue::from_value),
                belongs_to_set: self.belongs_to_set.map(StringValue::from_value),
                belongs_to_combination: self.belongs_to_combination.map(StringValue::from_value),
                contests: vec![],
            },
            candidates: self.candidates,
        })
    }
}

impl Default for CandidateListsAffiliationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EMLElement for CandidateListsAffiliation {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Affiliation", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let data = collect_struct!(elem, CandidateListsAffiliation {
            identifier: AffiliationIdentifier::EML_NAME => |elem| AffiliationIdentifier::read_eml(elem)?,
            affiliation_type: ("Type", NS_EML) => |elem| elem.string_value()?,
            list_data: ListData::EML_NAME => |elem| ListData::read_eml(elem)?,
            candidates as Vec: CandidateListsCandidate::EML_NAME => |elem| CandidateListsCandidate::read_eml(elem)?,
        });

        if data.candidates.is_empty() {
            let err = EMLErrorKind::MissingElement(CandidateListsCandidate::EML_NAME.as_owned())
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
            .child_elem(AffiliationIdentifier::EML_NAME, &self.identifier)?
            .child(("Type", NS_EML), |elem| {
                elem.text(self.affiliation_type.raw().as_ref())?.finish()
            })?
            .child_elem(ListData::EML_NAME, &self.list_data)?
            .child_elems(CandidateListsCandidate::EML_NAME, &self.candidates)?
            .finish()
    }
}

/// An affiliation identifier consisting of an id and a registered name.
#[derive(Debug, Clone)]
pub struct AffiliationIdentifier {
    /// The affiliation id.
    pub id: StringValue<AffiliationId>,

    /// The registered name of the affiliation.
    pub registered_name: Option<String>,
}

impl AffiliationIdentifier {
    /// Create a new AffiliationIdentifier.
    pub fn new(id: AffiliationId, registered_name: Option<impl Into<String>>) -> Self {
        Self {
            id: StringValue::Parsed(id),
            registered_name: registered_name.map(|name| name.into()),
        }
    }
}

impl EMLElement for AffiliationIdentifier {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("AffiliationIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(
            elem,
            AffiliationIdentifier {
                id: elem.string_value_attr("Id", None)?,
                registered_name: ("RegisteredName", NS_EML) => |elem| elem.text_without_children_opt()?,
            }
        ))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr("Id", self.id.raw().as_ref())?
            .child(("RegisteredName", NS_EML), |w| {
                if let Some(name) = &self.registered_name {
                    w.text(name)?.finish()
                } else {
                    w.empty()
                }
            })?
            .finish()?;
        Ok(())
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
    pub gender: Option<StringValue<Gender>>,

    /// The qualifying address of the candidate.
    pub qualifying_address: Option<QualifyingAddress>,
}

impl CandidateListsCandidate {
    /// Create a new builder for building a candidate for the candidate lists document.
    pub fn builder() -> CandidateListsCandidateBuilder {
        CandidateListsCandidateBuilder::new()
    }
}

/// A builder for building [`CandidateListsCandidate`] structs.
#[derive(Debug, Clone)]
pub struct CandidateListsCandidateBuilder {
    identifier: Option<CandidateIdentifier>,
    date_of_birth: Option<XsDate>,
    gender: Option<Gender>,
    full_name: Option<PersonNameStructure>,
    qualifying_address: Option<QualifyingAddress>,
}

impl CandidateListsCandidateBuilder {
    /// Create a new builder for building a candidate for the candidate lists document.
    pub fn new() -> Self {
        Self {
            identifier: None,
            date_of_birth: None,
            gender: None,
            full_name: None,
            qualifying_address: None,
        }
    }

    /// Set the candidate id for the candidate.
    pub fn identifier(mut self, identifier: impl Into<CandidateIdentifier>) -> Self {
        self.identifier = Some(identifier.into());
        self
    }

    /// Set the date of birth for the candidate.
    pub fn date_of_birth(mut self, date_of_birth: impl Into<XsDate>) -> Self {
        self.date_of_birth = Some(date_of_birth.into());
        self
    }

    /// Set the gender for the candidate.
    pub fn gender(mut self, gender: impl Into<Gender>) -> Self {
        self.gender = Some(gender.into());
        self
    }

    /// Set the full name for the candidate.
    pub fn full_name(mut self, full_name: impl Into<PersonNameStructure>) -> Self {
        self.full_name = Some(full_name.into());
        self
    }

    /// Set the qualifying address for the candidate.
    pub fn qualifying_address(mut self, qualifying_address: impl Into<QualifyingAddress>) -> Self {
        self.qualifying_address = Some(qualifying_address.into());
        self
    }

    /// Build the candidate, returning an error if any required fields are missing.
    pub fn build(self) -> Result<CandidateListsCandidate, EMLError> {
        Ok(CandidateListsCandidate {
            identifier: self
                .identifier
                .ok_or_else(|| EMLErrorKind::MissingBuildProperty("identifier").without_span())?,
            full_name: self
                .full_name
                .ok_or_else(|| EMLErrorKind::MissingBuildProperty("full_name").without_span())?,
            date_of_birth: self.date_of_birth.map(StringValue::from_value),
            gender: self.gender.map(StringValue::from_value),
            qualifying_address: self.qualifying_address,
        })
    }
}

impl Default for CandidateListsCandidateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EMLElement for CandidateListsCandidate {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Candidate", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        // TODO: parse Contact, Agent, kr:DateOfBirthAnnex and kr:NationalIdentificationNumber when present

        Ok(collect_struct!(elem, CandidateListsCandidate {
            identifier: CandidateIdentifier::EML_NAME => |elem| CandidateIdentifier::read_eml(elem)?,
            full_name: ("CandidateFullName", NS_EML) => |elem| PersonNameStructure::read_eml_element(elem)?,
            date_of_birth as Option: ("DateOfBirth", NS_EML) => |elem| elem.string_value()?,
            gender as Option: ("Gender", NS_EML) => |elem| elem.string_value()?,
            qualifying_address as Option: QualifyingAddress::EML_NAME => |elem| QualifyingAddress::read_eml(elem)?,
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
            .child_elem_option(
                QualifyingAddress::EML_NAME,
                self.qualifying_address.as_ref(),
            )?
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

impl QualifyingAddress {
    /// Create a new qualifying address with locality information and an optional country.
    pub fn new(
        locality: impl Into<QualifyingAddressLocality>,
        country_name_code: Option<impl Into<CountryNameCode>>,
    ) -> Self {
        match country_name_code {
            Some(code) => QualifyingAddress::Country(QualifyingAddressCountry {
                locality: locality.into(),
                country_name_code: Some(code.into()),
            }),
            None => QualifyingAddress::Locality(locality.into()),
        }
    }

    /// Get the locality information for the qualifying address.
    pub fn locality(&self) -> &QualifyingAddressLocality {
        match self {
            QualifyingAddress::Locality(locality) => locality,
            QualifyingAddress::Country(country) => &country.locality,
        }
    }

    /// Get the country information for the qualifying address, if present.
    pub fn country_name_code(&self) -> Option<&CountryNameCode> {
        match self {
            QualifyingAddress::Locality(_) => None,
            QualifyingAddress::Country(country) => country.country_name_code.as_ref(),
        }
    }
}

impl From<QualifyingAddressLocality> for QualifyingAddress {
    fn from(locality: QualifyingAddressLocality) -> Self {
        QualifyingAddress::Locality(locality)
    }
}

impl From<QualifyingAddressCountry> for QualifyingAddress {
    fn from(country: QualifyingAddressCountry) -> Self {
        QualifyingAddress::Country(country)
    }
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

    /// Get the locality name for the qualifying address locality.
    pub fn locality_name(&self) -> &str {
        &self.locality_name.name
    }

    /// Set the address line for the locality.
    pub fn with_address_line(self, address_line: impl Into<AddressLine>) -> Self {
        self.with_address_line_option(Some(address_line))
    }

    /// Set the address line for the locality, if present.
    pub fn with_address_line_option(
        mut self,
        address_line: Option<impl Into<AddressLine>>,
    ) -> Self {
        self.address_line = address_line.map(Into::into);
        self
    }

    /// Set the postal code for the locality.
    pub fn with_postal_code(self, postal_code: impl Into<PostalCode>) -> Self {
        self.with_postal_code_option(Some(postal_code))
    }

    /// Set the postal code for the locality, if present.
    pub fn with_postal_code_option(mut self, postal_code: Option<impl Into<PostalCode>>) -> Self {
        self.postal_code = postal_code.map(Into::into);
        self
    }

    /// Set the Type attribute for the locality.
    pub fn with_locality_type(self, locality_type: impl Into<String>) -> Self {
        self.with_locality_type_option(Some(locality_type))
    }

    /// Set the Type attribute for the locality, if present.
    pub fn with_locality_type_option(mut self, locality_type: Option<impl Into<String>>) -> Self {
        self.locality_type = locality_type.map(Into::into);
        self
    }

    /// Set the UsageType attribute for the locality.
    pub fn with_usage_type(self, usage_type: impl Into<String>) -> Self {
        self.with_usage_type_option(Some(usage_type))
    }

    /// Set the UsageType attribute for the locality, if present.
    pub fn with_usage_type_option(mut self, usage_type: Option<impl Into<String>>) -> Self {
        self.usage_type = usage_type.map(Into::into);
        self
    }

    /// Set the Indicator attribute for the locality.
    pub fn with_indicator(self, indicator: impl Into<String>) -> Self {
        self.with_indicator_option(Some(indicator))
    }

    /// Set the Indicator attribute for the locality, if present.
    pub fn with_indicator_option(mut self, indicator: Option<impl Into<String>>) -> Self {
        self.indicator = indicator.map(Into::into);
        self
    }
}

impl From<&str> for QualifyingAddressLocality {
    fn from(value: &str) -> Self {
        QualifyingAddressLocality::new(value)
    }
}

impl From<String> for QualifyingAddressLocality {
    fn from(value: String) -> Self {
        QualifyingAddressLocality::new(value)
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

    /// Set the Type attribute for the address line.
    pub fn with_type(mut self, address_line_type: impl Into<String>) -> Self {
        self.address_line_type = Some(address_line_type.into());
        self
    }

    /// Set the Code attribute for the address line.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

impl From<&str> for AddressLine {
    fn from(value: &str) -> Self {
        AddressLine::new(value)
    }
}

impl From<String> for AddressLine {
    fn from(value: String) -> Self {
        AddressLine::new(value)
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

/// Postal code information.
#[derive(Debug, Clone)]
pub struct PostalCode {
    /// Number of the postal code.
    pub postal_code_number: PostalCodeNumber,
}

impl PostalCode {
    /// Create a new PostalCode.
    pub fn new(postal_code_number: impl Into<PostalCodeNumber>) -> Self {
        PostalCode {
            postal_code_number: postal_code_number.into(),
        }
    }
}

impl From<&str> for PostalCode {
    fn from(value: &str) -> Self {
        PostalCode::new(value)
    }
}

impl From<String> for PostalCode {
    fn from(value: String) -> Self {
        PostalCode::new(value)
    }
}

impl From<PostalCodeNumber> for PostalCode {
    fn from(postal_code_number: PostalCodeNumber) -> Self {
        PostalCode { postal_code_number }
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

    /// Set the Type attribute for the postal code number.
    pub fn with_type(mut self, postal_code_number_type: impl Into<String>) -> Self {
        self.postal_code_number_type = Some(postal_code_number_type.into());
        self
    }

    /// Set the Code attribute for the postal code number.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

impl From<&str> for PostalCodeNumber {
    fn from(value: &str) -> Self {
        PostalCodeNumber::new(value)
    }
}

impl From<String> for PostalCodeNumber {
    fn from(value: String) -> Self {
        PostalCodeNumber::new(value)
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
        locality: impl Into<QualifyingAddressLocality>,
    ) -> Self {
        Self {
            country_name_code: country_code.map(|code| CountryNameCode::new(code)),
            locality: locality.into(),
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

#[cfg(test)]
mod tests {
    use chrono::TimeZone as _;

    use super::*;
    use crate::{
        common::PersonName,
        io::{
            EMLParsingMode, EMLRead as _, EMLWrite as _, test_write_eml_element, test_xml_fragment,
        },
        utils::{AuthorityId, CandidateId},
    };

    #[test]
    fn test_affiliation_identifier() {
        let xml = test_xml_fragment(
            r#"
            <AffiliationIdentifier xmlns="urn:oasis:names:tc:evs:schema:eml" Id="1">
                <RegisteredName>Affiliation 1</RegisteredName>
            </AffiliationIdentifier>
            "#,
        );

        let affiliation_identifier = AffiliationIdentifier::parse_eml(&xml, EMLParsingMode::Strict)
            .ok()
            .unwrap();
        assert_eq!(
            affiliation_identifier.id,
            StringValue::Parsed(AffiliationId::new(NonZeroU64::new(1).unwrap()))
        );
        assert_eq!(
            affiliation_identifier.registered_name,
            Some("Affiliation 1".to_string())
        );

        let xml_output = test_write_eml_element(&affiliation_identifier, &[NS_EML]).unwrap();
        assert_eq!(xml_output, xml);
    }

    #[test]
    fn test_empty_affiliation_identifier() {
        let xml = test_xml_fragment(
            r#"
                <AffiliationIdentifier xmlns="urn:oasis:names:tc:evs:schema:eml" Id="2">
                    <RegisteredName/>
                </AffiliationIdentifier>
            "#,
        );

        let affiliation_identifier = AffiliationIdentifier::parse_eml(&xml, EMLParsingMode::Strict)
            .ok()
            .unwrap();
        assert_eq!(
            affiliation_identifier.id,
            StringValue::Parsed(AffiliationId::new(NonZeroU64::new(2).unwrap()))
        );
        assert_eq!(affiliation_identifier.registered_name, None);

        let xml_output = test_write_eml_element(&affiliation_identifier, &[NS_EML]).unwrap();
        assert_eq!(xml_output, xml);
    }

    #[test]
    fn test_qualifying_address_full() {
        let c = QualifyingAddressCountry::new(
            Some("NL"),
            QualifyingAddressLocality::new("Amsterdam")
                .with_address_line(
                    AddressLine::new("Test 1")
                        .with_code("TestCode")
                        .with_type("TestType"),
                )
                .with_postal_code(
                    PostalCodeNumber::new("1234 AB")
                        .with_code("TestCode")
                        .with_type("TestType"),
                )
                .with_indicator("Test")
                .with_locality_type("City")
                .with_usage_type("Example"),
        );

        assert_eq!(c.country_name_code, Some(CountryNameCode::new("NL")));
        assert_eq!(c.locality.locality_name.name, "Amsterdam");
        assert_eq!(c.locality.address_line.as_ref().unwrap().value, "Test 1");
        assert_eq!(
            c.locality
                .address_line
                .as_ref()
                .unwrap()
                .code
                .as_ref()
                .unwrap(),
            "TestCode"
        );
        assert_eq!(
            c.locality
                .address_line
                .as_ref()
                .unwrap()
                .address_line_type
                .as_ref()
                .unwrap(),
            "TestType"
        );
        assert_eq!(
            c.locality
                .postal_code
                .as_ref()
                .unwrap()
                .postal_code_number
                .value,
            "1234 AB"
        );
        assert_eq!(
            c.locality
                .postal_code
                .as_ref()
                .unwrap()
                .postal_code_number
                .code
                .as_ref()
                .unwrap(),
            "TestCode"
        );
        assert_eq!(
            c.locality
                .postal_code
                .as_ref()
                .unwrap()
                .postal_code_number
                .postal_code_number_type
                .as_ref()
                .unwrap(),
            "TestType"
        );
        assert_eq!(c.locality.indicator.as_ref().unwrap(), "Test");
        assert_eq!(c.locality.locality_type.as_ref().unwrap(), "City");
        assert_eq!(c.locality.usage_type.as_ref().unwrap(), "Example");

        test_write_eml_element(&c, &[NS_XAL]).unwrap();
    }

    #[test]
    fn test_qualifying_address_parsing() {
        let xml = test_xml_fragment(
            r#"
            <QualifyingAddress xmlns="urn:oasis:names:tc:evs:schema:eml" xmlns:xal="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0">
                <xal:Country>
                    <xal:Locality>
                        <xal:LocalityName>Amsterdam</xal:LocalityName>
                    </xal:Locality>
                </xal:Country>
            </QualifyingAddress>
            "#,
        );

        let qualifying_address = QualifyingAddress::parse_eml(&xml, EMLParsingMode::Strict)
            .ok()
            .unwrap();
        match &qualifying_address {
            QualifyingAddress::Country(country) => {
                assert_eq!(country.country_name_code, None);
                assert_eq!(country.locality.locality_name.name, "Amsterdam");
            }
            _ => panic!("Expected country qualifying address"),
        }

        let xml_output = test_write_eml_element(&qualifying_address, &[NS_EML, NS_XAL]).unwrap();
        assert_eq!(xml_output, xml);
    }

    #[test]
    fn candidate_lists_construction() {
        let cl = CandidateLists::builder()
            .lists_type(CandidateListsType::Single)
            .transaction_id(TransactionId::new(1))
            .managing_authority(ManagingAuthority::new(AuthorityId::new("1234").unwrap()))
            .issue_date(XsDate::from_date(2024, 6, 10).unwrap())
            .creation_date_time(
                chrono::Utc
                    .with_ymd_and_hms(2014, 11, 28, 12, 0, 9)
                    .unwrap(),
            )
            .election_identifier(
                CandidateListsElectionIdentifier::builder()
                    .id(ElectionId::new("GR2026_Test").unwrap())
                    .category(ElectionCategory::GR)
                    .election_date(XsDate::from_date(2024, 11, 5).unwrap())
                    .nomination_date(XsDate::from_date(2024, 10, 1).unwrap())
                    .build_for_candidate_lists()
                    .unwrap(),
            )
            .contests([CandidateListsContest::builder()
                .identifier(ContestIdentifier::geen())
                .affiliations([CandidateListsAffiliation::builder()
                    .id(AffiliationId::new(NonZeroU64::new(1).unwrap()))
                    .registered_name("Affiliation 1")
                    .affiliation_type(AffiliationType::StandAloneList)
                    .publish_gender(true)
                    .candidates([CandidateListsCandidate::builder()
                        .identifier(CandidateId::new(NonZeroU64::new(1).unwrap()))
                        .full_name(
                            PersonName::new("Pietersen")
                                .with_initials("P.")
                                .with_first_name("Piet"),
                        )
                        .qualifying_address(QualifyingAddressCountry::new(Some("NL"), "Amsterdam"))
                        .build()
                        .unwrap()])
                    .build()
                    .unwrap()])
                .build()
                .unwrap()])
            .build()
            .unwrap();

        let xml = cl.write_eml_root_str(true).unwrap();

        // check if it still is the same after a second parse and write
        let parsed = CandidateLists::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        let xml2 = parsed.write_eml_root_str(true).unwrap();
        assert_eq!(xml, xml2);
    }

    #[test]
    fn test_invalid_document_type() {
        assert!(
            CandidateLists::parse_eml(
                include_str!(
                    "../../test-emls/candidate_lists/eml230b_invalid_document_type.eml.xml"
                ),
                EMLParsingMode::Strict
            )
            .ok_with_errors()
            .is_err()
        );
    }

    #[test]
    fn test_invalid_empty_affiliates() {
        assert!(
            CandidateLists::parse_eml(
                include_str!(
                    "../../test-emls/candidate_lists/eml230b_invalid_empty_affiliates.eml.xml"
                ),
                EMLParsingMode::Strict
            )
            .ok_with_errors()
            .is_err()
        );
    }

    #[test]
    fn test_invalid_empty_candidates() {
        assert!(
            CandidateLists::parse_eml(
                include_str!(
                    "../../test-emls/candidate_lists/eml230b_invalid_empty_candidates.eml.xml"
                ),
                EMLParsingMode::Strict
            )
            .ok_with_errors()
            .is_err()
        );
    }

    #[test]
    fn test_invalid_incorrect_election_date() {
        assert!(
            CandidateLists::parse_eml(
                include_str!(
                    "../../test-emls/candidate_lists/eml230b_invalid_incorrect_election_date.eml.xml"
                ),
                EMLParsingMode::Strict
            )
            .ok_with_errors()
            .is_err()
        );
    }

    #[test]
    fn test_incorrect_election_domain() {
        assert!(
            CandidateLists::parse_eml(
                include_str!(
                    "../../test-emls/candidate_lists/eml230b_invalid_incorrect_election_domain.eml.xml"
                ),
                EMLParsingMode::Strict
            )
            .ok_with_errors()
            .is_err()
        );
    }

    #[test]
    fn test_incorrect_election_category() {
        assert!(
            CandidateLists::parse_eml(
                include_str!(
                    "../../test-emls/candidate_lists/eml230b_invalid_incorrect_election_category.eml.xml"
                ),
                EMLParsingMode::Strict
            )
            .ok_with_errors()
            .is_err()
        );
    }

    #[test]
    fn test_incorrect_missing_authority() {
        assert!(
            CandidateLists::parse_eml(
                include_str!(
                    "../../test-emls/candidate_lists/eml230b_invalid_missing_authority.eml.xml"
                ),
                EMLParsingMode::Strict
            )
            .ok_with_errors()
            .is_err()
        );
    }

    #[test]
    fn test_with_missing_addresses() {
        assert!(
            CandidateLists::parse_eml(
                include_str!(
                    "../../test-emls/candidate_lists/eml230b_test_without_addresses.eml.xml"
                ),
                EMLParsingMode::Strict
            )
            .ok_with_errors()
            .is_ok()
        );
    }
}
