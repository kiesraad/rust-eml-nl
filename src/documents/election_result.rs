//! Document variant for the EML_NL Result (`520`) document.

use std::{ops::Deref, str::FromStr};

use crate::{
    EML_SCHEMA_VERSION, EMLError, EMLErrorKind, EMLResultExt as _, NS_EML, NS_KR,
    common::{
        CandidateIdentifier, CanonicalizationMethod, ContestIdentifier, CreationDateTime,
        ElectionDomain, ManagingAuthority, MinimalQualifyingAddress, PersonNameStructure,
        TransactionId,
    },
    documents::{ElectionIdentifierBuilder, accepted_root},
    io::{
        EMLElement, EMLElementReader, EMLElementWriter, EMLReadElement as _, EMLWriteElement as _,
        QualifiedName, collect_struct,
    },
    utils::{
        AffiliationId, ElectionCategory, ElectionId, ElectionSubcategory, Gender, StringValue,
        StringValueData, XsDate, XsDateTime,
    },
};

pub(crate) const EML_ELECTION_RESULT_ID: &str = "520";

/// Representing a `110a` document, containing an election definition.
#[derive(Debug, Clone)]
pub struct ElectionResult {
    /// Transaction id of the document.
    pub transaction_id: TransactionId,

    /// Managing authority of the election, if present.
    pub managing_authority: ManagingAuthority,

    /// Time this document was created.
    pub creation_date_time: CreationDateTime,

    /// Canonicalization method used in this document, if present.
    pub canonicalization_method: Option<CanonicalizationMethod>,

    /// The result data.
    pub result: ElectionResultResult,
}

impl ElectionResult {
    /// Builder for creating a new instance.
    pub fn builder() -> ElectionResultBuilder {
        ElectionResultBuilder::new()
    }
}

impl FromStr for ElectionResult {
    type Err = EMLError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::io::EMLRead as _;
        Self::parse_eml(s, crate::io::EMLParsingMode::Strict).ok()
    }
}

impl TryFrom<&str> for ElectionResult {
    type Error = EMLError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use crate::io::EMLRead as _;
        Self::parse_eml(value, crate::io::EMLParsingMode::Strict).ok()
    }
}

impl TryFrom<ElectionResult> for String {
    type Error = EMLError;

    fn try_from(value: ElectionResult) -> Result<Self, Self::Error> {
        use crate::io::EMLWrite as _;
        value.write_eml_root_str(true, true)
    }
}

/// Builder for [`ElectionResult`].
#[derive(Debug, Clone)]
pub struct ElectionResultBuilder {
    transaction_id: Option<TransactionId>,
    managing_authority: Option<ManagingAuthority>,
    creation_date_time: Option<CreationDateTime>,
    canonicalization_method: Option<CanonicalizationMethod>,
    result: Option<ElectionResultResult>,
    election_identifier: Option<ElectionResultElectionIdentifier>,
    contests: Vec<ElectionResultContest>,
}

impl ElectionResultBuilder {
    /// Create a new builder for [`ElectionResult`].
    pub fn new() -> Self {
        Self {
            transaction_id: None,
            managing_authority: None,
            creation_date_time: None,
            canonicalization_method: None,
            result: None,
            election_identifier: None,
            contests: vec![],
        }
    }

    /// Set the transaction id of the election result document.
    pub fn transaction_id(mut self, transaction_id: impl Into<TransactionId>) -> Self {
        self.transaction_id = Some(transaction_id.into());
        self
    }

    /// Set the managing authority of the election result document.
    pub fn managing_authority(mut self, managing_authority: impl Into<ManagingAuthority>) -> Self {
        self.managing_authority = Some(managing_authority.into());
        self
    }

    /// Set the creation date and time of the election result document.
    pub fn creation_date_time(mut self, creation_date_time: impl Into<XsDateTime>) -> Self {
        self.creation_date_time = Some(CreationDateTime::new(creation_date_time.into()));
        self
    }

    /// Set the canonicalization method used in the election result document.
    pub fn canonicalization_method(
        mut self,
        canonicalization_method: impl Into<CanonicalizationMethod>,
    ) -> Self {
        self.canonicalization_method = Some(canonicalization_method.into());
        self
    }

