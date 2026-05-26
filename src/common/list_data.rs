use std::{fmt, num::NonZeroU64};

use instant_xml::{FromXml, ToXml};
use thiserror::Error;

use crate::{
    EMLError, EMLValueResultExt, NS_KR,
    utils::{ContestId, PublicationLanguage, StringValue, StringValueData},
};

/// Additional data for affiliation lists.
#[derive(Debug, Clone)]
pub struct ListData {
    /// Whether to publish the genders for this list.
    pub publish_gender: StringValue<bool>,

    /// The publication language for this list.
    pub publication_language: Option<StringValue<PublicationLanguage>>,

    /// If this list is of type [`AffiliationType::SetOfEqualLists`](crate::utils::AffiliationType::SetOfEqualLists), the set
    /// it belongs to.
    pub belongs_to_set: Option<StringValue<NonZeroU64>>,

    /// If this list is of type [`AffiliationType::GroupOfLists`](crate::utils::AffiliationType::GroupOfLists), the
    /// combination it belongs to.
    pub belongs_to_combination: Option<StringValue<ListDataBelongsToCombination>>,

    /// An optional list of contests this list is associated with.
    pub contests: Vec<ListDataContest>,
}

impl ListData {
    /// Create a new `ListData` with default values.
    pub fn new(publish_gender: bool) -> Self {
        ListData {
            publish_gender: StringValue::Parsed(publish_gender),
            publication_language: None,
            belongs_to_set: None,
            belongs_to_combination: None,
            contests: Vec::new(),
        }
    }

    /// Get the publication language, defaulting to [`PublicationLanguage::default()`] if not set or invalid.
    pub fn get_publication_language(&self) -> PublicationLanguage {
        self.publication_language
            .as_ref()
            .map(|s| match s {
                StringValue::Parsed(v) => *v,
                StringValue::Raw(r) => PublicationLanguage::from_eml_value(r).unwrap_or_default(),
            })
            .unwrap_or_default()
    }

    /// Set the publication language for this list.
    pub fn with_publication_language(mut self, language: PublicationLanguage) -> Self {
        self.publication_language = Some(StringValue::Parsed(language));
        self
    }

    /// Set the set this list belongs to, if it is of type
    /// [`AffiliationType::SetOfEqualLists`](crate::utils::AffiliationType::SetOfEqualLists).
    pub fn with_belongs_to_set(mut self, set_id: NonZeroU64) -> Self {
        self.belongs_to_set = Some(StringValue::Parsed(set_id));
        self
    }

    /// Set the combination this list belongs to, if it is of type
    /// [`AffiliationType::GroupOfLists`](crate::utils::AffiliationType::GroupOfLists).
    pub fn with_belongs_to_combination(
        mut self,
        combination_id: ListDataBelongsToCombination,
    ) -> Self {
        self.belongs_to_combination = Some(StringValue::Parsed(combination_id));
        self
    }
}

