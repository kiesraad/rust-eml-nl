use std::{borrow::Cow, collections::HashMap};

use quick_xml::{
    Writer,
    events::{BytesDecl, BytesStart, BytesText, Event, attributes::Attribute},
};

use crate::{EMLError, EMLErrorKind, EMLResultExt, NS_EML, NS_KR, NS_XAL, NS_XNL};

#[derive(Debug, Clone)]
pub struct NsDefinitions {
    default_namespace_uri: Option<&'static str>,
    namespace_definitions: HashMap<&'static str, &'static str>,
}

pub struct EMLWriter {
    ns_definitions: NsDefinitions,
    writer: Writer<Vec<u8>>,
}

impl EMLWriter {
    /// Resolves the namespace URI to a prefix defined previously.
    ///
    /// Note that there is a subtle difference between attributes and elements:
    /// If elements have no explicit namespace, then they are in the default
    /// namespace (as specified by the xmlns="" attribute). For attributes, if
    /// they have no explicit namespace, they are in no namespace at all.
    ///
    /// This writer requires each resolved element/attribute to specify its
    /// namespace URI explicitly. So if you need an element without any prefix
    /// but have defined a default namespace, you must specify that namespace to
    /// get no prefix.
    fn resolve_namespace_prefix(
        &self,
        namespace: &str,
        is_attribute: bool,
    ) -> Result<Option<&str>, EMLError> {
        if self.is_default_namespace(Some(namespace)) {
            if is_attribute {
                // Attributes cannot be in the default namespace unless there is
                // an explicit prefix for that URI as well, but this writer does
                // not support that.
                return Err(EMLErrorKind::AttributeNamespaceError).without_span();
            } else {
                return Ok(None);
            }
        }

        for (prefix, uri) in &self.ns_definitions.namespace_definitions {
            if *uri == namespace {
                return Ok(Some(*prefix));
            }
        }
        Err(EMLErrorKind::UnknownNamespace(namespace.to_string())).without_span()
    }

    /// Given an (optional) namespace URI and a local name, returns the
    /// qualified name that should be used when writing the element or attribute.
    ///
    /// This function resolves the namespace URI to a prefix using the previously
    /// defined namespaces initialized when initializing the EMLWriter.
    ///
    /// Note the difference in behavior for attributes and elements as described
    /// in `resolve_namespace_prefix`.
    fn format_qname<'b>(
        &self,
        name: &'b str,
        namespace: Option<&str>,
        is_attribute: bool,
    ) -> Result<Cow<'b, str>, EMLError> {
        let namespace_name = namespace
            .map(|n| self.resolve_namespace_prefix(n, is_attribute))
            .transpose()?
            .flatten();

        match namespace_name {
            Some(ns_name) => Ok(Cow::Owned(format!("{}:{}", ns_name, name))),
            None => Ok(Cow::Borrowed(name)),
        }
    }

    /// Checks if the given namespace URI is configured as the default namespace.
    fn is_default_namespace(&self, namespace: Option<&str>) -> bool {
        match (namespace, self.ns_definitions.default_namespace_uri) {
            (Some(ns), Some(def_ns)) => ns == def_ns,
            (None, None) => true,
            _ => false,
        }
    }

    /// Checks if there is a default namespace defined.
    fn has_default_namespace(&self) -> bool {
        self.ns_definitions.default_namespace_uri.is_some()
    }
}

pub struct EMLElementWriter<'a> {
    start_tag: BytesStart<'a>,
    writer: &'a mut EMLWriter,
}

impl<'a> EMLElementWriter<'a> {
    pub fn new(
        writer: &'a mut EMLWriter,
        name: &'a str,
        namespace: Option<&'a str>,
    ) -> Result<Self, EMLError> {
        let elem_name = writer.format_qname(name, namespace, false)?;
        if namespace.is_none() && writer.has_default_namespace() {
            // Technically this is something that XML allows, but as it is not
            // needed for EML we do not support it here.
            return Err(EMLErrorKind::ElementNamespaceError).without_span();
        }

        let start_tag = BytesStart::new(elem_name);
        Ok(EMLElementWriter { start_tag, writer })
    }

    pub fn attr(
        mut self,
        name: &str,
        namespace: Option<&str>,
        value: &str,
    ) -> Result<Self, EMLError> {
        let attr_name = self.writer.format_qname(name, namespace, true)?;
        self = self.attr_raw((attr_name.as_ref(), value));
        Ok(self)
    }