    /// Set the result data of the election result document.
    ///
    /// You can use this to set the entire result at once, or you can use the
    /// [`Self::election_identifier`], [`Self::contests`] and/or [`Self::push_contest`]
    /// methods to set the election identifier and contests separately and let
    /// this builder build the result data for you.
    pub fn result(mut self, result: impl Into<ElectionResultResult>) -> Self {
        self.result = Some(result.into());
        self
    }

    /// Set the election identifier for the election result document.
    ///
    /// Has no effect if the result is already set using the [`Self::result`] method.
    pub fn election_identifier(
        mut self,
        election_identifier: impl Into<ElectionResultElectionIdentifier>,
    ) -> Self {
        self.election_identifier = Some(election_identifier.into());
        self
    }

    /// Set the contests for the election result document, this overrides any previously set contests.
    ///
    /// Has no effect if the result is already set using the [`Self::result`] method.
    pub fn contests(mut self, contests: impl Into<Vec<ElectionResultContest>>) -> Self {
        self.contests = contests.into();
        self
    }

    /// Add a contest to the election result document.
    ///
    /// Has no effect if the result is already set using the [`Self::result`] method.
    pub fn push_contest(mut self, contest: impl Into<ElectionResultContest>) -> Self {
        self.contests.push(contest.into());
        self
    }

    /// Build the [`ElectionResult`] from the provided data, returning an error if any required fields are missing.
    pub fn build(self) -> Result<ElectionResult, EMLError> {
        Ok(ElectionResult {
            transaction_id: self.transaction_id.ok_or_else(|| {
                EMLErrorKind::MissingBuildProperty("transaction_id").without_span()
            })?,
            managing_authority: self.managing_authority.ok_or_else(|| {
                EMLErrorKind::MissingBuildProperty("managing_authority").without_span()
            })?,
            creation_date_time: self.creation_date_time.ok_or_else(|| {
                EMLErrorKind::MissingBuildProperty("creation_date_time").without_span()
            })?,
            canonicalization_method: self.canonicalization_method,
            result: self.result.map_or_else(
                || {
                    let election_identifier = self.election_identifier.ok_or_else(|| {
                        EMLErrorKind::MissingBuildProperty("election_identifier").without_span()
                    })?;

                    if self.contests.is_empty() {
                        return Err(EMLErrorKind::MissingBuildProperty("contests").without_span());
                    }
                    Ok(ElectionResultResult::new(ElectionResultElection::new(
                        election_identifier,
                        self.contests,
                    )))
                },
                Ok,
            )?,
        })
    }
}

