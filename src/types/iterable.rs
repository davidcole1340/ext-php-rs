use super::array::Iter as ZendHashTableIter;
use super::iterator::Iter as ZendIteratorIter;
use crate::convert::FromZval;
use crate::flags::DataType;
use crate::types::{ZendHashTable, ZendIterator, Zval};

/// This type represents a PHP iterable, which can be either an array or an
/// object implementing the Traversable interface.
#[derive(Debug)]
pub enum Iterable<'a> {
    /// Iterable is an Array
    Array(&'a ZendHashTable),
    /// Iterable is a Traversable
    Traversable(&'a mut ZendIterator),
}

impl Iterable<'_> {
    /// Creates a new rust iterator from a PHP iterable.
    /// May return None if a Traversable cannot be rewound.
    // TODO: Check iter not returning iterator
    #[allow(clippy::iter_not_returning_iterator)]
    pub fn iter(&mut self) -> Option<Iter<'_>> {
        match self {
            Iterable::Array(array) => Some(Iter::Array(array.iter())),
            Iterable::Traversable(traversable) => Some(Iter::Traversable(traversable.iter()?)),
        }
    }
}

// TODO: Implement `iter_mut`
#[allow(clippy::into_iter_without_iter)]
impl<'a> IntoIterator for &'a mut Iterable<'a> {
    type Item = (Zval, &'a Zval);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter().expect("Could not rewind iterator!")
    }
}

impl<'a> FromZval<'a> for Iterable<'a> {
    const TYPE: DataType = DataType::Iterable;

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        if let Some(array) = zval.array() {
            return Some(Iterable::Array(array));
        }

        if let Some(traversable) = zval.traversable() {
            return Some(Iterable::Traversable(traversable));
        }

        None
    }
}

/// Rust iterator over a PHP iterable.
pub enum Iter<'a> {
    Array(ZendHashTableIter<'a>),
    Traversable(ZendIteratorIter<'a>),
}

impl<'a> Iterator for Iter<'a> {
    type Item = (Zval, &'a Zval);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Array(array) => array.next_zval(),
            Iter::Traversable(traversable) => traversable.next(),
        }
    }
}
