use std::{borrow::Cow, collections::HashMap};

use quick_xml::{
    NsReader,
    escape::unescape,
    events::{BytesStart, Event},
    name::{QName, ResolveResult},
};

use crate::{
    error::{EMLError, EMLErrorKind, EMLResultExt},
    io::QualifiedName,
};

/// Reading EML documents from a string slice.
pub trait EMLRead {
    /// Parse an EML document from the given string slice.
    ///
    /// The `strict_value_parsing` parameter indicates whether strict parsing of
    /// values (e.g. dates, numbers) should be performed. If set to false, values
    /// that cannot be parsed will be stored as raw strings instead. If set to
    /// true, parsing errors will result in an error being returned.
    fn parse_eml(input: &str, strict_value_parsing: bool) -> Result<Self, EMLError>
    where
        Self: Sized;
}

impl<T> EMLRead for T
where
    T: EMLReadElement,
{
    fn parse_eml(input: &str, strict_value_parsing: bool) -> Result<Self, EMLError>
    where
        Self: Sized,
    {
        let mut reader = EMLReader::init_from_str(input, strict_value_parsing);
        let mut root = reader.next_element()?;
        T::read_eml_element(&mut root)
    }
}

/// This trait should be implemented by all types that can be parsed from EML files.
pub(crate) trait EMLReadElement {
    fn read_eml_element(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, EMLError>
    where
        Self: Sized;
}

/// A span in the input data, represented as byte offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Start byte offset of the span (inclusive).
    pub start: u64,
    /// End byte offset of the span (exclusive).
    pub end: u64,
}

impl Span {
    /// Create a new span from the given start and end byte offsets.
    pub fn new(start: u64, end: u64) -> Span {
        Span { start, end }
    }
}

/// The main EML XML reader.
///
/// We require all EML files to be fully loaded in memory, so this reader only
/// works on byte slices. Furthermore, all files should be encoded in UTF-8.
pub(crate) struct EMLReader<'a> {
    inner: NsReader<&'a [u8]>,
    strict_value_parsing: bool,
}

impl<'a> EMLReader<'a> {
    /// Create this reader from a string slice.
    pub fn init_from_str(data: &'a str, strict_value_parsing: bool) -> EMLReader<'a> {
        Self::from_reader(NsReader::from_str(data), strict_value_parsing)
    }

    pub fn from_reader(reader: NsReader<&'a [u8]>, strict_value_parsing: bool) -> EMLReader<'a> {
        EMLReader {
            inner: reader,
            strict_value_parsing,
        }
    }

    fn next(&mut self) -> Result<(Event<'a>, Span), EMLError> {
        let span_start = self.inner.buffer_position();
        let event = self.inner.read_event();
        let event = match event {
            Ok(evt) => evt,
            Err(xml_err) => {
                let error_pos = self.inner.error_position();
                if error_pos == 0 {
                    // quick-xml returns error position 0 when it doesn't have an error yet,
                    // but if we do end up here we know the error must have happened somewhere
                    // after the end of the previous event and where-ever the current buffer
                    // position is.
                    return Err(xml_err)
                        .with_span(Span::new(span_start, self.inner.buffer_position()));
                } else {
                    return Err(xml_err).with_span(Span::new(error_pos, error_pos));
                }
            }
        };
        let span = Span::new(span_start, self.inner.buffer_position());
        Ok((event, span))
    }

    pub fn next_element<'tmp>(&'a mut self) -> Result<EMLElementReader<'tmp, 'a>, EMLError> {
        loop {
            match self.next()? {
                (Event::Start(start), span) => {
                    return Ok(EMLElementReader::from_start(self, start, false, span));
                }
                (Event::Empty(start), span) => {
                    return Ok(EMLElementReader::from_start(self, start, true, span));
                }
                _other => {
                    // Ignore other events
                }
            }
        }
    }
}

/// A reader for an XML element in an EML file.
///
/// This reader tries to ensure that the entire element is consumed before it
/// is dropped, but it is recommended to explicitly call `skip` to completely
/// consume the element.
pub(crate) struct EMLElementReader<'r, 'input> {
    reader: &'r mut EMLReader<'input>,
    start: BytesStart<'input>,
    depth: usize,
    found_matching_end: bool,
    is_empty: bool,
    span: Span,
    last_span: Span,
}

impl<'r, 'input> EMLElementReader<'r, 'input> {
    /// Given a start event that was just read from the reader, create a element
    /// reader until the matching end tag. If the start event was an empty
    /// element, this must be indicated using the `is_empty` parameter, otherwise
    /// the reader will parse this document invalidly.
    ///
    /// The span should be the span of the start event.
    pub fn from_start(
        reader: &'r mut EMLReader<'input>,
        start: BytesStart<'input>,
        is_empty: bool,
        span: Span,
    ) -> EMLElementReader<'r, 'input> {
        EMLElementReader {
            reader,
            start,
            depth: 1,
            found_matching_end: is_empty,
            is_empty,
            span,
            last_span: span,
        }
    }