impl Default for ElectionResultBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EMLElement for ElectionResult {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("EML", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        // TODO: parse the rest of the document
        accepted_root(elem)?;

        let document_id = elem.attribute_value_req(("Id", None))?;
        if document_id != EML_ELECTION_RESULT_ID {
            return Err(EMLErrorKind::InvalidDocumentType(
                EML_ELECTION_RESULT_ID,
                document_id.to_string(),
            ))
            .with_span(elem.span());
        }

        Ok(collect_struct!(elem, ElectionResult {
            transaction_id: TransactionId::EML_NAME => |elem| TransactionId::read_eml(elem)?,
            managing_authority: ManagingAuthority::EML_NAME => |elem| ManagingAuthority::read_eml(elem)?,
            creation_date_time: CreationDateTime::EML_NAME => |elem| CreationDateTime::read_eml(elem)?,
            canonicalization_method as Option: CanonicalizationMethod::EML_NAME => |elem| CanonicalizationMethod::read_eml(elem)?,
            result: ElectionResultResult::EML_NAME => |elem| ElectionResultResult::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr(("Id", None), EML_ELECTION_RESULT_ID)?
            .attr(("SchemaVersion", None), EML_SCHEMA_VERSION)?
            .child_elem(TransactionId::EML_NAME, &self.transaction_id)?
            .child_elem(ManagingAuthority::EML_NAME, &self.managing_authority)?
            .child_elem(CreationDateTime::EML_NAME, &self.creation_date_time)?
            // Note: we don't output the CanonicalizationMethod because we aren't canonicalizing our output
            // .child_elem_option(
            //     CanonicalizationMethod::EML_NAME,
            //     self.canonicalization_method.as_ref(),
            // )?
            .child_elem(ElectionResultResult::EML_NAME, &self.result)?
            .finish()
    }
}

/// The result data of an election result document.
#[derive(Debug, Clone)]
pub struct ElectionResultResult {
    /// The election for which the result applies.
    pub election: ElectionResultElection,
}

impl ElectionResultResult {
    /// Create a new result from the given election.
    pub fn new(election: impl Into<ElectionResultElection>) -> Self {
        ElectionResultResult {
            election: election.into(),
        }
    }
}

impl EMLElement for ElectionResultResult {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Result", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, ElectionResultResult {
            election: ElectionResultElection::EML_NAME => |elem| ElectionResultElection::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(ElectionResultElection::EML_NAME, &self.election)?
            .finish()
    }
}

/// The election for which the result applies.
#[derive(Debug, Clone)]
pub struct ElectionResultElection {
    /// Identifier for the election.
    pub identifier: ElectionResultElectionIdentifier,

    /// Contests within the election.
    pub contests: Vec<ElectionResultContest>,
}

impl ElectionResultElection {
    /// Create a new election result election from the given identifier and contests.
    pub fn new(
        identifier: impl Into<ElectionResultElectionIdentifier>,
        contests: impl Into<Vec<ElectionResultContest>>,
    ) -> Self {
        ElectionResultElection {
            identifier: identifier.into(),
            contests: contests.into(),
        }
    }
}

impl EMLElement for ElectionResultElection {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Election", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, ElectionResultElection {
            identifier: ElectionResultElectionIdentifier::EML_NAME => |elem| ElectionResultElectionIdentifier::read_eml(elem)?,
            contests as Vec: ElectionResultContest::EML_NAME => |elem| ElectionResultContest::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(ElectionResultElectionIdentifier::EML_NAME, &self.identifier)?
            .child_elems(ElectionResultContest::EML_NAME, &self.contests)?
            .finish()
    }
}

/// Identifier for the election for which the result applies.
#[derive(Debug, Clone)]
pub struct ElectionResultElectionIdentifier {
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
}

impl ElectionResultElectionIdentifier {
    /// Builder for creating a new instance.
    pub fn builder() -> ElectionIdentifierBuilder {
        ElectionIdentifierBuilder::new()
    }
}

impl EMLElement for ElectionResultElectionIdentifier {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ElectionIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(
            elem,
            ElectionResultElectionIdentifier {
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
            .finish()
    }
}

/// A contest within an election result.
#[derive(Debug, Clone)]
pub struct ElectionResultContest {
    /// Identifier of the contest.
    pub identifier: ContestIdentifier,

    /// Selections within the contest.
    pub selections: Vec<ElectionResultSelection>,
}

impl ElectionResultContest {
    /// Create a new election result contest from the given identifier and selections.
    pub fn new(
        identifier: impl Into<ContestIdentifier>,
        selections: impl Into<Vec<ElectionResultSelection>>,
    ) -> Self {
        ElectionResultContest {
            identifier: identifier.into(),
            selections: selections.into(),
        }
    }
}

impl EMLElement for ElectionResultContest {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Contest", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, ElectionResultContest {
            identifier: ContestIdentifier::EML_NAME => |elem| ContestIdentifier::read_eml(elem)?,
            selections as Vec: ElectionResultSelection::EML_NAME => |elem| ElectionResultSelection::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(ContestIdentifier::EML_NAME, &self.identifier)?
            .child_elems(ElectionResultSelection::EML_NAME, &self.selections)?
            .finish()
    }
}

/// The ranking of a selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RankingType {
    /// First ranked selection.
    First,
    /// Second ranked selection.
    Second,
}

impl RankingType {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "1" => Some(RankingType::First),
            "2" => Some(RankingType::Second),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            RankingType::First => "1",
            RankingType::Second => "2",
        }
    }
}

/// Error type for invalid ranking type values.
#[derive(Debug, Clone, thiserror::Error)]
#[error("Invalid ranking type value: {0}")]
pub struct InvalidRankingTypeError(String);

impl StringValueData for RankingType {
    type Error = InvalidRankingTypeError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error> {
        RankingType::from_str(s).ok_or_else(|| InvalidRankingTypeError(s.to_string()))
    }

