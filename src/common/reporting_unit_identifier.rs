use crate::{
    EMLError, NS_EML,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName},
    utils::{ReportingUnitIdentifierId, StringValue},
};

/// Identifier for the reporting unit.
#[derive(Debug, Clone)]
pub struct ReportingUnitIdentifier {
    /// Id of the reporting unit.
    pub id: StringValue<ReportingUnitIdentifierId>,
    /// Name of the reporting unit.
    pub name: Box<str>,
}

impl ReportingUnitIdentifier {
    /// Create a new `ReportingUnitIdentifier` with the given id and name.
    pub fn new(id: impl Into<ReportingUnitIdentifierId>, name: impl Into<Box<str>>) -> Self {
        ReportingUnitIdentifier {
            id: StringValue::from_value(id.into()),
            name: name.into(),
        }
    }
}

impl EMLElement for ReportingUnitIdentifier {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ReportingUnitIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let name = elem.text_without_children()?;
        let id = elem.string_value_attr("Id", None)?;
        Ok(ReportingUnitIdentifier { id, name })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr("Id", self.id.raw().as_ref())?
            .text(self.name.as_ref())?
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        NS_EML,
        io::{EMLParsingMode, EMLRead as _, test_write_eml_element, test_xml_fragment},
    };

    #[test]
    fn test_reporting_unit_identifier_construction() {
        let id = ReportingUnitIdentifierId::new("1234").unwrap();
        let reporting_unit_identifier = ReportingUnitIdentifier::new(id, "Test");

        assert_eq!(reporting_unit_identifier.id.raw(), "1234");
        assert_eq!(reporting_unit_identifier.name.as_ref(), "Test");
    }

    #[test]
    fn test_reporting_unit_identifier_parsing() {
        let xml = test_xml_fragment(
            r#"
            <ReportingUnitIdentifier xmlns="urn:oasis:names:tc:evs:schema:eml" Id="1234">Test</ReportingUnitIdentifier>
            "#,
        );
        let reporting_unit_identifier =
            ReportingUnitIdentifier::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(reporting_unit_identifier.id.raw(), "1234");
        assert_eq!(reporting_unit_identifier.name.as_ref(), "Test");

        let xml_output = test_write_eml_element(&reporting_unit_identifier, &[NS_EML]).unwrap();
        assert_eq!(xml_output, xml);
    }
}
