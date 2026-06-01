use crate::{
    EMLError, NS_KR,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName},
    utils::{ElectionDomainId, StringValue},
};

/// Election domain of an election.
///
/// The (top level) region where the election takes place. Only needed if the
/// ElectionDomain is part of the election name, e.g. election of the council of
/// a municipality or province. Not needed e.g. for Tweede Kamer or European
/// Parliament.
#[derive(Debug, Clone)]
pub struct ElectionDomain {
    /// Identifier of the election domain
    pub id: Option<StringValue<ElectionDomainId>>,
    /// Name of the election domain
    pub name: Box<str>,
}

impl ElectionDomain {
    /// Create a new ElectionDomain
    pub fn new(id: Option<ElectionDomainId>, name: impl Into<Box<str>>) -> Self {
        ElectionDomain {
            id: id.map(StringValue::from_value),
            name: name.into(),
        }
    }
}

impl EMLElement for ElectionDomain {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ElectionDomain", Some(NS_KR));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let id = elem.string_value_attr_opt("Id")?;
        let name = elem.text_without_children()?;

        Ok(ElectionDomain { id, name })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr_opt("Id", self.id.as_ref().map(|v| v.raw()))?
            .text(self.name.as_ref())?
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{EMLParsingMode, EMLRead, test_write_eml_element, test_xml_fragment};

    #[test]
    fn test_election_domain_construction() {
        let ed = ElectionDomain::new(Some(ElectionDomainId::new("1234").unwrap()), "Test Domain");
        assert_eq!(ed.id.as_ref().unwrap().raw(), "1234");
        assert_eq!(ed.name.as_ref(), "Test Domain");
    }

    #[test]
    fn test_election_domain_parsing() {
        let xml = test_xml_fragment(
            r#"<kr:ElectionDomain xmlns:kr="http://www.kiesraad.nl/extensions" Id="1234">Test Domain</kr:ElectionDomain>"#,
        );
        let ed = ElectionDomain::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(ed.id.as_ref().unwrap().raw(), "1234");
        assert_eq!(ed.name.as_ref(), "Test Domain");

        let xml_output = test_write_eml_element(&ed, &[NS_KR]).unwrap();
        assert_eq!(xml_output, xml);
    }

    #[test]
    fn test_election_domain_parsing_without_id() {
        let xml = test_xml_fragment(
            r#"<kr:ElectionDomain xmlns:kr="http://www.kiesraad.nl/extensions">Test Domain</kr:ElectionDomain>"#,
        );
        let ed = ElectionDomain::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert!(ed.id.is_none());
        assert_eq!(ed.name.as_ref(), "Test Domain");
        let xml_output = test_write_eml_element(&ed, &[NS_KR]).unwrap();
        assert_eq!(xml_output, xml);
    }
}