    fn to_raw_value(&self) -> Box<str> {
        self.as_str().into()
    }
}

/// Yes/no type, represented as a boolean.
/// In the EML, this is represented as "yes" or "no".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct YesNoType(bool);

impl Deref for YesNoType {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl YesNoType {
    /// Create a new YesNoType from a boolean value.
    pub fn new(value: bool) -> Self {
        YesNoType(value)
    }

    /// Create a new YesNoType from an EML string ("yes" or "no").
    pub fn from_eml_value(s: &str) -> Result<Self, InvalidYesNoTypeError> {
        match s {
            "yes" => Ok(YesNoType(true)),
            "no" => Ok(YesNoType(false)),
            _ => Err(InvalidYesNoTypeError(s.to_string())),
        }
    }

    /// Returns the string representation of the value ("yes" or "no").
    pub fn to_eml_value(&self) -> &'static str {
        if self.0 { "yes" } else { "no" }
    }

    /// Returns `true` if the value is "yes".
    pub fn is_yes(&self) -> bool {
        self.0
    }

    /// Returns `true` if the value is "no".
    pub fn is_no(&self) -> bool {
        !self.0
    }
}

impl From<bool> for YesNoType {
    fn from(value: bool) -> Self {
        YesNoType::new(value)
    }
}

/// Error type for invalid yes/no type values.
#[derive(Debug, Clone, thiserror::Error)]
#[error("Invalid yes/no type value: {0}")]
pub struct InvalidYesNoTypeError(String);

impl StringValueData for YesNoType {
    type Error = InvalidYesNoTypeError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error> {
        Self::from_eml_value(s)
    }

    fn to_raw_value(&self) -> Box<str> {
        self.to_eml_value().into()
    }
}

/// A selection within an election contest.
#[derive(Debug, Clone)]
pub struct ElectionResultSelection {
    /// The type of selection.
    pub selection_type: ElectionResultSelectionType,

    /// Number of votes received.
    pub votes: Option<StringValue<u64>>,

    /// Ranking of the selection, if applicable.
    pub ranking: Option<StringValue<RankingType>>,

    /// Whether the selection was elected.
    pub elected: StringValue<YesNoType>,
}

impl ElectionResultSelection {
    /// Create a builder for creating a new [`ElectionResultSelection`] instance.
    pub fn builder() -> ElectionResultSelectionBuilder {
        ElectionResultSelectionBuilder::new()
    }
}

/// Builder for [`ElectionResultSelection`].
#[derive(Debug, Clone)]
pub struct ElectionResultSelectionBuilder {
    selection_type: Option<ElectionResultSelectionType>,
    votes: Option<StringValue<u64>>,
    ranking: Option<StringValue<RankingType>>,
    elected: Option<StringValue<YesNoType>>,
}

impl ElectionResultSelectionBuilder {
    /// Create a new builder for [`ElectionResultSelection`].
    pub fn new() -> Self {
        Self {
            selection_type: None,
            votes: None,
            ranking: None,
            elected: None,
        }
    }

    /// Set the selection type to a candidate selection with the given candidate.
    pub fn candidate(mut self, candidate: impl Into<CandidateSelection>) -> Self {
        self.selection_type = Some(ElectionResultSelectionType::Candidate(Box::new(
            candidate.into(),
        )));
        self
    }

    /// Set the selection type to an affiliation selection with the given affiliation.
    pub fn affiliation(mut self, affiliation: impl Into<AffiliationSelection>) -> Self {
        self.selection_type = Some(ElectionResultSelectionType::Affiliation(Box::new(
            affiliation.into(),
        )));
        self
    }

    /// Set the number of votes received for this selection.
    pub fn votes(mut self, votes: impl Into<u64>) -> Self {
        self.votes = Some(StringValue::from_value(votes.into()));
        self
    }

    /// Set the ranking of this selection.
    pub fn ranking(mut self, ranking: impl Into<RankingType>) -> Self {
        self.ranking = Some(StringValue::from_value(ranking.into()));
        self
    }

    /// Set whether this selection was elected.
    pub fn elected(mut self, elected: impl Into<YesNoType>) -> Self {
        self.elected = Some(StringValue::from_value(elected.into()));
        self
    }

