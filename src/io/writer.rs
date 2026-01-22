use std::{borrow::Cow, collections::HashMap};

use quick_xml::{
    Writer,
    events::{BytesDecl, BytesStart, BytesText, Event, attributes::Attribute},
};

use crate::{
    EMLError, EMLErrorKind, EMLResultExt, NS_EML, NS_KR, NS_XAL, NS_XNL, io::QualifiedName,
};

#[derive(Debug, Clone)]
pub(crate) struct NsDefinitions {
    default_namespace_uri: Option<&'static str>,
    namespace_definitions: HashMap<&'static str, &'static str>,
}

pub(crate) struct EMLWriter {
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
    fn format_qname<'b, 'c>(
        &self,
        name: &'b QualifiedName<'b, 'c>,
        is_attribute: bool,
    ) -> Result<Cow<'b, str>, EMLError> {
        let namespace_name = name
            .namespace
            .as_ref()
            .map(|n| self.resolve_namespace_prefix(n.as_ref(), is_attribute))
            .transpose()?
            .flatten();

        match namespace_name {
            Some(ns_name) => Ok(Cow::Owned(format!(
                "{}:{}",
                ns_name,
                name.local_name.as_ref()
            ))),
            None => Ok(Cow::Borrowed(name.local_name.as_ref())),
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

pub(crate) struct EMLElementWriter<'a> {
    start_tag: BytesStart<'a>,
    writer: &'a mut EMLWriter,
}

impl<'a> EMLElementWriter<'a> {
    pub(crate) fn new(
        writer: &'a mut EMLWriter,
        name: &'a QualifiedName<'a, 'a>,
    ) -> Result<Self, EMLError> {
        let elem_name = writer.format_qname(name, false)?;
        if name.namespace.is_none() && writer.has_default_namespace() {
            // Technically this is something that XML allows, but as it is not
            // needed for EML we do not support it here.
            return Err(EMLErrorKind::ElementNamespaceError).without_span();
        }

        let start_tag = BytesStart::new(elem_name);
        Ok(EMLElementWriter { start_tag, writer })
    }

    pub fn attr<'b, 'c>(
        mut self,
        name: impl Into<QualifiedName<'b, 'c>>,
        value: &str,
    ) -> Result<Self, EMLError> {
        let name = name.into();
        let attr_name = self.writer.format_qname(&name, true)?;
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

    pub fn child_option<'b, 'c, T>(
        self,
        name: impl Into<QualifiedName<'b, 'c>>,
        value: Option<T>,
        child_writer: impl FnOnce(EMLElementWriter, T) -> Result<(), EMLError>,
    ) -> Result<EMLElementContentWriter<'a>, EMLError> {
        self.content()?.child_option(name, value, child_writer)
    }

    pub fn child<'b, 'c>(
        self,
        name: impl Into<QualifiedName<'b, 'c>>,
        child_writer: impl FnOnce(EMLElementWriter) -> Result<(), EMLError>,
    ) -> Result<EMLElementContentWriter<'a>, EMLError> {
        self.content()?.child(name, child_writer)
    }

    pub fn child_elem<'b, 'c>(
        self,
        name: impl Into<QualifiedName<'b, 'c>>,
        value: &impl EMLWriteElement,
    ) -> Result<EMLElementContentWriter<'a>, EMLError> {
        self.content()?.child_elem(name, value)
    }

    #[expect(unused)]
    pub fn child_elem_option<'b, 'c>(
        self,
        name: impl Into<QualifiedName<'b, 'c>>,
        value: Option<&impl EMLWriteElement>,
    ) -> Result<EMLElementContentWriter<'a>, EMLError> {
        self.content()?.child_elem_option(name, value)
    }

    pub fn text(self, text: &str) -> Result<EMLElementContentWriter<'a>, EMLError> {
        self.content()?.text(text)
    }

    pub fn empty(self) -> Result<(), EMLError> {
        self.writer
            .writer
            .write_event(Event::Empty(self.start_tag.borrow()))
            .without_span()?;
        Ok(())
    }
}

pub(crate) struct EMLElementContentWriter<'a> {
    start_tag: BytesStart<'a>,
    writer: &'a mut EMLWriter,
}

impl<'a> EMLElementContentWriter<'a> {
    pub fn child<'b, 'c>(
        self,
        name: impl Into<QualifiedName<'b, 'c>>,
        child_writer: impl FnOnce(EMLElementWriter) -> Result<(), EMLError>,
    ) -> Result<Self, EMLError> {
        let name = name.into();
        let elem_writer = EMLElementWriter::new(self.writer, &name)?;
        child_writer(elem_writer)?;
        Ok(self)
    }

