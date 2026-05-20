//! Document variant for the EML_NL Candidate List (`230b`) document.

use std::{fmt, num::NonZeroU64, str::FromStr};

use instant_xml::{Accumulate, FromXml, ToXml, ser::Context};

use crate::{
    EML_SCHEMA_VERSION, EMLError, NS_EML, NS_KR, NS_XAL,
    common::{
        CandidateIdentifier, CanonicalizationMethod, ContestIdentifier, CountryNameCode,
        CreationDateTime, ElectionDomain, IssueDate, ListData, ListDataBelongsToCombination,
        LocalityName, ManagingAuthority, PersonNameStructure, TransactionId,
    },
    documents::ElectionIdentifierBuilder,
    eml_ns_context,
    error::EMLErrorKind,
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
        Self::parse_eml(s).ok()
    }
}

impl TryFrom<&str> for CandidateLists {
    type Error = EMLError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use crate::io::EMLRead as _;
        Self::parse_eml(value).ok()
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

// Custom: root EML element with dynamic Id attribute and full namespace context.
impl<'xml> FromXml<'xml> for CandidateLists {
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
        use instant_xml::{Error, de::Node};

        if into.is_some() {
            return Err(Error::DuplicateValue(field));
        }

        let mut transaction_id = <TransactionId as FromXml>::Accumulator::default();
        let mut managing_authority = <ManagingAuthority as FromXml>::Accumulator::default();
        let mut issue_date = <IssueDate as FromXml>::Accumulator::default();
        let mut creation_date_time = <CreationDateTime as FromXml>::Accumulator::default();
        let mut canonicalization_method =
            <CanonicalizationMethod as FromXml>::Accumulator::default();
        let mut candidate_list = <CandidateListsCandidateList as FromXml>::Accumulator::default();

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
            } else if CandidateListsCandidateList::matches(id, None) {
                let mut nested = deserializer.nested(element);
                CandidateListsCandidateList::deserialize(&mut candidate_list, field, &mut nested)?;
                nested.ignore()?;
            } else {
                let mut nested = deserializer.nested(element);
                nested.ignore()?;
            }
        }

        *into = Some(CandidateLists {
            lists_type: CandidateListsType::Single, // Default; overridden by EML dispatch
            transaction_id: transaction_id.try_done(field)?,
            managing_authority: managing_authority.try_done(field)?,
            issue_date: issue_date.try_done(field)?,
            creation_date_time: creation_date_time.try_done(field)?,
            canonicalization_method,
            candidate_list: candidate_list.try_done(field)?,
        });
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Element;
}

// Custom: root EML element with dynamic Id attribute and full namespace context.
impl ToXml for CandidateLists {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix = serializer.write_start("EML", NS_EML, Some(eml_ns_context()))?;
        serializer.write_attr("Id", "", self.lists_type.to_eml_id())?;
        serializer.write_attr("SchemaVersion", "", EML_SCHEMA_VERSION)?;
        serializer.end_start()?;
        self.transaction_id.serialize(None, serializer)?;
        self.managing_authority.serialize(None, serializer)?;
        self.issue_date.serialize(None, serializer)?;
        self.creation_date_time.serialize(None, serializer)?;
        self.candidate_list.serialize(None, serializer)?;
        serializer.write_close(prefix)
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
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "CandidateList", ns(NS_EML))]
pub struct CandidateListsCandidateList {
    /// The date of the candidate list, if present.
    #[xml(rename = "ListDate")]
    pub list_date: Option<CandidateListsListDate>,

    /// The election information.
    #[xml(rename = "Election")]
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

/// The date of the candidate list.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "ListDate", ns(NS_EML))]
pub struct CandidateListsListDate(pub StringValue<XsDateOrDateTime>);

impl From<XsDateOrDateTime> for CandidateListsListDate {
    fn from(value: XsDateOrDateTime) -> Self {
        CandidateListsListDate(StringValue::from_value(value))
    }
}

/// The election information in the candidate lists.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "Election", ns(NS_EML))]
pub struct CandidateListsElection {
    /// Identifier for the election.
    #[xml(rename = "ElectionIdentifier")]
    pub identifier: CandidateListsElectionIdentifier,
    /// Election contest details.
    #[xml(rename = "Contest")]
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

/// Identifier for the election.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "ElectionIdentifier", ns(NS_EML, kr = NS_KR))]
pub struct CandidateListsElectionIdentifier {
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

impl CandidateListsElectionIdentifier {
    /// Create a new Election Identifier builder
    pub fn builder() -> ElectionIdentifierBuilder {
        ElectionIdentifierBuilder::new()
    }
}

/// Election contest details.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "Contest", ns(NS_EML))]
pub struct CandidateListsContest {
    /// Identifier for the contest.
    #[xml(rename = "ContestIdentifier")]
    pub identifier: ContestIdentifier,