    /// Build the [`ElectionResultSelection`] from the provided data, returning an error if any required fields are missing.
    pub fn build(self) -> Result<ElectionResultSelection, EMLError> {
        Ok(ElectionResultSelection {
            selection_type: self.selection_type.ok_or_else(|| {
                EMLErrorKind::MissingBuildProperty("selection_type").without_span()
            })?,
            votes: self.votes,
            ranking: self.ranking,
            elected: self
                .elected
                .ok_or_else(|| EMLErrorKind::MissingBuildProperty("elected").without_span())?,
        })
    }
}

impl Default for ElectionResultSelectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

const VOTES_EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Votes", Some(NS_EML));
const RANKING_EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Ranking", Some(NS_EML));
const ELECTED_EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Elected", Some(NS_EML));

impl EMLElement for ElectionResultSelection {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Selection", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let mut selection_type = None;
        let mut votes = None;
        let mut ranking = None;
        let mut elected = None;

        while let Some(mut child) = elem.next_child()? {
            let name = child.name()?;

            match name {
                n if n == CandidateSelection::EML_NAME => {
                    selection_type = Some(ElectionResultSelectionType::Candidate(Box::new(
                        CandidateSelection::read_eml(&mut child)?,
                    )));
                }
                n if n == AffiliationSelection::EML_NAME => {
                    selection_type = Some(ElectionResultSelectionType::Affiliation(Box::new(
                        AffiliationSelection::read_eml(&mut child)?,
                    )));
                }
                n if n == VOTES_EML_NAME => {
                    votes = Some(child.string_value()?);
                }
                n if n == RANKING_EML_NAME => {
                    ranking = Some(child.string_value()?);
                }
                n if n == ELECTED_EML_NAME => {
                    elected = Some(child.string_value()?);
                }
                n => {
                    let err =
                        EMLErrorKind::UnexpectedElement(n.as_owned(), Self::EML_NAME.as_owned())
                            .with_span(child.inner_span());
                    if child.parsing_mode().is_strict() {
                        return Err(err);
                    } else {
                        child.push_err(err);
                        child.skip()?;
                    }
                }
            }
        }

        let elected = elected.unwrap_or_else(|| StringValue::from_value(YesNoType::new(false)));

        Ok(ElectionResultSelection {
            selection_type: selection_type
                .ok_or_else(|| EMLErrorKind::MissingSelectionType.with_span(elem.inner_span()))?,
            votes,
            ranking,
            elected,
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        let writer = match &self.selection_type {
            ElectionResultSelectionType::Candidate(candidate_selection) => {
                writer.child_elem(CandidateSelection::EML_NAME, candidate_selection.as_ref())?
            }
            ElectionResultSelectionType::Affiliation(affiliation_selection) => writer.child_elem(
                AffiliationSelection::EML_NAME,
                affiliation_selection.as_ref(),
            )?,
        };
        let writer = writer
            .child_option(VOTES_EML_NAME, self.votes.as_ref(), |elem, value| {
                elem.text(value.raw().as_ref())?.finish()
            })?
            .child_option(RANKING_EML_NAME, self.ranking.as_ref(), |elem, value| {
                elem.text(value.raw().as_ref())?.finish()
            })?;

        if self.elected.copied_value().ok() == Some(YesNoType(true)) {
            writer
                .child(ELECTED_EML_NAME, |elem| {
                    elem.text(self.elected.raw().as_ref())?.finish()
                })?
                .finish()
        } else {
            writer.finish()
        }
    }
}

/// The type of selection.
#[derive(Debug, Clone)]
pub enum ElectionResultSelectionType {
    /// Selection of a candidate.
    Candidate(Box<CandidateSelection>),
    /// Selection of an affiliation.
    Affiliation(Box<AffiliationSelection>),
}

/// Selection of a candidate.
#[derive(Debug, Clone)]
pub struct CandidateSelection {
    /// Identifier of the candidate.
    pub identifier: CandidateIdentifier,

    /// Name of the candidate.
    pub name: PersonNameStructure,

    /// Gender of the candidate.
    pub gender: Option<StringValue<Gender>>,