impl<'xml> FromXml<'xml> for ListData {
    fn matches(id: instant_xml::Id<'_>, field: Option<instant_xml::Id<'_>>) -> bool {
        match field {
            Some(field) => id == field,
            None => {
                id == instant_xml::Id {
                    ns: NS_KR,
                    name: "ListData",
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

        let mut publish_gender = None;
        let mut publication_language = None;
        let mut belongs_to_set = None;
        let mut belongs_to_combination = None;
        let mut contests = Vec::new();

        // Read attributes
        while let Some(node) = deserializer.next() {
            let node = node?;
            match node {
                Node::Attribute(attr) => match attr.local {
                    "PublishGender" => {
                        publish_gender =
                            Some(match StringValue::<bool>::from_raw_parsed(&*attr.value) {
                                Ok(v) => v,
                                Err(_) => StringValue::Raw(attr.value.into_owned()),
                            });
                    }
                    "PublicationLanguage" => {
                        publication_language = Some(
                            match StringValue::<PublicationLanguage>::from_raw_parsed(&*attr.value)
                            {
                                Ok(v) => v,
                                Err(_) => StringValue::Raw(attr.value.into_owned()),
                            },
                        );
                    }
                    "BelongsToSet" => {
                        belongs_to_set = Some(
                            match StringValue::<NonZeroU64>::from_raw_parsed(&*attr.value) {
                                Ok(v) => v,
                                Err(_) => StringValue::Raw(attr.value.into_owned()),
                            },
                        );
                    }
                    "BelongsToCombination" => {
                        belongs_to_combination = Some(
                            match StringValue::<ListDataBelongsToCombination>::from_raw_parsed(
                                &*attr.value,
                            ) {
                                Ok(v) => v,
                                Err(_) => StringValue::Raw(attr.value.into_owned()),
                            },
                        );
                    }
                    name => return Err(Error::UnexpectedValue(name.to_owned())),
                },
                Node::Open(element) => {
                    let id = deserializer.element_id(&element)?;
                    if id
                        == (instant_xml::Id {
                            ns: NS_KR,
                            name: "Contests",
                        })
                    {
                        let mut nested = deserializer.nested(element);
                        while let Some(node) = nested.next() {
                            let node = node?;
                            if let Node::Open(contest_elem) = node {
                                let mut acc =
                                    <ListDataContest as FromXml<'xml>>::Accumulator::default();
                                let mut cn = nested.nested(contest_elem);
                                ListDataContest::deserialize(&mut acc, field, &mut cn)?;
                                cn.ignore()?;
                                contests.push(acc.try_done(field)?);
                            }
                        }
                    } else {
                        let mut nested = deserializer.nested(element);
                        nested.ignore()?;
                    }
                }
                Node::Text(s) if s.trim().is_empty() => continue,
                node => return Err(Error::UnexpectedNode(format!("{node:?}"))),
            }
        }

        *into = Some(ListData {
            publish_gender: publish_gender.ok_or(Error::MissingValue("PublishGender"))?,
            publication_language,
            belongs_to_set,
            belongs_to_combination,
            contests,
        });
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Element;
}

// Custom: children wrapped in conditional `<Contests>` element; empty-element when no contests.
impl ToXml for ListData {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<'_, W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix =
            serializer.write_start("ListData", NS_KR, None::<instant_xml::ser::Context<0>>)?;
        serializer.write_attr("PublishGender", "", &self.publish_gender)?;

        if let Some(v) = &self.publication_language {
            serializer.write_attr("PublicationLanguage", "", v)?;
        }
        if let Some(v) = &self.belongs_to_set {
            serializer.write_attr("BelongsToSet", "", v)?;
        }
        if let Some(v) = &self.belongs_to_combination {
            serializer.write_attr("BelongsToCombination", "", v)?;
        }
        if self.contests.is_empty() {
            return serializer.end_empty();
        }

        serializer.end_start()?;
        let contests_prefix =
            serializer.write_start("Contests", NS_KR, None::<instant_xml::ser::Context<0>>)?;
        serializer.end_start()?;
        for contest in &self.contests {
            contest.serialize(None, serializer)?;
        }
        serializer.write_close(contests_prefix)?;
        serializer.write_close(prefix)
    }
}

/// Data for a contest associated with a list.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename = "Contest", rename_all = "PascalCase", ns(NS_KR), force_prefix)]
pub struct ListDataContest {
    /// The contest ID.
    #[xml(attribute)]
    pub id: StringValue<ContestId>,

    /// An optional name for the contest.
    #[xml(direct)]
    pub name: Option<String>,
}

impl ListDataContest {
    /// Create a new `ListDataContest` with the given ID and no name.
    pub fn new(id: ContestId) -> Self {
        ListDataContest {
            id: StringValue::from_value(id),
            name: None,
        }
    }

    /// Set the name of the contest.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// Type representing the combination a list belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListDataBelongsToCombination(String);

impl ListDataBelongsToCombination {
    /// Create a new `ListDataBelongsToCombination` with the given combination identifier.
    pub fn new(combination_id: impl AsRef<str>) -> Result<Self, EMLError> {
        ListDataBelongsToCombination::parse_from_str(combination_id.as_ref()).wrap_value_error()
    }

    /// Get the raw string value of this combination.
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Error returned when an invalid list data belongs to combination type string is encountered.
#[derive(Debug, Clone, Error)]
#[error("Invalid list data belongs to combination type: {0}")]
pub struct InvalidListDataBelongsToCombinationError(String);

impl StringValueData for ListDataBelongsToCombination {
    type Error = InvalidListDataBelongsToCombinationError;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        // Note: assuming that `|` is not allowed in combination identifiers, unlike the regex in the spec
        if s.len() == 1
            && s.chars()
                .next()
                .map(|c| c.is_ascii_alphabetic())
                .unwrap_or(false)
        {
            Ok(ListDataBelongsToCombination(s.to_string()))
        } else {
            Err(InvalidListDataBelongsToCombinationError(s.to_string()))
        }
    }

    fn to_raw_value(&self) -> String {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{EMLRead as _, test_xml_fragment};

    #[test]
    fn test_list_data_construction() {
        let list_data = ListData::new(true)
            .with_publication_language(PublicationLanguage::Frisian)
            .with_belongs_to_set(NonZeroU64::new(1).unwrap())
            .with_belongs_to_combination(ListDataBelongsToCombination("A".to_string()));

        assert_eq!(list_data.publish_gender.raw(), "true");
        assert_eq!(
            list_data.get_publication_language(),
            PublicationLanguage::Frisian
        );
        assert_eq!(list_data.belongs_to_set.as_ref().unwrap().raw(), "1");
        assert_eq!(
            list_data.belongs_to_combination.as_ref().unwrap().raw(),
            "A"
        );
    }

    #[test]
    fn test_list_data_contest_construction() {
        let contest =
            ListDataContest::new(ContestId::new("1234").unwrap()).with_name("Test Contest");

        assert_eq!(contest.id.raw(), "1234");
        assert_eq!(contest.name.as_ref().unwrap(), "Test Contest");
    }

    #[test]
    fn test_list_data_parsing() {
        let xml = test_xml_fragment(
            r#"
            <kr:ListData xmlns:kr="http://www.kiesraad.nl/extensions" PublishGender="true" PublicationLanguage="nl" BelongsToSet="1" BelongsToCombination="A">
                <kr:Contests>
                    <kr:Contest Id="1234">Test Contest 1</kr:Contest>
                    <kr:Contest Id="5678">Test Contest 2</kr:Contest>
                </kr:Contests>
            </kr:ListData>
            "#,
        );

        let list_data = ListData::parse_eml(&xml).unwrap();

        assert_eq!(
            list_data.belongs_to_combination,
            Some(StringValue::Parsed(ListDataBelongsToCombination(
                "A".to_string()
            )))
        );
        assert_eq!(
            list_data.publication_language,
            Some(StringValue::Parsed(PublicationLanguage::Dutch))
        );
        assert_eq!(
            list_data.belongs_to_set,
            Some(StringValue::Parsed(NonZeroU64::new(1).unwrap()))
        );

        assert_eq!(list_data.contests.len(), 2);
        assert_eq!(list_data.contests[0].id.raw(), "1234");
        assert_eq!(
            list_data.contests[0].name.as_deref(),
            Some("Test Contest 1")
        );
        assert_eq!(list_data.contests[1].id.raw(), "5678");
        assert_eq!(
            list_data.contests[1].name.as_deref(),
            Some("Test Contest 2")
        );
    }

    #[test]
    fn test_list_data_simple_parsing() {
        let xml = test_xml_fragment(
            r#"<kr:ListData xmlns:kr="http://www.kiesraad.nl/extensions" PublishGender="false"/>"#,
        );

        let list_data = ListData::parse_eml(&xml).unwrap();

        assert!(!list_data.publish_gender.value().unwrap().into_owned());
        assert_eq!(
            list_data.get_publication_language(),
            PublicationLanguage::Dutch
        );
    }
}
