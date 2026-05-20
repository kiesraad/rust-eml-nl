use std::num::NonZeroU64;

use instant_xml::{FromXml, ToXml};

use crate::{
    NS_EML,
    utils::{CandidateId, NameShortCode, StringValue},
};

/// Candidate identifier, but not for 510 document types.
#[derive(Debug, Clone, FromXml, ToXml)]
#[xml(rename_all = "PascalCase", ns(NS_EML))]
pub struct CandidateIdentifier {
    /// The candidate id.
    #[xml(attribute)]
    pub id: StringValue<CandidateId>,

    /// The display order of the candidate.
    #[xml(attribute)]
    pub display_order: Option<StringValue<NonZeroU64>>,

    /// The short code of the candidate.
    ///
    /// Note: This can be specified either as an attribute or as a child
    /// element, but the attribute takes precedence if both are present.
    /// Additionally we only ever output it as an attribute for simplicity.
    #[xml(attribute)]
    pub short_code: Option<StringValue<NameShortCode>>,

    /// The expected confirmation reference of the candidate.
    #[xml(attribute)]
    pub expected_confirmation_reference: Option<String>,
}

impl CandidateIdentifier {
    /// Create a new CandidateIdentifier.
    pub fn new(id: CandidateId) -> Self {
        CandidateIdentifier {
            id: StringValue::Parsed(id),
            display_order: None,
            short_code: None,
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

    /// Set the expected confirmation reference of the candidate.
    pub fn with_expected_confirmation_reference(mut self, reference: impl Into<String>) -> Self {
        self.expected_confirmation_reference = Some(reference.into());
        self
    }
}

impl From<CandidateId> for CandidateIdentifier {
    fn from(value: CandidateId) -> Self {
        CandidateIdentifier::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{EMLParsingMode, EMLRead as _, test_xml_fragment};

    #[test]
    fn test_simple_candidate_identifier() {
        let xml = test_xml_fragment(
            r#"
            <CandidateIdentifier xmlns="urn:oasis:names:tc:evs:schema:eml" Id="1"/>
            "#,
        );
        let can_id = CandidateIdentifier::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
        assert_eq!(
            can_id.id,
            StringValue::Parsed(CandidateId::new(NonZeroU64::new(1).unwrap()))
        );
        assert_eq!(can_id.display_order, None);
        assert_eq!(can_id.short_code, None);
        assert_eq!(can_id.expected_confirmation_reference, None);
    }

    #[test]
    fn test_all_attributes_candidate_identifier() {
        let xml = test_xml_fragment(
            r#"
            <CandidateIdentifier xmlns="urn:oasis:names:tc:evs:schema:eml" Id="2254" DisplayOrder="2" ShortCode="1234" ExpectedConfirmationReference="Ref123"/>
            "#,
        );
        let can_id = CandidateIdentifier::parse_eml(&xml, EMLParsingMode::Strict).unwrap();
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
        assert_eq!(
            can_id.expected_confirmation_reference,
            Some("Ref123".to_string())
        );
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
            Some("reference".to_string())
        );
    }
}