    /// The minimal qualifying address of the candidate, if present.
    pub qualifying_address: MinimalQualifyingAddress,
}

impl CandidateSelection {
    /// Create a new builder for building an instance.
    pub fn builder() -> CandidateSelectionBuilder {
        CandidateSelectionBuilder::new()
    }
}

/// A builder for [`CandidateSelection`].
#[derive(Debug, Clone)]
pub struct CandidateSelectionBuilder {
    identifier: Option<CandidateIdentifier>,
    name: Option<PersonNameStructure>,
    gender: Option<StringValue<Gender>>,
    qualifying_address: Option<MinimalQualifyingAddress>,
    locality_name: Option<Box<str>>,
    country_name_code: Option<Box<str>>,
}

impl CandidateSelectionBuilder {
    /// Create a new CandidateSelectionBuilder for building [`CandidateSelection`] instances.
    pub fn new() -> Self {
        CandidateSelectionBuilder {
            identifier: None,
            name: None,
            gender: None,
            qualifying_address: None,
            locality_name: None,
            country_name_code: None,
        }
    }

    /// Set the identifier of the candidate selection.
    pub fn identifier(mut self, identifier: impl Into<CandidateIdentifier>) -> Self {
        self.identifier = Some(identifier.into());
        self
    }

    /// Set the name of the candidate selection.
    pub fn name(mut self, name: impl Into<PersonNameStructure>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the gender of the candidate selection.
    pub fn gender(mut self, gender: impl Into<Gender>) -> Self {
        self.gender = Some(StringValue::from_value(gender.into()));
        self
    }

    /// Set the minimal qualifying address of the candidate selection.
    ///
    /// You may also set the locality name and country name code separately
    /// using the [`Self::locality_name`] and [`Self::country_name_code`] methods.
    pub fn qualifying_address(
        mut self,
        qualifying_address: impl Into<MinimalQualifyingAddress>,
    ) -> Self {
        self.qualifying_address = Some(qualifying_address.into());
        self
    }

    /// Set the locality name for the candidate selection.
    ///
    /// Has no effect if the qualifying address is already set using the
    /// [`Self::qualifying_address`] method.
    pub fn locality_name(mut self, locality_name: impl Into<Box<str>>) -> Self {
        self.locality_name = Some(locality_name.into());
        self
    }

    /// Set the country name code for the candidate selection.
    ///
    /// Has no effect if the qualifying address is already set using the
    /// [`Self::qualifying_address`] method.
    pub fn country_name_code(mut self, country_name_code: impl Into<Box<str>>) -> Self {
        self.country_name_code = Some(country_name_code.into());
        self
    }

    /// Build a CandidateSelection, returning an error if any required properties are missing.
    pub fn build(self) -> Result<CandidateSelection, EMLError> {
        Ok(CandidateSelection {
            identifier: self
                .identifier
                .ok_or_else(|| EMLErrorKind::MissingBuildProperty("identifier").without_span())?,
            name: self
                .name
                .ok_or_else(|| EMLErrorKind::MissingBuildProperty("name").without_span())?,
            gender: self.gender,
            qualifying_address: self.qualifying_address.map_or_else(
                || {
                    let locality_name = self.locality_name.ok_or_else(|| {
                        EMLErrorKind::MissingBuildProperty("locality_name").without_span()
                    })?;

                    if let Some(country_name_code) = self.country_name_code {
                        Ok(MinimalQualifyingAddress::new_country(
                            country_name_code,
                            locality_name,
                        ))
                    } else {
                        Ok(MinimalQualifyingAddress::new_locality(locality_name))
                    }
                },
                Ok,
            )?,
        })
    }
}

impl Default for CandidateSelectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EMLElement for CandidateSelection {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Candidate", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, CandidateSelection {
            identifier: CandidateIdentifier::EML_NAME => |elem| CandidateIdentifier::read_eml(elem)?,
            name: ("CandidateFullName", NS_EML) => |elem| PersonNameStructure::read_eml_element(elem)?,
            gender as Option: ("Gender", NS_EML) => |elem| elem.string_value()?,
            qualifying_address: MinimalQualifyingAddress::EML_NAME => |elem| MinimalQualifyingAddress::read_eml(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child_elem(CandidateIdentifier::EML_NAME, &self.identifier)?
            .child(("CandidateFullName", NS_EML), |elem| {
                self.name.write_eml_element(elem)
            })?
            .child_option(("Gender", NS_EML), self.gender.as_ref(), |elem, value| {
                elem.text(value.raw().as_ref())?.finish()
            })?
            .child_elem(MinimalQualifyingAddress::EML_NAME, &self.qualifying_address)?
            .finish()
    }
}

/// Selection of an affiliation.
#[derive(Debug, Clone)]
pub struct AffiliationSelection {
    /// Id of the affiliation.
    pub id: StringValue<AffiliationId>,

