use ::divvy::Str;
use ::kserd::*;
use std::{borrow::Cow, cmp::Ordering};
use Entry::*;

/// A table cell.
///
/// An entry is mostly of arbitary type, distinguishing between empty (`Nil`) entries, numeric
/// entries (`Num`), and object entries `T`.
///
/// An entry exhibits total equality if `T: Eq`, and has partial ordering. Ordering of the same
/// variants will work, but if the variants are different, no ordering occurs.
///
/// ```rust
/// # use table::*;
/// use std::cmp::Ordering;
/// let lhs: Entry<&str> = Entry::Nil;
/// let rhs = Entry::Obj("a");
/// // no ordering between Nil and Obj types
/// assert_eq!(lhs.partial_cmp(&rhs), None);
///
/// let lhs = Entry::Obj("b");
/// // ordering between same variants
/// assert_eq!(lhs.partial_cmp(&rhs), Some(Ordering::Greater));
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Entry<T> {
    /// An empty entry.
    Nil,
    /// A numeric entry. [`Number`] is used to acheive [`Ord`] and [`Eq`] across integers and
    /// floats.
    Num(Number),
    /// An arbitary object type.
    Obj(T),
}

impl<T> Default for Entry<T> {
    fn default() -> Self {
        Entry::Nil
    }
}

impl<T> Entry<T> {
    /// Entry is [`Entry::Nil`] variant.
    pub fn is_nil(&self) -> bool {
        matches!(self, Nil)
    }

    /// Entry is [`Entry::Num`] variant.
    pub fn is_num(&self) -> bool {
        matches!(self, Num(_))
    }

    /// Entry is [`Entry::Obj`] variant.
    pub fn is_obj(&self) -> bool {
        matches!(self, Obj(_))
    }

    /// Retreives value if Entry is [`Entry::Num`] variant.
    pub fn num(&self) -> Option<Number> {
        match self {
            Entry::Num(n) => Some(*n),
            _ => None,
        }
    }

    /// Retreives value if Entry is [`Entry::Obj`] variant.
    pub fn obj(&self) -> Option<&T> {
        match self {
            Entry::Obj(t) => Some(t),
            _ => None,
        }
    }

    /// Represent this entry as a _borrowed_ string.
    ///
    /// If `T` implements [`AsRef`]`<str>` then `Entry` can be represented as a string, without
    /// allocation. If `Entry` is numeric, allocation is required and `Cow::Owned` is returned.
    ///
    /// ```rust
    /// # use ::table::Entry;
    /// assert_eq!(Entry::<String>::Nil.as_str(), "-");
    /// assert_eq!(Entry::<String>::Num(3.14.into()).as_str(), "3.14");
    /// assert_eq!(Entry::<String>::Obj("what".into()).as_str(), "what");
    /// ```
    pub fn as_str(&self) -> Cow<'_, str>
    where
        T: AsRef<str>,
    {
        match self {
            Nil => Cow::Borrowed("-"),
            Num(n) => Cow::Owned(n.to_string()),
            Obj(o) => Cow::Borrowed(o.as_ref()),
        }
    }
}

impl<T: Copy> From<&Entry<T>> for Entry<T> {
    fn from(e: &Entry<T>) -> Self {
        match e {
            Nil => Nil,
            Num(x) => Num(*x),
            Obj(x) => Obj(*x),
        }
    }
}

impl<'a> From<&Kserd<'a>> for Entry<Str> {
    fn from(kserd: &Kserd<'a>) -> Self {
        match &kserd.val {
            Value::Unit => Nil,
            Value::Num(n) => Num(*n),
            Value::Str(s) => Obj(Str::new(s.as_str())),
            _ => Obj(Str::new(kserd.as_str())),
        }
    }
}

impl<T: AsRef<str>> PartialEq<str> for Entry<T> {
    fn eq(&self, rhs: &str) -> bool {
        match self {
            Obj(lhs) => lhs.as_ref() == rhs,
            _ => false,
        }
    }
}

impl<T: PartialOrd> PartialOrd for Entry<T> {
    fn partial_cmp(&self, rhs: &Entry<T>) -> Option<Ordering> {
        match (self, rhs) {
            (Nil, Nil) => Some(Ordering::Equal),
            (Num(lhs), Num(rhs)) => lhs.partial_cmp(rhs),
            (Obj(lhs), Obj(rhs)) => lhs.partial_cmp(rhs),
            _ => None,
        }
    }
}
