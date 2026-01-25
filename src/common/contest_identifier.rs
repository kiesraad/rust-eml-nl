use crate::{
    EMLError, NS_EML,
    io::{EMLElement, EMLElementReader, EMLElementWriter, QualifiedName},
    utils::{ContestIdType, ContestIdTypeGeen, StringValue},
};

/// Identifier for the contest.
#[derive(Debug, Clone)]
pub struct ContestIdentifier {
    /// Id of the contest.
    pub id: StringValue<ContestIdType>,
}

impl EMLElement for ContestIdentifier {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("ContestIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        let id = elem.string_value_attr("Id", None)?;
        Ok(ContestIdentifier { id })
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer.attr("Id", self.id.raw().as_ref())?.empty()
    }
}

/// Identifier for the contest with 'geen' type.
#[derive(Debug, Clone)]
pub struct ContestIdentifierGeen {
    /// Id of the contest.
    pub id: StringValue<ContestIdTypeGeen>,
}

impl ContestIdentifierGeen {
    /// Create a new `ContestIdentifierGeen`.
    pub fn new() -> Self {
        ContestIdentifierGeen {
            id: StringValue::Parsed(ContestIdTypeGeen::new()),
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
