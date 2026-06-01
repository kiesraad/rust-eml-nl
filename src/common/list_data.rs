use std::num::NonZeroU64;

use thiserror::Error;

use crate::{
    EMLError, EMLValueResultExt, NS_KR,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName, collect_struct},
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

impl EMLElement for ListData {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("ListData", Some(NS_KR));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
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

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
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
                    writer
                        .child_elems(ListDataContest::EML_NAME, &self.contests)?
                        .finish()
                })?
                .finish()
        }
    }
}

/// Data for a contest associated with a list.
#[derive(Debug, Clone)]
pub struct ListDataContest {
    /// The contest ID.
    pub id: StringValue<ContestId>,

    /// An optional name for the contest.
    pub name: Option<Box<str>>,
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
    pub fn with_name(mut self, name: impl Into<Box<str>>) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl EMLElement for ListDataContest {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Contest", Some(NS_KR));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(ListDataContest {
            id: elem.string_value_attr("Id", None)?,
            name: elem.text_without_children_opt()?,
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        let writer = writer.attr("Id", &self.id.raw())?;

        if let Some(name) = &self.name {
            writer.text(name)?.finish()
        } else {
            writer.empty()
        }
    }
}

/// Type representing the combination a list belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListDataBelongsToCombination(Box<str>);

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
            Ok(ListDataBelongsToCombination(s.into()))
        } else {
            Err(InvalidListDataBelongsToCombinationError(s.into()))
        }
    }

    fn to_raw_value(&self) -> Box<str> {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{EMLRead as _, test_write_eml_element, test_xml_fragment};

    #[test]
    fn test_list_data_construction() {
        let list_data = ListData::new(true)
            .with_publication_language(PublicationLanguage::Frisian)
            .with_belongs_to_set(NonZeroU64::new(1).unwrap())
            .with_belongs_to_combination(ListDataBelongsToCombination("A".into()));

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
        assert_eq!(contest.name.as_ref().unwrap().as_ref(), "Test Contest");
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

        let list_data = ListData::parse_eml(&xml, crate::io::EMLParsingMode::Strict).unwrap();

        assert_eq!(
            list_data.belongs_to_combination,
            Some(StringValue::Parsed(ListDataBelongsToCombination(
                "A".into()
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

        let xml_output = test_write_eml_element(&list_data, &[NS_KR]).unwrap();
        assert_eq!(xml_output, xml);
    }

    #[test]
    fn test_list_data_simple_parsing() {
        let xml = test_xml_fragment(
            r#"<kr:ListData xmlns:kr="http://www.kiesraad.nl/extensions" PublishGender="false"/>"#,
        );

        let list_data = ListData::parse_eml(&xml, crate::io::EMLParsingMode::Strict).unwrap();

        assert!(!list_data.publish_gender.value().unwrap().into_owned());
        assert_eq!(
            list_data.get_publication_language(),
            PublicationLanguage::Dutch
        );

        let xml_output = test_write_eml_element(&list_data, &[NS_KR]).unwrap();
        assert_eq!(xml_output, xml);
    }
}
