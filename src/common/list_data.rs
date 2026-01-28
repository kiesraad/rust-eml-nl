use std::num::NonZeroU64;

use thiserror::Error;

use crate::{
    NS_KR,
    io::{EMLElement, QualifiedName, collect_struct},
    utils::{ContestIdType, PublicationLanguageType, StringValue, StringValueData},
};

/// Additional data for affiliation lists.
#[derive(Debug, Clone)]
pub struct ListData {
    /// Whether to publish the genders for this list.
    pub publish_gender: StringValue<bool>,

    /// The publication language for this list.
    pub publication_language: Option<StringValue<PublicationLanguageType>>,

    /// If this list is of type [`AffiliationType::SetOfEqualLists`](crate::utils::AffiliationType::SetOfEqualLists), the set
    /// it belongs to.
    pub belongs_to_set: Option<StringValue<NonZeroU64>>,

    /// If this list is of type [`AffiliationType::GroupOfLists`](crate::utils::AffiliationType::GroupOfLists), the
    /// combination it belongs to.
    pub belongs_to_combination: Option<StringValue<ListDataBelongsToCombinationType>>,

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

    /// Get the publication language, defaulting to [`PublicationLanguageType::default()`] if not set or invalid.
    pub fn get_publication_language(&self) -> PublicationLanguageType {
        self.publication_language
            .as_ref()
            .map(|s| match s {
                StringValue::Parsed(v) => *v,
                StringValue::Raw(r) => {
                    PublicationLanguageType::from_str_value(r).unwrap_or_default()
                }
            })
            .unwrap_or_default()
    }
}

impl EMLElement for ListData {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("ListData", Some(NS_KR));

    fn read_eml(elem: &mut crate::io::EMLElementReader<'_, '_>) -> Result<Self, crate::EMLError> {
        let publish_gender = elem.string_value_attr("PublishGender", None)?;
        let publication_language = elem.string_value_attr_opt("PublicationLanguage")?;
        let belongs_to_set = elem.string_value_attr_opt("BelongsToSet")?;
        let belongs_to_combination = elem.string_value_attr_opt("BelongsToCombination")?;

        // temporary struct to collect optional contests element
        struct ListDataContests {
            contests: Option<Vec<ListDataContest>>,
        }

        let tmp = collect_struct!(elem, ListDataContests {
            contests as Option: ("Contests", NS_KR) => |elem| {
                // Temporary struct to collect contest elements
                struct Contests {
                    contests: Vec<ListDataContest>,
                }

                let res = collect_struct!(elem, Contests {
                    contests as Vec: ListDataContest::EML_NAME => |elem| ListDataContest::read_eml(elem)?,
                });

                res.contests
            },
        });

        Ok(ListData {
            publish_gender,
            publication_language,
            belongs_to_set,
            belongs_to_combination,
            contests: tmp.contests.unwrap_or_default(),
        })
    }

    fn write_eml(&self, writer: crate::io::EMLElementWriter) -> Result<(), crate::EMLError> {
        let writer = writer
            .attr("PublishGender", &self.publish_gender.raw())?
            .attr_opt(
                "PublicationLanguage",
                self.publication_language.as_ref().map(|s| s.raw()),
            )?
            .attr_opt(
                "BelongsToSet",
                self.belongs_to_set.as_ref().map(|s| s.raw()),
            )?
            .attr_opt(
                "BelongsToCombination",
                self.belongs_to_combination.as_ref().map(|s| s.raw()),
            )?;

        if self.contests.is_empty() {
            writer.empty()
        } else {
            writer
                .child(("Contests", NS_KR), |writer| {
                    let mut writer = writer.content()?;
                    for contest in &self.contests {
                        writer = writer.child(ListDataContest::EML_NAME, |writer| {
                            contest.write_eml(writer)
                        })?;
                    }

                    writer.finish()
                })?
                .finish()
        }
    }
}

/// Data for a contest associated with a list.
#[derive(Debug, Clone)]
pub struct ListDataContest {
    /// The contest ID.
    pub id: StringValue<ContestIdType>,

    /// An optional name for the contest.
    pub name: Option<String>,
}

impl EMLElement for ListDataContest {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Contest", Some(NS_KR));

    fn read_eml(elem: &mut crate::io::EMLElementReader<'_, '_>) -> Result<Self, crate::EMLError>
    where
        Self: Sized,
    {
        Ok(collect_struct!(
            elem,
            ListDataContest {
                id: elem.string_value_attr("Id", None)?,
                name: elem.text_without_children_opt()?,
            }
        ))
    }

    fn write_eml(&self, writer: crate::io::EMLElementWriter) -> Result<(), crate::EMLError> {
        let writer = writer.attr("Id", &self.id.raw())?;

        if let Some(name) = &self.name {
            writer.text(name)?.finish()
        } else {
            writer.empty()
        }
    }
}

/// Type representing the combination a list belongs to.
#[derive(Debug, Clone)]
pub struct ListDataBelongsToCombinationType(String);

/// Error returned when an invalid list data belongs to combination type string is encountered.
#[derive(Debug, Clone, Error)]
#[error("Invalid list data belongs to combination type: {0}")]
pub struct InvalidListDataBelongsToCombinationType(String);

impl StringValueData for ListDataBelongsToCombinationType {
    type Error = InvalidListDataBelongsToCombinationType;

    fn parse_from_str(s: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        if s.len() == 1
            && s.chars()
                .next()
                .map(|c| c.is_ascii_alphabetic())
                .unwrap_or(false)
        {
            Ok(ListDataBelongsToCombinationType(s.to_string()))
        } else {
            Err(InvalidListDataBelongsToCombinationType(s.to_string()))
        }
    }

    fn to_raw_value(&self) -> String {
        self.0.clone()
    }
}
