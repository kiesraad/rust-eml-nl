use crate::{
    EMLError, NS_EML,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName, collect_struct},
    utils::{ContestId, ContestIdGeen, StringValue},
};

/// Identifier for the contest.
#[derive(Debug, Clone)]
pub struct ContestIdentifier {
    /// Id of the contest.
    pub id: StringValue<ContestId>,

    /// Name of the contest.
    pub name: Option<Box<str>>,
}

impl ContestIdentifier {
    /// Create a new `ContestIdentifier`.
    pub fn new(id: ContestId) -> Self {
        ContestIdentifier {
            id: StringValue::Parsed(id),
            name: None,
        }
    }

    /// Set a name for this contest identifier
    pub fn with_name(mut self, name: impl Into<Box<str>>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Check if the contest identifier is of type 'geen'.
    pub fn is_geen(&self) -> bool {
        match &self.id {
            StringValue::Parsed(id) => id.is_geen(),
            StringValue::Raw(s) => s.as_ref() == "geen",
        }
    }

    /// Check if the contest identifier is of type 'alle'.
    pub fn is_alle(&self) -> bool {
        match &self.id {
            StringValue::Parsed(id) => id.is_alle(),
            StringValue::Raw(s) => s.as_ref() == "alle",
        }
    }

    /// Create a new `ContestIdentifier` with 'geen' type.
    pub fn geen() -> Self {
        ContestIdentifier::new(ContestId::geen())
    }

    /// Create a new `ContestIdentifier` with 'alle' type.
    pub fn alle() -> Self {
        ContestIdentifier::new(ContestId::alle())
    }
}

impl EMLElement for ContestIdentifier {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ContestIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, ContestIdentifier {
            id: elem.string_value_attr("Id", None)?,
            name as Option: ("ContestName", NS_EML) => |elem| elem.text_without_children()?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        let writer = writer.attr("Id", self.id.raw().as_ref())?;

        if let Some(name) = &self.name {
            writer
                .child(("ContestName", NS_EML), |w| w.text(name.as_ref())?.finish())?
                .finish()
        } else {
            writer.empty()
        }
    }
}

/// Identifier for the contest with 'geen' type.
#[derive(Debug, Clone)]
pub struct ContestIdentifierGeen {
    /// Id of the contest.
    pub id: StringValue<ContestIdGeen>,
}

impl ContestIdentifierGeen {
    /// Create a new `ContestIdentifierGeen`.
    pub fn new() -> Self {
        ContestIdentifierGeen {
            id: StringValue::Parsed(ContestIdGeen::new()),
        }
    }
}

impl Default for ContestIdentifierGeen {
    fn default() -> Self {
        Self::new()
    }
}

impl EMLElement for ContestIdentifierGeen {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ContestIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let id = elem.string_value_attr("Id", None)?;
        Ok(ContestIdentifierGeen { id })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer.attr("Id", self.id.raw().as_ref())?.empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{EMLParsingMode, EMLRead, test_write_eml_element, test_xml_fragment};

    #[test]
    fn test_contest_identifier_construction() {
        let contest_id = ContestIdentifier::new(ContestId::new("1234").unwrap());
        assert_eq!(contest_id.id.raw(), "1234");
        assert!(!contest_id.is_geen());
        assert!(!contest_id.is_alle());

        let contest_id_geen = ContestIdentifier::geen();
        assert_eq!(contest_id_geen.id.raw(), "geen");
        assert!(contest_id_geen.is_geen());
        assert!(!contest_id_geen.is_alle());

        let contest_id_alle = ContestIdentifier::alle();
        assert_eq!(contest_id_alle.id.raw(), "alle");
        assert!(!contest_id_alle.is_geen());
        assert!(contest_id_alle.is_alle());
    }

    #[test]
    fn test_contest_identifier_parsing() {
        let xml = test_xml_fragment(
            r#"<ContestIdentifier xmlns="urn:oasis:names:tc:evs:schema:eml" Id="1234"/>"#,
        );
        let contest_id = ContestIdentifier::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(contest_id.id.raw(), "1234");

        let xml_output = test_write_eml_element(&contest_id, &[NS_EML]).unwrap();
        assert_eq!(xml_output, xml);
    }

    #[test]
    fn test_contest_identifier_geen_construction() {
        let contest_id_geen = ContestIdentifierGeen::new();
        assert_eq!(contest_id_geen.id.raw(), "geen");
    }

    #[test]
    fn test_contest_identifier_geen_parsing() {
        let xml = test_xml_fragment(
            r#"<ContestIdentifier xmlns="urn:oasis:names:tc:evs:schema:eml" Id="geen"/>"#,
        );
        let contest_id_geen =
            ContestIdentifierGeen::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(contest_id_geen.id.raw(), "geen");

        let xml_output = test_write_eml_element(&contest_id_geen, &[NS_EML]).unwrap();
        assert_eq!(xml_output, xml);
    }
}
