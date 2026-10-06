use std::num::NonZeroU64;

use crate::{
    EMLError, NS_EML,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName, collect_struct},
    utils::{CandidateId, NameShortCode, StringValue},
};

/// Candidate identifier, but not for 510 document types.
#[derive(Debug, Clone)]
pub struct CandidateIdentifier {
    /// The candidate id.
    pub id: StringValue<CandidateId>,

    /// The display order of the candidate.
    pub display_order: Option<StringValue<NonZeroU64>>,

    /// The short code of the candidate, specified as the `ShortCode` attribute.
    ///
    /// This marks the place where the candidate is fully defined. Each short code
    /// attribute must be unique within the election.
    pub short_code: Option<StringValue<NameShortCode>>,

    /// The short code of the candidate, specified as a `ShortCode` child element.
    ///
    /// This refers to a candidate that is fully defined elsewhere in the document
    /// with a matching `short_code` attribute.
    pub short_code_ref: Option<StringValue<NameShortCode>>,

    /// The expected confirmation reference of the candidate.
    pub expected_confirmation_reference: Option<Box<str>>,
}

impl CandidateIdentifier {
    /// Create a new CandidateIdentifier.
    pub fn new(id: CandidateId) -> Self {
        CandidateIdentifier {
            id: StringValue::Parsed(id),
            display_order: None,
            short_code: None,
            short_code_ref: None,
            expected_confirmation_reference: None,
        }
    }

    /// Set the display order of the candidate.
    pub fn with_display_order(mut self, display_order: NonZeroU64) -> Self {
        self.display_order = Some(StringValue::Parsed(display_order));
        self
    }

    /// Set the short code of the candidate.
    pub fn with_short_code(mut self, short_code: NameShortCode) -> Self {
        self.short_code = Some(StringValue::Parsed(short_code));
        self
    }

    /// Set the short code reference of the candidate.
    pub fn with_short_code_ref(mut self, short_code: NameShortCode) -> Self {
        self.short_code_ref = Some(StringValue::Parsed(short_code));
        self
    }

    /// Set the expected confirmation reference of the candidate.
    pub fn with_expected_confirmation_reference(mut self, reference: impl Into<Box<str>>) -> Self {
        self.expected_confirmation_reference = Some(reference.into());
        self
    }

    /// Returns true if this identifier has a short code reference.
    pub fn has_reference(&self) -> bool {
        self.short_code_ref.is_some()
    }
}

impl From<CandidateId> for CandidateIdentifier {
    fn from(value: CandidateId) -> Self {
        CandidateIdentifier::new(value)
    }
}