    /// Extracts the resolved name of this element as a tuple of local name and
    /// an optional namespace URI.
    pub fn name(&self) -> Result<QualifiedName<'_, '_>, EMLError> {
        self.get_resolved_name(self.start.name(), self.span, false)
    }

    /// Checks if this element has the given local name and optional namespace URI.
    pub fn has_name<'a, 'b>(
        &self,
        name: impl Into<QualifiedName<'a, 'b>>,
    ) -> Result<bool, EMLError> {
        self.is_resolved_name(self.start.name(), self.span, name, false)
    }

    /// Find the next child element of this element and return a reader for that
    /// part of the document. The returned reader must be fully consumed before
    /// continuing to read from this element. If the entire element is consumed,
    /// this will return None.
    pub fn next_child(&mut self) -> Result<Option<EMLElementReader<'_, 'input>>, EMLError> {
        loop {
            match self.next()? {
                Some((Event::Start(start), span)) => {
                    self.depth -= 1; // the child must handle the end tag itself
                    return Ok(Some(EMLElementReader::from_start(
                        self.reader,
                        start,
                        false,
                        span,
                    )));
                }
                Some((Event::Empty(start), span)) => {
                    return Ok(Some(EMLElementReader::from_start(
                        self.reader,
                        start,
                        true,
                        span,
                    )));
                }
                None => return Ok(None),
                _other => {
                    // Ignore other events
                }
            }
        }
    }

    /// Get the value of an attribute. If the attribute does not exist this will
    /// return an error.
    pub fn attribute_value_req<'a, 'b>(
        &self,
        name: impl Into<QualifiedName<'a, 'b>>,
    ) -> Result<Cow<'_, str>, EMLError> {
        let name = name.into();
        self.attribute_value(name.clone())?
            .ok_or_else(|| EMLErrorKind::MissingAttribute(name.as_owned()))
            .with_span(self.span)
    }

