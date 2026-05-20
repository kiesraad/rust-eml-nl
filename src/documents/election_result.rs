//! Document variant for the EML_NL Result (`520`) document.

use std::{ops::Deref, str::FromStr};

use instant_xml::{FromXml, ToXml};

use crate::{
    EML_SCHEMA_VERSION, EMLError, EMLErrorKind, NS_EML, NS_KR,
    common::{
        CandidateIdentifier, CanonicalizationMethod, ContestIdentifier, CreationDateTime,
        ElectionDomain, ManagingAuthority, MinimalQualifyingAddress, PersonNameStructure,
        TransactionId,
    },
    documents::ElectionIdentifierBuilder,
    eml_ns_context,
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
        Self::parse_eml(s).ok()
    }
}

impl TryFrom<&str> for ElectionResult {
    type Error = EMLError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use crate::io::EMLRead as _;
        Self::parse_eml(value).ok()
    }
}

impl TryFrom<ElectionResult> for String {
    type Error = EMLError;

    fn try_from(value: ElectionResult) -> Result<Self, Self::Error> {
        use crate::io::EMLWrite as _;
        value.write_eml_root_str(true)
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

// Custom: root EML element with Id/SchemaVersion attributes and full namespace context.
impl<'xml> FromXml<'xml> for ElectionResult {
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
        let mut creation_date_time = <CreationDateTime as FromXml>::Accumulator::default();
        let mut canonicalization_method =
            <CanonicalizationMethod as FromXml>::Accumulator::default();
        let mut result = <ElectionResultResult as FromXml>::Accumulator::default();

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
            } else if ElectionResultResult::matches(id, None) {
                let mut nested = deserializer.nested(element);
                ElectionResultResult::deserialize(&mut result, field, &mut nested)?;
                nested.ignore()?;
            } else {
                let mut nested = deserializer.nested(element);
                nested.ignore()?;
            }
        }

        *into = Some(ElectionResult {
            transaction_id: transaction_id.try_done(field)?,
            managing_authority: managing_authority.try_done(field)?,
            creation_date_time: creation_date_time.try_done(field)?,
            canonicalization_method,
            result: result.try_done(field)?,
        });
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Element;
}

// Custom: root EML element with Id/SchemaVersion attributes and full namespace context.
impl ToXml for ElectionResult {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix = serializer.write_start("EML", NS_EML, Some(eml_ns_context()))?;
        serializer.write_attr("Id", "", EML_ELECTION_RESULT_ID)?;
        serializer.write_attr("SchemaVersion", "", EML_SCHEMA_VERSION)?;
        serializer.end_start()?;
        self.transaction_id.serialize(None, serializer)?;
        self.managing_authority.serialize(None, serializer)?;
        self.creation_date_time.serialize(None, serializer)?;
        self.result.serialize(None, serializer)?;
        serializer.write_close(prefix)
    }
}

/// The result data of an election result document.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "Result", ns(NS_EML))]
pub struct ElectionResultResult {
    /// The election for which the result applies.
    #[xml(rename = "Election")]
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

/// The election for which the result applies.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "Election", ns(NS_EML))]
pub struct ElectionResultElection {
    /// Identifier for the election.
    #[xml(rename = "ElectionIdentifier")]
    pub identifier: ElectionResultElectionIdentifier,

    /// Contests within the election.
    #[xml(rename = "Contest")]
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

/// Identifier for the election for which the result applies.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "ElectionIdentifier", ns(NS_EML, kr = NS_KR))]
pub struct ElectionResultElectionIdentifier {
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
}

impl ElectionResultElectionIdentifier {
    /// Builder for creating a new instance.
    pub fn builder() -> ElectionIdentifierBuilder {
        ElectionIdentifierBuilder::new()
    }
}

/// A contest within an election result.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "Contest", ns(NS_EML))]
pub struct ElectionResultContest {
    /// Identifier of the contest.
    #[xml(rename = "ContestIdentifier")]
    pub identifier: ContestIdentifier,

    /// Selections within the contest.
    #[xml(rename = "Selection")]
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

/// The ranking of a selection.
#[derive(Debug, Clone, PartialEq, Eq, FromXml, ToXml)]
#[xml(scalar)]
pub enum RankingType {
    /// First ranked selection.
    #[xml(rename = "1")]
    First,
    /// Second ranked selection.
    #[xml(rename = "2")]
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

    fn to_raw_value(&self) -> String {
        self.as_str().to_string()
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

    fn to_raw_value(&self) -> String {
        self.to_eml_value().to_string()
    }
}

/// A selection within an election contest.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "Selection", rename_all = "PascalCase", ns(NS_EML))]
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

/// The type of selection.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(forward)]
pub enum ElectionResultSelectionType {
    /// Selection of a candidate.
    Candidate(Box<CandidateSelection>),
    /// Selection of an affiliation.
    Affiliation(Box<AffiliationSelection>),
}

/// Selection of a candidate.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "Candidate", ns(NS_EML))]
pub struct CandidateSelection {
    /// Identifier of the candidate.
    #[xml(rename = "CandidateIdentifier")]
    pub identifier: CandidateIdentifier,

    /// Name of the candidate.
    #[xml(rename = "CandidateFullName")]
    pub name: PersonNameStructure,

    /// Gender of the candidate.
    #[xml(rename = "Gender")]
    pub gender: Option<StringValue<Gender>>,

    /// The minimal qualifying address of the candidate, if present.
    #[xml(rename = "QualifyingAddress")]
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
    locality_name: Option<String>,
    country_name_code: Option<String>,
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
    pub fn locality_name(mut self, locality_name: impl Into<String>) -> Self {
        self.locality_name = Some(locality_name.into());
        self
    }

    /// Set the country name code for the candidate selection.
    ///
    /// Has no effect if the qualifying address is already set using the
    /// [`Self::qualifying_address`] method.
    pub fn country_name_code(mut self, country_name_code: impl Into<String>) -> Self {
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

/// Selection of an affiliation.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "AffiliationIdentifier", ns(NS_EML))]
pub struct AffiliationSelection {
    /// Id of the affiliation.
    #[xml(attribute, rename = "Id")]
    pub id: StringValue<AffiliationId>,

    /// Name of the affiliation.
    #[xml(rename = "RegisteredName")]
    pub name: String,
}

impl AffiliationSelection {
    /// Create a new AffiliationSelection with the given id and name.
    pub fn new(id: impl Into<AffiliationId>, name: impl Into<String>) -> Self {
        AffiliationSelection {
            id: StringValue::from_value(id.into()),
            name: name.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use chrono::TimeZone as _;

    use crate::{
        common::{AuthorityIdentifier, PersonName},
        documents::EML,
        io::{EMLRead as _, EMLWrite},
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
        let xml = election_result.write_eml_root_str(true).unwrap();

        // check if it still is the same after a second parse and write
        let eml = EML::parse_eml(&xml).unwrap();
        let parsed = eml
            .as_result_doc()
            .expect("expected election result variant");
        let xml2 = parsed.write_eml_root_str(true).unwrap();
        assert_eq!(xml, xml2);
    }
}