    /// Affiliations participating in the contest.
    #[xml(rename = "Affiliation")]
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

// Custom: unwraps `<AffiliationIdentifier>` sub-element with Id attribute into struct fields.
impl<'xml> FromXml<'xml> for CandidateListsAffiliation {
    fn matches(id: instant_xml::Id<'_>, _field: Option<instant_xml::Id<'_>>) -> bool {
        id == instant_xml::Id {
            ns: NS_EML,
            name: "Affiliation",
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut instant_xml::Deserializer<'cx, 'xml>,
    ) -> Result<(), instant_xml::Error> {
        use instant_xml::{Error, de::Node};

        if into.is_some() {
            return Err(Error::DuplicateValue(field));
        }

        let mut identifier = <AffiliationIdentifier as FromXml>::Accumulator::default();
        let mut affiliation_type =
            <StringValue<AffiliationType> as FromXml>::Accumulator::default();
        let mut list_data = <ListData as FromXml>::Accumulator::default();
        let mut candidates = <Vec<CandidateListsCandidate> as FromXml>::Accumulator::default();

        while let Some(node) = deserializer.next() {
            let element = match node? {
                Node::Open(element) => element,
                Node::Text(s) if s.trim().is_empty() => continue,
                node => return Err(Error::UnexpectedNode(format!("{node:?}"))),
            };

            let id = deserializer.element_id(&element)?;
            if AffiliationIdentifier::matches(id, None) {
                let mut nested = deserializer.nested(element);
                AffiliationIdentifier::deserialize(&mut identifier, field, &mut nested)?;
                nested.ignore()?;
            } else if id
                == (instant_xml::Id {
                    ns: NS_EML,
                    name: "Type",
                })
            {
                let mut nested = deserializer.nested(element);
                <StringValue<AffiliationType>>::deserialize(
                    &mut affiliation_type,
                    field,
                    &mut nested,
                )?;
                nested.ignore()?;
            } else if ListData::matches(id, None) {
                let mut nested = deserializer.nested(element);
                ListData::deserialize(&mut list_data, field, &mut nested)?;
                nested.ignore()?;
            } else if <Vec<CandidateListsCandidate> as FromXml>::matches(id, None) {
                let mut nested = deserializer.nested(element);
                <Vec<CandidateListsCandidate> as FromXml>::deserialize(
                    &mut candidates,
                    field,
                    &mut nested,
                )?;
                nested.ignore()?;
            } else {
                let mut nested = deserializer.nested(element);
                nested.ignore()?;
            }
        }

        *into = Some(CandidateListsAffiliation {
            identifier: identifier.try_done(field)?,
            affiliation_type: affiliation_type.try_done(field)?,
            list_data: list_data.try_done(field)?,
            candidates: candidates.try_done(field)?,
        });
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Element;
}

// Custom: wraps identifier fields in `<AffiliationIdentifier>` sub-element with Id attribute.
impl ToXml for CandidateListsAffiliation {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix = serializer.write_start("Affiliation", NS_EML, None::<Context<0>>)?;
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

/// An affiliation identifier consisting of an id and a registered name.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(
    rename = "AffiliationIdentifier",
    rename_all = "PascalCase",
    ns(NS_EML)
)]
pub struct AffiliationIdentifier {
    /// The affiliation id.
    #[xml(attribute)]
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

/// A candidate in an affiliation.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "Candidate", ns(NS_EML))]
pub struct CandidateListsCandidate {
    /// The candidate identifier.
    #[xml(rename = "CandidateIdentifier")]
    pub identifier: CandidateIdentifier,

    /// The full name of the candidate.
    #[xml(rename = "CandidateFullName")]
    pub full_name: PersonNameStructure,

    /// The date of birth of the candidate, if present.
    #[xml(rename = "DateOfBirth")]
    pub date_of_birth: Option<StringValue<XsDate>>,