    pub fn child_option<'b, 'c, T>(
        self,
        name: impl Into<QualifiedName<'b, 'c>>,
        value: Option<T>,
        child_writer: impl FnOnce(EMLElementWriter, T) -> Result<(), EMLError>,
    ) -> Result<EMLElementContentWriter<'a>, EMLError> {
        if let Some(v) = value {
            self.child(name, |w| child_writer(w, v))
        } else {
            Ok(self)
        }
    }

    pub fn child_elem<'b, 'c>(
        self,
        name: impl Into<QualifiedName<'b, 'c>>,
        value: &impl EMLWriteElement,
    ) -> Result<Self, EMLError> {
        self.child(name, write_eml_element(value))
    }

    pub fn child_elem_option<'b, 'c>(
        self,
        name: impl Into<QualifiedName<'b, 'c>>,
        value: Option<&impl EMLWriteElement>,
    ) -> Result<EMLElementContentWriter<'a>, EMLError> {
        self.child_option(name, value, |writer, value| {
            write_eml_element(value)(writer)
        })
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

pub(crate) trait EMLWriteInternal {
    fn write_root<'a, 'b>(
        &self,
        root_name: Option<impl Into<QualifiedName<'a, 'b>>>,
        default_namespace_uri: Option<Option<&'static str>>,
        namespace_definitions: Option<HashMap<&'static str, &'static str>>,
        pretty_print: bool,
        include_declaration: bool,
    ) -> Result<Vec<u8>, EMLError>;

    fn write_root_str<'a, 'b>(
        &self,
        root_name: Option<impl Into<QualifiedName<'a, 'b>>>,
        default_namespace_uri: Option<Option<&'static str>>,
        namespace_definitions: Option<HashMap<&'static str, &'static str>>,
        pretty_print: bool,
        include_declaration: bool,
    ) -> Result<String, EMLError>;
}

impl<T> EMLWriteInternal for T
where
    T: EMLWriteElement,
{
    fn write_root<'a, 'b>(
        &self,
        root_name: Option<impl Into<QualifiedName<'a, 'b>>>,
        default_namespace_uri: Option<Option<&'static str>>,
        namespace_definitions: Option<HashMap<&'static str, &'static str>>,
        pretty_print: bool,
        include_declaration: bool,
    ) -> Result<Vec<u8>, EMLError> {
        // default values are for EML root element
        let root = root_name
            .map(|v| v.into())
            .unwrap_or_else(|| QualifiedName::new("EML", Some(NS_EML)));
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
        let mut element = EMLElementWriter::new(&mut eml_writer, &root)?;
        if let Some(ns_uri) = ns_definitions.default_namespace_uri {
            element = element.attr_raw(("xmlns", ns_uri));
        }
        for (prefix, uri) in &ns_definitions.namespace_definitions {
            element = element.attr_raw((format!("xmlns:{}", *prefix).as_str(), *uri));
        }
        self.write_eml_element(element)?;
        Ok(eml_writer.writer.into_inner())
    }

    fn write_root_str<'a, 'b>(
        &self,
        root_name: Option<impl Into<QualifiedName<'a, 'b>>>,
        default_namespace_uri: Option<Option<&'static str>>,
        namespace_definitions: Option<HashMap<&'static str, &'static str>>,
        pretty_print: bool,
        include_declaration: bool,
    ) -> Result<String, EMLError> {
        String::from_utf8(self.write_root(
            root_name,
            default_namespace_uri,
            namespace_definitions,
            pretty_print,
            include_declaration,
        )?)
        .without_span()
    }
}

/// Writing EML documents to a [`String`] or [`Vec<u8>`].
///
/// The errors generated during writing do not contain location information, as
/// there is no document to refer to yet. Most of the time errors generated
/// during writing are underlying errors or logic errors in your implementation,
/// so location information would be of limited use anyway.
pub trait EMLWrite {
    /// Writes an EML document with an EML root element to a byte vector.
    fn write_eml_root(
        &self,
        pretty_print: bool,
        include_declaration: bool,
    ) -> Result<Vec<u8>, EMLError>;

    /// Writes an EML document with an EML root element to a string.
    fn write_eml_root_str(
        &self,
        pretty_print: bool,
        include_declaration: bool,
    ) -> Result<String, EMLError>;
}

impl<T> EMLWrite for T
where
    T: EMLWriteInternal,
{
    fn write_eml_root(
        &self,
        pretty_print: bool,
        include_declaration: bool,
    ) -> Result<Vec<u8>, EMLError> {
        self.write_root(
            None::<QualifiedName<'_, '_>>,
            None,
            None,
            pretty_print,
            include_declaration,
        )
    }

    /// Writes an EML document with an EML root element to a string.
    fn write_eml_root_str(
        &self,
        pretty_print: bool,
        include_declaration: bool,
    ) -> Result<String, EMLError> {
        self.write_root_str(
            None::<QualifiedName<'_, '_>>,
            None,
            None,
            pretty_print,
            include_declaration,
        )
    }
}

pub(crate) trait EMLWriteElement {
    fn write_eml_element(&self, writer: EMLElementWriter) -> Result<(), EMLError>;
}

pub(crate) fn write_eml_element(
    element: &impl EMLWriteElement,
) -> impl FnOnce(EMLElementWriter) -> Result<(), EMLError> {
    |writer| element.write_eml_element(writer)
}
