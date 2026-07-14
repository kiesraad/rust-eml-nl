use crate::io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName};
use crate::utils::CommitteeCategory;
use crate::{EMLError, NS_KR};

/// Committee
#[derive(Debug, Clone)]
pub struct Committee {
    /// Category of the committee.
    pub category: CommitteeCategory,

    /// Optional committee name.
    pub name: Option<String>,

    /// Whether the committee accepts central submissions.
    pub accept_central_submissions: Option<bool>,
}

impl Committee {
    ///Create a new committee.
    pub fn new(category: CommitteeCategory) -> Self {
        Committee {
            category,
            name: None,
            accept_central_submissions: None,
        }
    }

    /// Set the `CommitteeName` attribute of the `Committee` element.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the `AcceptCentralSubmissions` attribute of the `Committee` element.
    pub fn with_accept_central_submissions(mut self, accept: bool) -> Self {
        self.accept_central_submissions = Some(accept);
        self
    }
}

impl EMLElement for Committee {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Committee", Some(NS_KR));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(Committee {
            category: CommitteeCategory::new(elem.attribute_value_req("CommitteeCategory")?)?,
            name: elem
                .attribute_value("CommitteeName")?
                .map(|name| name.into_owned()),
            accept_central_submissions: elem
                .string_value_attr_opt("AcceptCentralSubmissions")?
                .map(|value| value.copied_value())
                .transpose()?,
        })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr("CommitteeCategory", self.category.to_eml_value())?
            .attr_opt("CommitteeName", self.name.as_ref())?
            .attr_opt(
                "AcceptCentralSubmissions",
                self.accept_central_submissions
                    .map(|value| value.to_string()),
            )?
            .empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{EMLParsingMode, EMLRead, test_write_eml_element, test_xml_fragment};

    #[test]
    fn test_committee_parsing() {
        let xml = test_xml_fragment(
            r#"<kr:Committee xmlns:kr="http://www.kiesraad.nl/extensions" CommitteeCategory="HSB" CommitteeName="Committee 1" AcceptCentralSubmissions="false"/>"#,
        );
        let committee = Committee::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(committee.name, Some("Committee 1".to_string()));
        assert_eq!(committee.category, CommitteeCategory::HSB);
        assert_eq!(committee.accept_central_submissions, Some(false));

        let xml_output = test_write_eml_element(&committee, &[NS_KR]).unwrap();
        assert_eq!(xml_output, xml);
    }
}