    /// The gender of the candidate, if present.
    #[xml(rename = "Gender")]
    pub gender: Option<StringValue<Gender>>,

    /// The qualifying address of the candidate.
    #[xml(rename = "QualifyingAddress")]
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

// Custom: enum dispatch (Locality/Country variants) inside a `<QualifyingAddress>` element.
impl<'xml> FromXml<'xml> for QualifyingAddress {
    fn matches(id: instant_xml::Id<'_>, field: Option<instant_xml::Id<'_>>) -> bool {
        match field {
            Some(field) => id == field,
            None => {
                id == instant_xml::Id {
                    ns: NS_EML,
                    name: "QualifyingAddress",
                }
            }
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut instant_xml::Deserializer<'cx, 'xml>,
    ) -> Result<(), instant_xml::Error> {
        use instant_xml::{Error, de::Node};

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
                *into = Some(QualifyingAddress::Locality(acc.try_done(field)?));
            } else if QualifyingAddressCountry::matches(id, None) {
                let mut acc = <QualifyingAddressCountry as FromXml<'xml>>::Accumulator::default();
                let mut nested = deserializer.nested(element);
                QualifyingAddressCountry::deserialize(&mut acc, field, &mut nested)?;
                nested.ignore()?;
                *into = Some(QualifyingAddress::Country(acc.try_done(field)?));
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

// Custom: enum dispatch (Locality/Country variants) inside a `<QualifyingAddress>` element.
impl ToXml for QualifyingAddress {
    fn serialize<W: fmt::Write + ?::core::marker::Sized>(
        &self,
        _field: Option<::instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<W>,
    ) -> ::std::result::Result<(), instant_xml::Error> {
        let addr = serializer.write_start("QualifyingAddress", NS_EML, None::<Context<0>>)?;
        serializer.end_start()?;
        match self {
            QualifyingAddress::Locality(inner) => inner.serialize(None, serializer)?,
            QualifyingAddress::Country(inner) => inner.serialize(None, serializer)?,
        }
        serializer.write_close(addr)
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

/// Qualifying address locality.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "Locality", ns(NS_XAL), force_prefix)]
pub struct QualifyingAddressLocality {
    /// The Type attribute, if present.
    #[xml(attribute, rename = "Type")]
    pub locality_type: Option<String>,

    /// The UsageType attribute, if present.
    #[xml(attribute, rename = "UsageType")]
    pub usage_type: Option<String>,

    /// The Indicator attribute, if present.
    #[xml(attribute, rename = "Indicator")]
    pub indicator: Option<String>,

    /// The address line, if present.
    #[xml(rename = "AddressLine")]
    pub address_line: Option<AddressLine>,

    /// The locality name.
    #[xml(rename = "LocalityName")]
    pub locality_name: LocalityName,

    /// The postal code, if present.
    #[xml(rename = "PostalCode")]
    pub postal_code: Option<PostalCode>,
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

/// Address line information.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "AddressLine", ns(NS_XAL), force_prefix)]
pub struct AddressLine {
    /// The Type attribute, if present.
    #[xml(attribute, rename = "Type")]
    pub address_line_type: Option<String>,

    /// The Code attribute, if present.
    #[xml(attribute, rename = "Code")]
    pub code: Option<String>,

    /// The address line value.
    #[xml(direct)]
    pub value: String,
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

/// Postal code information.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "PostalCode", ns(NS_XAL), force_prefix)]
pub struct PostalCode {
    /// Number of the postal code.
    #[xml(rename = "PostalCodeNumber")]
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

/// The postal code number.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "PostalCodeNumber", ns(NS_XAL), force_prefix)]
pub struct PostalCodeNumber {
    /// The Type attribute, if present.
    #[xml(attribute, rename = "Type")]
    pub postal_code_number_type: Option<String>,

    /// The Code attribute, if present.
    #[xml(attribute, rename = "Code")]
    pub code: Option<String>,

    /// The postal code number value.
    #[xml(direct)]
    pub value: String,
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
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "Country", ns(NS_XAL), force_prefix)]
pub struct QualifyingAddressCountry {
    /// The country name code, if present.
    #[xml(rename = "CountryNameCode")]
    pub country_name_code: Option<CountryNameCode>,
    /// The locality within the country.
    #[xml(rename = "Locality")]
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

#[cfg(test)]
mod tests {
    use chrono::TimeZone as _;

    use super::*;
    use crate::{
        common::PersonName,
        documents::EML,
        io::{EMLRead as _, EMLWrite as _, test_xml_fragment},
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

        let affiliation_identifier = AffiliationIdentifier::parse_eml(&xml).ok().unwrap();
        assert_eq!(
            affiliation_identifier.id,
            StringValue::Parsed(AffiliationId::new(NonZeroU64::new(1).unwrap()))
        );
        assert_eq!(
            affiliation_identifier.registered_name,
            Some("Affiliation 1".to_string())
        );
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

        let affiliation_identifier = AffiliationIdentifier::parse_eml(&xml).ok().unwrap();
        assert_eq!(
            affiliation_identifier.id,
            StringValue::Parsed(AffiliationId::new(NonZeroU64::new(2).unwrap()))
        );
        assert_eq!(affiliation_identifier.registered_name, Some(String::new()));
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

        let qualifying_address = QualifyingAddress::parse_eml(&xml).ok().unwrap();
        match &qualifying_address {
            QualifyingAddress::Country(country) => {
                assert_eq!(country.country_name_code, None);
                assert_eq!(country.locality.locality_name.name, "Amsterdam");
            }
            _ => panic!("Expected country qualifying address"),
        }
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
        let eml = EML::parse_eml(&xml).unwrap();
        let parsed = eml
            .as_candidate_lists_doc()
            .expect("expected candidate lists variant");
        let xml2 = parsed.write_eml_root_str(true).unwrap();
        assert_eq!(xml, xml2);
    }

    #[test]
    #[ignore = "post-parse validation not yet reimplemented"]
    fn test_invalid_document_type() {
        assert!(
            CandidateLists::parse_eml(include_str!(
                "../../test-emls/candidate_lists/eml230b_invalid_document_type.eml.xml"
            ),)
            .ok_with_errors()
            .is_err()
        );
    }

    #[test]
    #[ignore = "post-parse validation not yet reimplemented"]
    fn test_invalid_empty_affiliates() {
        assert!(
            CandidateLists::parse_eml(include_str!(
                "../../test-emls/candidate_lists/eml230b_invalid_empty_affiliates.eml.xml"
            ),)
            .ok_with_errors()
            .is_err()
        );
    }

    #[test]
    #[ignore = "post-parse validation not yet reimplemented"]
    fn test_invalid_empty_candidates() {
        assert!(
            CandidateLists::parse_eml(include_str!(
                "../../test-emls/candidate_lists/eml230b_invalid_empty_candidates.eml.xml"
            ),)
            .ok_with_errors()
            .is_err()
        );
    }

    #[test]
    #[ignore = "post-parse validation not yet reimplemented"]
    fn test_invalid_incorrect_election_date() {
        assert!(
            CandidateLists::parse_eml(include_str!(
                "../../test-emls/candidate_lists/eml230b_invalid_incorrect_election_date.eml.xml"
            ),)
            .ok_with_errors()
            .is_err()
        );
    }

    #[test]
    #[ignore = "post-parse validation not yet reimplemented"]
    fn test_incorrect_election_domain() {
        assert!(
            CandidateLists::parse_eml(include_str!(
                "../../test-emls/candidate_lists/eml230b_invalid_incorrect_election_domain.eml.xml"
            ),)
            .ok_with_errors()
            .is_err()
        );
    }

    #[test]
    #[ignore = "post-parse validation not yet reimplemented"]
    fn test_incorrect_election_category() {
        assert!(
            CandidateLists::parse_eml(
                include_str!(
                    "../../test-emls/candidate_lists/eml230b_invalid_incorrect_election_category.eml.xml"
                ),
            )
            .ok_with_errors()
            .is_err()
        );
    }

    #[test]
    fn test_incorrect_missing_authority() {
        assert!(
            CandidateLists::parse_eml(include_str!(
                "../../test-emls/candidate_lists/eml230b_invalid_missing_authority.eml.xml"
            ),)
            .ok_with_errors()
            .is_err()
        );
    }

    #[test]
    fn test_with_missing_addresses() {
        let eml = EML::parse_eml(include_str!(
            "../../test-emls/candidate_lists/eml230b_test_without_addresses.eml.xml"
        ))
        .ok_with_errors();
        assert!(eml.unwrap().0.as_candidate_lists_doc().is_some());
    }
}
