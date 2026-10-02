use std::borrow::Cow;

use crate::{
    EMLError, NS_EML,
    common::{Contact, LivingAddress, PersonNameStructure},
    io::{
        EMLElement, EMLElementReader, EMLElementWriter, EMLReadElement, EMLWriteElement,
        QualifiedName, collect_struct,
    },
};

/// An agent acting on behalf of a candidate.
#[derive(Debug, Clone)]
pub struct Agent {
    /// The role of the agent (e.g. "H10" or "H10a").
    pub role: Option<String>,

    /// The agent's name.
    pub agent_identifier: AgentIdentifier,

    /// Contact details for the agent, if present.
    pub contact: Option<Contact>,

    /// The living address of the agent.
    pub living_address: LivingAddress,
}

impl EMLElement for Agent {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Agent", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, Agent {
            role: elem.attribute_value("Role")?.map(Cow::into_owned),
            agent_identifier: AgentIdentifier::EML_NAME => |elem| elem.read_element::<AgentIdentifier>()?,
            contact as Option: Contact::EML_NAME => |elem| elem.read_element::<Contact>()?,
            living_address: LivingAddress::EML_NAME => |elem| elem.read_element::<LivingAddress>()?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .attr_opt("Role", self.role.as_ref())?
            .child_elem(AgentIdentifier::EML_NAME, &self.agent_identifier)?
            .child_elem_option(Contact::EML_NAME, self.contact.as_ref())?
            .child_elem(LivingAddress::EML_NAME, &self.living_address)?
            .finish()
    }
}

/// Agent identifier containing the agent's name.
#[derive(Debug, Clone)]
pub struct AgentIdentifier {
    /// The agent's name.
    pub agent_name: PersonNameStructure,
}

impl AgentIdentifier {
    /// Create a new `AgentIdentifier`.
    pub fn new(agent_name: impl Into<PersonNameStructure>) -> Self {
        AgentIdentifier {
            agent_name: agent_name.into(),
        }
    }
}

impl EMLElement for AgentIdentifier {
    const EML_NAME: QualifiedName<'_, '_> =
        QualifiedName::from_static("AgentIdentifier", Some(NS_EML));

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError> {
        Ok(collect_struct!(elem, AgentIdentifier {
            agent_name: ("AgentName", NS_EML) => |elem| PersonNameStructure::read_eml_element(elem)?,
        }))
    }

    fn write_eml(&self, writer: EMLElementWriter) -> Result<(), EMLError> {
        writer
            .child(("AgentName", NS_EML), |writer| {
                self.agent_name.write_eml_element(writer)
            })?
            .finish()
    }
}