    /// Get the value of an attribute. If the attribute does not exist this will
    /// return None.
    pub fn attribute_value<'a, 'b>(
        &self,
        name: impl Into<QualifiedName<'a, 'b>>,
    ) -> Result<Option<Cow<'_, str>>, EMLError> {
        let name = name.into();
        // quick-xml does not expose any way to get the span of individual attributes, so we use the whole start tag span for now
        for attr in self.start.attributes() {
            let attr = attr.with_span(self.span)?;
            if self.is_resolved_name(attr.key, self.span, name.clone(), true)? {
                return Ok(Some(
                    attr.decode_and_unescape_value(self.reader.inner.decoder())
                        .with_span(self.span)?,
                ));
            }
        }
        Ok(None)
    }

    /// Get a hasmap of all attributes of the start tag of this element.
    #[expect(unused)]
    pub fn attributes(&self) -> Result<HashMap<QualifiedName<'_, '_>, Cow<'_, str>>, EMLError> {
        let mut attributes = HashMap::new();
        // quick-xml does not expose any way to get the span of individual attributes, so we use the whole start tag span for now
        for attr in self.start.attributes() {
            let attr = attr.with_span(self.span)?;
            let name = self.get_resolved_name(attr.key, self.span, true)?;
            let value = attr
                .decode_and_unescape_value(self.reader.inner.decoder())
                .with_span(self.span)?;
            attributes.insert(name, value);
        }
        Ok(attributes)
    }

    /// Extracts the text content of this element, consuming all events until
    /// the end of the element. If anything other than text is found, this will
    /// return an error (not consuming everything).
    pub fn text_without_children(&mut self) -> Result<String, EMLError> {
        let mut text = String::new();
        loop {
            match self.next()? {
                Some((Event::Text(t), span)) => {
                    let decoded = t.xml_content().with_span(span)?;
                    text.push_str(decoded.as_ref());
                }
                Some((Event::CData(t), span)) => {
                    let decoded = t.xml_content().with_span(span)?;
                    text.push_str(decoded.as_ref());
                }
                Some((Event::GeneralRef(r), span)) => {
                    let ref_name = r.decode().with_span(span)?;
                    let formatted_entity = format!("&{};", ref_name);

                    text.push_str(unescape(&formatted_entity).with_span(span)?.as_ref());
                }
                Some((Event::Comment(_), _)) => {
                    // Ignore comments
                }
                None => break,
                Some((_other, span)) => {
                    return Err(EMLErrorKind::UnexpectedEvent).with_span(span);
                }
            }
        }
        Ok(text)
    }

    /// Skip all remaining content/events in this element. Stops reading just
    /// after the matching end tag.
    pub fn skip(&mut self) -> Result<(), EMLError> {
        while let Some(_evt) = self.next()? {
            // Just consume events until the end of this element
        }
        Ok(())
    }

    /// Returns the span of the start tag of this element.
    pub fn span(&self) -> Span {
        self.span
    }

    /// Returns the span of the last event that was read from this element.
    pub fn last_span(&self) -> Span {
        self.last_span
    }

    /// Returns the full span of this element up until the current event,
    /// including the start tag. If the entire element has been consumed, this
    /// will return the full span of the element.
    #[expect(unused)]
    pub fn full_span(&self) -> Span {
        Span::new(self.span.start, self.last_span.end)
    }

    /// Returns the inner span of this element, excluding the start tag. The
    /// returned span does not include the span of the last read event. If the
    /// entire element has been consumed, this will return the span between the
    /// start and end tags.
    pub fn inner_span(&self) -> Span {
        Span::new(self.span.end, self.last_span.start)
    }

    /// Returns whether strict value parsing is enabled for this reader
    pub fn strict_value_parsing(&self) -> bool {
        self.reader.strict_value_parsing
    }

    /// Extracts the namespace URI from a ResolveResult.
    fn namespace_name<'a>(
        &self,
        resolve_result: ResolveResult<'a>,
        span: Span,
    ) -> Result<Option<Cow<'a, str>>, EMLError> {
        match resolve_result {
            ResolveResult::Bound(namespace) => Ok(Some(
                self.reader
                    .inner
                    .decoder()
                    .decode(namespace.into_inner())
                    .with_span(span)?,
            )),
            ResolveResult::Unbound => Ok(None),
            ResolveResult::Unknown(scope) => Err(EMLErrorKind::UnknownNamespace(
                self.reader
                    .inner
                    .decoder()
                    .decode(&scope)
                    .with_span(span)?
                    .into_owned(),
            ))
            .with_span(span),
        }
    }

    /// Checks if the given qualified name is of the expected local name and
    /// optional namespace URI.
    fn is_resolved_name<'a, 'b, 'c>(
        &self,
        name: QName<'a>,
        span: Span,
        expected_name: impl Into<QualifiedName<'b, 'c>>,
        is_attribute: bool,
    ) -> Result<bool, EMLError> {
        let expected_name = expected_name.into();
        let resolved_name = self.get_resolved_name(name, span, is_attribute)?;
        let matches_local = resolved_name.local_name.as_ref() == expected_name.local_name.as_ref();
        let matches_namespace = match (
            expected_name.namespace.as_deref(),
            resolved_name.namespace.as_deref(),
        ) {
            (Some(expected), Some(found)) => expected == found,
            (None, None) => true,
            _ => false,
        };
        Ok(matches_local && matches_namespace)
    }

    /// Extracts the resolved local name and optional namespace URI from the
    /// given qualified name (i.e. name that may include a prefix such as
    /// `xmlns:eml`)
    fn get_resolved_name<'a>(
        &'a self,
        name: QName<'a>,
        span: Span,
        is_attribute: bool,
    ) -> Result<QualifiedName<'a, 'a>, EMLError> {
        let (resolved, local_name) = if is_attribute {
            self.reader.inner.resolver().resolve_attribute(name)
        } else {
            self.reader.inner.resolver().resolve_element(name)
        };
        let namespace = self.namespace_name(resolved, span)?;
        let local_name = self
            .reader
            .inner
            .decoder()
            .decode(local_name.into_inner())
            .with_span(span)?;

        Ok(QualifiedName::new(local_name, namespace))
    }

    /// Reads the next event from this element, returning None if the end of
    /// this element has been reached.
    fn next(&mut self) -> Result<Option<(Event<'input>, Span)>, EMLError> {
        if self.found_matching_end {
            return Ok(None);
        }

        let (evt, span) = self.reader.next()?;
        self.last_span = span;
        if matches!(evt, Event::Start(_)) {
            self.depth += 1;
        }

        if matches!(evt, Event::End(_)) {
            self.depth -= 1;
        }

        if matches!(evt, Event::Eof) {
            return Err(EMLErrorKind::UnexpectedEof).with_span(span);
        }

        if self.depth == 0
            && let Event::End(e) = &evt
        {
            if e.name() == self.start.name() {
                self.found_matching_end = true;
                return Ok(None);
            } else {
                return Err(EMLErrorKind::UnexpectedEndElement).with_span(span);
            }
        }

        Ok(Some((evt, span)))
    }

    /// Returns whether this element is empty (i.e., has no content and no end tag).
    pub fn is_empty(&self) -> bool {
        self.is_empty
    }
}