    /// Name of the affiliation.
    pub name: Box<str>,
}

impl AffiliationSelection {
    /// Create a new AffiliationSelection with the given id and name.
    pub fn new(id: impl Into<AffiliationId>, name: impl Into<Box<str>>) -> Self {
        AffiliationSelection {
            id: StringValue::from_value(id.into()),
            name: name.into(),
        }
    }
}

impl EMLElement for AffiliationSelection {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("AffiliationIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, AffiliationSelection {
            id: elem.string_value_attr("Id", None)?,
            name: ("RegisteredName", NS_EML) => |elem| elem.text_without_children()?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr("Id", self.id.raw().as_ref())?
            .child(("RegisteredName", NS_EML), |elem| {
                elem.text(self.name.as_ref())?.finish()
            })?
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use chrono::TimeZone as _;

    use crate::{
        common::{AuthorityIdentifier, PersonName},
        io::{EMLParsingMode, EMLRead as _, EMLWrite},
        utils::{AuthorityId, CandidateId},
    };

    use super::*;

    #[test]
    fn test_election_result_construction() {
        let election_result = ElectionResult::builder()
            .transaction_id(TransactionId::new(1))
            .managing_authority(
                AuthorityIdentifier::new(AuthorityId::new("1234").unwrap()).with_name("Place"),
            )
            .creation_date_time(chrono::Utc.with_ymd_and_hms(2024, 3, 17, 12, 0, 0).unwrap())
            .election_identifier(
                ElectionResultElectionIdentifier::builder()
                    .id(ElectionId::new("GR2024_Place").unwrap())
                    .name("Gemeenteraadsverkiezingen 2024")
                    .category(ElectionCategory::GR)
                    .election_date(XsDate::from_date(2024, 3, 17).unwrap())
                    .build_for_result()
                    .unwrap(),
            )
            .contests([ElectionResultContest::new(
                ContestIdentifier::geen(),
                [
                    ElectionResultSelection::builder()
                        .affiliation(AffiliationSelection::new(
                            AffiliationId::new(NonZeroU64::new(1).unwrap()),
                            "Example",
                        ))
                        .elected(true)
                        .build()
                        .unwrap(),
                    ElectionResultSelection::builder()
                        .candidate(
                            CandidateSelection::builder()
                                .identifier(CandidateId::new(NonZeroU64::new(1).unwrap()))
                                .name(PersonName::new("Smid").with_first_name("Example"))
                                .locality_name("Locality")
                                .country_name_code("NL")
                                .build()
                                .unwrap(),
                        )
                        .elected(true)
                        .votes(100u64)
                        .build()
                        .unwrap(),
                    ElectionResultSelection::builder()
                        .candidate(
                            CandidateSelection::builder()
                                .identifier(CandidateId::new(NonZeroU64::new(2).unwrap()))
                                .name(PersonName::new("Test").with_first_name("Example"))
                                .locality_name("Locality")
                                .country_name_code("NL")
                                .build()
                                .unwrap(),
                        )
                        .elected(false)
                        .votes(0u64)
                        .build()
                        .unwrap(),
                ],
            )])
            .build()
            .unwrap();
        let xml = election_result.write_eml_root_str(true, true).unwrap();
        assert_eq!(
            xml,
            include_str!("../../test-files/election_result/eml520_construction_output.eml.xml")
        );

        // check if it still is the same after a second parse and write
        let parsed = ElectionResult::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        let xml2 = parsed.write_eml_root_str(true, true).unwrap();
        assert_eq!(xml, xml2);
    }
}