impl EMLElement for CandidateIdentifier {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("CandidateIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, CandidateIdentifier {
            id: elem.string_value_attr("Id", None)?,
            display_order: elem.string_value_attr_opt("DisplayOrder")?,
            short_code: elem.string_value_attr_opt("ShortCode")?,
            short_code_ref as Option: ("ShortCode", NS_EML) => |elem| elem.string_value()?,
            expected_confirmation_reference: elem
                .attribute_value("ExpectedConfirmationReference")?
                .map(|s| s.into()),
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        let writer = writer
            .attr("Id", &self.id.raw())?
            .attr_opt("DisplayOrder", self.display_order.as_ref().map(|v| v.raw()))?
            .attr_opt("ShortCode", self.short_code.as_ref().map(|v| v.raw()))?
            .attr_opt(
                "ExpectedConfirmationReference",
                self.expected_confirmation_reference.as_ref(),
            )?;

        if let Some(short_code_ref) = &self.short_code_ref {
            writer
                .child(("ShortCode", NS_EML), |elem| {
                    elem.text(short_code_ref.raw().as_ref())?.finish()
                })?
                .finish()
        } else {
            writer.empty()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EMLVersion,
        io::{EMLParsingMode, EMLRead as _, test_write_eml_element, test_xml_fragment},
    };

    #[test]
    fn test_simple_candidate_identifier() {
        let xml = test_xml_fragment(
            r#"
            <CandidateIdentifier xmlns="urn:oasis:names:tc:evs:schema:eml" Id="1"/>
            "#,
        );
        let can_id = CandidateIdentifier::parse_eml_fragment(
            &xml,
            EMLParsingMode::Strict,
            EMLVersion::default(),
        )
        .unwrap();
        assert_eq!(
            can_id.id,
            StringValue::Parsed(CandidateId::new(NonZeroU64::new(1).unwrap()))
        );
        assert_eq!(can_id.display_order, None);
        assert_eq!(can_id.short_code, None);
        assert_eq!(can_id.short_code_ref, None);
        assert_eq!(can_id.expected_confirmation_reference, None);

        let xml_output = test_write_eml_element(&can_id, &[NS_EML], EMLVersion::default()).unwrap();
        assert_eq!(xml_output, xml);
    }

    #[test]
    fn test_all_attributes_candidate_identifier() {
        let xml = test_xml_fragment(
            r#"
            <CandidateIdentifier xmlns="urn:oasis:names:tc:evs:schema:eml" Id="2254" DisplayOrder="2" ShortCode="1234" ExpectedConfirmationReference="Ref123"/>
            "#,
        );
        let can_id = CandidateIdentifier::parse_eml_fragment(
            &xml,
            EMLParsingMode::Strict,
            EMLVersion::default(),
        )
        .unwrap();
        assert_eq!(
            can_id.id,
            StringValue::Parsed(CandidateId::new(NonZeroU64::new(2254).unwrap()))
        );
        assert_eq!(
            can_id.display_order,
            Some(StringValue::Parsed(NonZeroU64::new(2).unwrap()))
        );
        assert_eq!(
            can_id.short_code,
            Some(StringValue::Parsed(NameShortCode::new("1234").unwrap()))
        );
        assert_eq!(can_id.short_code_ref, None);
        assert_eq!(
            can_id.expected_confirmation_reference,
            Some("Ref123".into())
        );

        let xml_output = test_write_eml_element(&can_id, &[NS_EML], EMLVersion::default()).unwrap();
        assert_eq!(xml_output, xml);
    }

    #[test]
    fn test_short_code_element_candidate_identifier() {
        let xml = test_xml_fragment(
            r#"
            <CandidateIdentifier xmlns="urn:oasis:names:tc:evs:schema:eml" Id="3">
                <ShortCode>VlagW</ShortCode>
            </CandidateIdentifier>
            "#,
        );
        let can_id = CandidateIdentifier::parse_eml_fragment(
            &xml,
            EMLParsingMode::Strict,
            EMLVersion::default(),
        )
        .unwrap();
        assert_eq!(can_id.short_code, None);
        assert_eq!(
            can_id.short_code_ref,
            Some(StringValue::Parsed(NameShortCode::new("VlagW").unwrap()))
        );

        let xml_output = test_write_eml_element(&can_id, &[NS_EML], EMLVersion::default()).unwrap();
        assert_eq!(xml_output, xml);
    }

    #[test]
    fn test_candidate_identifier_construction() {
        let can_id = CandidateIdentifier::new(CandidateId::new(NonZeroU64::new(5678).unwrap()))
            .with_display_order(NonZeroU64::new(3).unwrap())
            .with_short_code(NameShortCode::new("9876").unwrap())
            .with_expected_confirmation_reference("reference");
        assert_eq!(
            can_id.id,
            StringValue::Parsed(CandidateId::new(NonZeroU64::new(5678).unwrap()))
        );
        assert_eq!(
            can_id.display_order,
            Some(StringValue::Parsed(NonZeroU64::new(3).unwrap()))
        );
        assert_eq!(
            can_id.short_code,
            Some(StringValue::Parsed(NameShortCode::new("9876").unwrap()))
        );
        assert_eq!(
            can_id.expected_confirmation_reference,
            Some("reference".into())
        );
    }
}
