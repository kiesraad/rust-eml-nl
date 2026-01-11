use std::{borrow::Cow, fmt::Display, ops::Deref};

pub(crate) trait IntoQualifiedNameCow<'a, 'b, 'c> {
    fn into_qname_cow(self) -> Cow<'a, QualifiedName<'b, 'c>>;
}

impl<'a, 'b, 'c> IntoQualifiedNameCow<'a, 'b, 'c> for &'a QualifiedName<'b, 'c> {
    fn into_qname_cow(self) -> Cow<'a, QualifiedName<'b, 'c>> {
        Cow::Borrowed(self)
    }
}

impl<'a, 'b, 'c, T> IntoQualifiedNameCow<'a, 'b, 'c> for T
where
    QualifiedName<'b, 'c>: From<T>,
{
    fn into_qname_cow(self) -> Cow<'a, QualifiedName<'b, 'c>> {
        Cow::Owned(QualifiedName::from(self))
    }
}

/// A qualified XML name, consisting of a local name and an optional namespace URI.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct QualifiedName<'a, 'b> {
    /// Local name of the qualified name.
    pub local_name: Cow<'a, str>,
    /// Optional namespace URI of the qualified name.
    pub namespace: Option<Cow<'b, str>>,
}

impl<'a, 'b> From<(&'a str, Option<&'b str>)> for QualifiedName<'a, 'b> {
    fn from((local_name, namespace): (&'a str, Option<&'b str>)) -> Self {
        QualifiedName::new(local_name, namespace)
    }
}

impl<'a, 'b> From<(&'a str, &'b str)> for QualifiedName<'a, 'b> {
    fn from((local_name, namespace): (&'a str, &'b str)) -> Self {
        QualifiedName::new(local_name, Some(namespace))
    }
}

impl<'a> From<(&'a str,)> for QualifiedName<'a, 'a> {
    fn from((local_name,): (&'a str,)) -> Self {
        QualifiedName::new(local_name, None::<&str>)
    }
}

impl<'a> From<&'a str> for QualifiedName<'a, 'a> {
    fn from(local_name: &'a str) -> Self {
        QualifiedName::new(local_name, None::<&str>)
    }
}

impl<'a, 'b> QualifiedName<'a, 'b> {
    /// Create a new qualified name with the given local name and namespace URI.
    pub fn new(
        local_name: impl Into<Cow<'a, str>>,
        namespace: Option<impl Into<Cow<'b, str>>>,
    ) -> Self {
        QualifiedName {
            local_name: local_name.into(),
            namespace: namespace.map(|ns| ns.into()),
        }
    }

    /// Create a new qualified name from static string slices, usable in const contexts.
    pub const fn from_static(local_name: &'static str, namespace: Option<&'static str>) -> Self {
        if let Some(namespace) = namespace {
            QualifiedName {
                local_name: Cow::Borrowed(local_name),
                namespace: Some(Cow::Borrowed(namespace)),
            }
        } else {
            QualifiedName {
                local_name: Cow::Borrowed(local_name),
                namespace: None,
            }
        }
    }

    /// Convert this (possibly borrowed) qualified name into one that fully owns
    /// its data.
    pub fn as_owned(&self) -> OwnedQualifiedName {
        OwnedQualifiedName::new(self.local_name.as_ref(), self.namespace.as_deref())
    }
}

impl<'a, 'b> Display for QualifiedName<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ns) = &self.namespace {
            write!(f, "{{{}}}{}", ns, self.local_name)
        } else {
            write!(f, "{}", self.local_name)
        }
    }
}

/// A fully owned qualified name (consisting of local name and optional namespace URI).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct OwnedQualifiedName(QualifiedName<'static, 'static>);

impl OwnedQualifiedName {
    /// Create a new owned qualified name with the given local name and namespace URI.
    pub fn new(local_name: impl Into<String>, namespace: Option<impl Into<String>>) -> Self {
        OwnedQualifiedName(QualifiedName::new(
            Cow::Owned(local_name.into()),
            namespace.map(|ns| Cow::Owned(ns.into())),
        ))
    }

    /// Consume this owned qualified name and return the inner qualified name.
    ///
    /// The inner qualified name will own its data.
    pub fn into_inner(self) -> QualifiedName<'static, 'static> {
        self.0
    }
}

impl Display for OwnedQualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for OwnedQualifiedName {
    type Target = QualifiedName<'static, 'static>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qualified_name_from() {
        let qname1: QualifiedName = ("LocalName", Some("http://example.com/ns")).into();
        assert_eq!(qname1.local_name, "LocalName");
        assert_eq!(qname1.namespace.as_deref(), Some("http://example.com/ns"));

        let qname2: QualifiedName = ("LocalName", None).into();
        assert_eq!(qname2.local_name, "LocalName");
        assert_eq!(qname2.namespace, None);

        let qname3: QualifiedName = ("LocalName", "http://example.com/ns").into();
        assert_eq!(qname3.local_name, "LocalName");
        assert_eq!(qname3.namespace.as_deref(), Some("http://example.com/ns"));

        let qname4: QualifiedName = ("LocalName",).into();
        assert_eq!(qname4.local_name, "LocalName");
        assert_eq!(qname4.namespace, None);
    }

    #[test]
    fn test_qualified_name_display() {
        let qname1: QualifiedName = ("LocalName", Some("http://example.com/ns")).into();
        assert_eq!(qname1.to_string(), "{http://example.com/ns}LocalName");

        let qname2: QualifiedName = ("LocalName", None).into();
        assert_eq!(qname2.to_string(), "LocalName");

        let owned_name = OwnedQualifiedName::new("LocalName", Some("http://example.com/ns"));
        assert_eq!(owned_name.to_string(), "{http://example.com/ns}LocalName");
    }

    #[test]
    fn test_owned_qualified_name() {
        let owned_name = OwnedQualifiedName::new("LocalName", Some("http://example.com/ns"));
        let qname: QualifiedName = owned_name.into_inner();
        assert_eq!(qname.local_name, "LocalName");
        assert_eq!(qname.namespace.as_deref(), Some("http://example.com/ns"));

        let owned_from_borrowed =
            QualifiedName::from(("LocalName", "http://example.com/ns")).as_owned();
        assert!(matches!(owned_from_borrowed.local_name, Cow::Owned(_)));
        assert!(matches!(owned_from_borrowed.namespace, Some(Cow::Owned(_))));
    }
}