    fn attr_raw<'b>(mut self, attr: impl Into<Attribute<'b>>) -> Self {
        self.start_tag.push_attribute(attr);
        self
    }

    pub fn content(self) -> Result<EMLElementContentWriter<'a>, EMLError> {
        self.writer
            .writer
            .write_event(Event::Start(self.start_tag.borrow()))
            .without_span()?;
        Ok(EMLElementContentWriter {
            start_tag: self.start_tag,
            writer: self.writer,
        })
    }

    pub fn child(
        self,
        name: &str,
        namespace: Option<&str>,
        child_writer: impl FnOnce(EMLElementWriter) -> Result<(), EMLError>,
    ) -> Result<EMLElementContentWriter<'a>, EMLError> {
        self.content()?.child(name, namespace, child_writer)
    }

    pub fn text(self, text: &str) -> Result<EMLElementContentWriter<'a>, EMLError> {
        self.content()?.text(text)
    }

    pub fn finish(self) -> Result<(), EMLError> {
        self.content()?.finish()
    }

    pub fn empty(self) -> Result<(), EMLError> {
        self.writer
            .writer
            .write_event(Event::Empty(self.start_tag.borrow()))
            .without_span()?;
        Ok(())
    }
}

pub struct EMLElementContentWriter<'a> {
    start_tag: BytesStart<'a>,
    writer: &'a mut EMLWriter,
}

impl<'a> EMLElementContentWriter<'a> {
    pub fn child(
        self,
        name: &str,
        namespace: Option<&str>,
        child_writer: impl FnOnce(EMLElementWriter) -> Result<(), EMLError>,
    ) -> Result<Self, EMLError> {
        let elem_writer = EMLElementWriter::new(self.writer, name, namespace)?;
        child_writer(elem_writer)?;
        Ok(self)
    }

    pub fn text(self, text: &str) -> Result<Self, EMLError> {
        self.writer
            .writer
            .write_event(Event::Text(BytesText::new(text)))
            .without_span()?;
        Ok(self)
    }

    pub fn finish(self) -> Result<(), EMLError> {
        self.writer
            .writer
            .write_event(quick_xml::events::Event::End(self.start_tag.to_end()))
            .without_span()?;
        Ok(())
    }
}

pub trait EMLWrite {
    fn write_eml_element(&self, writer: EMLElementWriter) -> Result<(), EMLError>;

    fn write_root(
        &self,
        root_name: Option<&str>,
        default_namespace_uri: Option<Option<&'static str>>,
        namespace_definitions: Option<HashMap<&'static str, &'static str>>,
        pretty_print: bool,
        include_declaration: bool,
    ) -> Result<Vec<u8>, EMLError> {
        // default values are for EML root element
        let root = root_name.unwrap_or("EML");
        let default_namespace_uri = default_namespace_uri.unwrap_or(Some(NS_EML));
        let namespace_definitions = namespace_definitions.unwrap_or_else(|| {
            let mut ns_defs = HashMap::new();
            ns_defs.insert("kr", NS_KR);
            ns_defs.insert("xal", NS_XAL);
            ns_defs.insert("xnl", NS_XNL);
            // ns_defs.insert("ds", NS_DS);
            // ns_defs.insert("xmlns", NS_XMLNS);
            // ns_defs.insert("xml", NS_XML);
            ns_defs
        });

        let ns_definitions = NsDefinitions {
            default_namespace_uri,
            namespace_definitions,
        };

        let mut writer = if pretty_print {
            Writer::new_with_indent(Vec::new(), b' ', 4)
        } else {
            Writer::new(Vec::new())
        };

        if include_declaration {
            writer
                .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
                .without_span()?;
        }
        let mut eml_writer = EMLWriter {
            ns_definitions: ns_definitions.clone(),
            writer,
        };
        let mut element = EMLElementWriter::new(&mut eml_writer, root, default_namespace_uri)?;
        if let Some(ns_uri) = ns_definitions.default_namespace_uri {
            element = element.attr_raw(("xmlns", ns_uri));
        }
        for (prefix, uri) in &ns_definitions.namespace_definitions {
            element = element.attr_raw((format!("xmlns:{}", *prefix).as_str(), *uri));
        }
        self.write_eml_element(element)?;
        Ok(eml_writer.writer.into_inner())
    }

    fn write_eml_root(
        &self,
        pretty_print: bool,
        include_declaration: bool,
    ) -> Result<Vec<u8>, EMLError> {
        self.write_root(None, None, None, pretty_print, include_declaration)
    }
}

pub fn write_eml_element(
    element: &impl EMLWrite,
) -> impl FnOnce(EMLElementWriter) -> Result<(), EMLError> {
    return |writer| element.write_eml_element(writer);
}