impl Drop for EMLElementReader<'_, '_> {
    fn drop(&mut self) {
        // Ensure we have consumed the entire element
        let _ = self.skip();
    }
}

macro_rules! collect_struct {
    // This macro starts by first matching the external syntax and converting
    // that to an internal representation that can be more easily processed.
    // Once all tokens have been processed, we continue with the @emit rule.
    // In this phase, we output the base structure of the code. In this phase,
    // we again delegate to other rules to output specific parts of the code.
    // These parts are: @decl: declares temporary variables that
    // will hold the parsed values; @matcher: code to check for each field while
    // reading children from the XML element; and @assign: code to assign the
    // final values to the struct fields. This final part once again uses a
    // recursive approach to output the assignments one by one because of
    // limitations in macro_rules! that prevent us from directly outputting the
    // list expansions as one (Rust stops expanding once it sees a macro in a
    // field position and then fails with a syntax error).

    // entry point of the macro, forward to expand rules
    ( $root:expr, $ty:ident { $($rest:tt)* }) => {
        collect_struct!(@expand [$root] [$ty] [] $($rest)* )
    };

    // accumulate for a normal row
    ( @expand [$root:expr] [$ty:ident] [$($items:tt ; )*]
        $field:ident: $namespaced_name:expr => |$var:ident| $map:expr ,
        $($tail:tt)*
    ) => {
        collect_struct!(@expand [$root] [$ty] [
            $($items ; )*
            (@field [$field] [$namespaced_name] [$var] [$map]) ;
        ] $($tail)*)
    };

    // accumulate, for an option row
    ( @expand [$root:expr] [$ty:ident] [$($items:tt ; )*]
        $field:ident as Option: $namespaced_name:expr => |$var:ident| $map:expr ,
        $($tail:tt)*
    ) => {
        collect_struct!(@expand [$root] [$ty] [
            $($items ; )*
            (@optional [$field] [$namespaced_name] [$var] [$map]) ;
        ] $($tail)*)
    };

    // accumulate for a direct row
    ( @expand [$root:expr] [$ty:ident] [$($items:tt ; )*]
        $field:ident: $value:expr ,
        $($tail:tt)*
    ) => {
        collect_struct!(@expand [$root] [$ty] [
            $($items ; )*
            (@direct [$field] [$value]) ;
        ] $($tail)*)
    };

    // accumulation of items completed, start emitting
    ( @expand [$root:expr] [$ty:ident] [$($items:tt ; )*] ) => {
        collect_struct!(@emit [$root] [$ty] [$($items ; )*])
    };

    // Emit the actual code to read the struct
    ( @emit [$root:expr] [$ty:ident] [$($items:tt ; )*] ) => {{
        $( collect_struct!(@decl $items); )*

        while let Some(mut next_child) = $root.next_child()? {
            let name = next_child.name()?.as_owned().into_inner();
            let mut handled = false;

            $( collect_struct!(@matcher next_child, name, handled, $items); )*

            if !handled {
                // Unknown element at this level
                next_child.skip()?;
            }
        }

        collect_struct!(@assign $root, $ty, [], $($items ; )*)
    }};

    // Emit field declarations
    (@decl (@direct [$field:ident] [$value:expr])) => {};
    (@decl (@optional [$field:ident] [$namespaced_name:expr] [$var:ident] [$map:expr])) => {
        collect_struct!(@decl (@field [$field] [$namespaced_name] [$var] [$map]));
    };
    (@decl (@field [$field:ident] [$namespaced_name:expr] [$var:ident] [$map:expr])) => {
        let mut $field: Option<_> = None;
    };

    // Emit match arms for each field
    (@matcher $next_child:ident, $name:ident, $handled:ident, (@direct [$field:ident] [$value:expr])) => {};
    (@matcher $next_child:ident, $name:ident, $handled:ident, (@optional [$field:ident] [$namespaced_name:expr] [$var:ident] [$map:expr])) => {
        collect_struct!(@matcher $next_child, $name, $handled, (@field [$field] [$namespaced_name] [$var] [$map]));
    };
    (@matcher $next_child:ident, $name:ident, $handled:ident, (@field [$field:ident] [$namespaced_name:expr] [$var:ident] [$map:expr])) => {
        if !$handled &&
            &$name == $crate::io::IntoQualifiedNameCow::into_qname_cow($namespaced_name).as_ref()
        {
            let $var = &mut $next_child;
            $field = Some($map);
            $var.skip()?;
            $handled = true;
        }
    };

    (@build_struct $root:expr, $ty:ident, $($items:tt ; )* ) => {
        $ty {
            collect_struct!(@assign $root, $($items ; )*)
        }
    };

    // Emit struct field assignments
    (@assign $root:expr, $ty:ident, [$($out:tt)*], (@direct [$field:ident] [$value:expr]) ; $($tail:tt)*) => {
        collect_struct!(@assign $root, $ty, [
            $($out)*
            $field: $value,
        ], $($tail)*)
    };
    (@assign $root:expr, $ty:ident, [$($out:tt)*], (@optional [$field:ident] [$namespaced_name:expr] [$var:ident] [$map:expr]) ; $($tail:tt)*) => {
        collect_struct!(@assign $root, $ty, [
            $($out)*
            $field: $field,
        ], $($tail)*)
    };
    (@assign $root:expr, $ty:ident, [$($out:tt)*], (@field [$field:ident] [$namespaced_name:expr] [$var:ident] [$map:expr]) ; $($tail:tt)*) => {
        collect_struct!(@assign $root, $ty, [
            $($out)*
            $field: $crate::error::EMLResultExt::with_span(
                $field.ok_or_else(|| $crate::error::EMLErrorKind::MissingElement(
                    $crate::io::QualifiedName::from($namespaced_name).as_owned()
                )),
                $root.last_span()
            )?,
        ], $($tail)*)
    };
    (@assign $root:expr, $ty:ident, [$($out:tt)*], ) => {
        $ty {
            $($out)*
        }
    };
}
pub(crate) use collect_struct;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unknown_namespace() {
        let document = r#"<eml:UnknownElement />"#;
        let mut reader = EMLReader::init_from_str(document, true);
        let root = reader.next_element().unwrap();
        let error = root.name().unwrap_err();
        assert!(matches!(
            error.kind,
            EMLErrorKind::UnknownNamespace(ns) if ns == "eml"
        ));
    }
}
