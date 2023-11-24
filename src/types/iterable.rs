use super::array::Iter as ZendHashTableIter;
use super::iterator::Iter as ZendIteratorIter;
use crate::convert::FromZval;
use crate::flags::DataType;
use crate::types::iterator::IterKey;
use crate::types::{ZendHashTable, ZendIterator, Zval};

/// This type represents a PHP iterable, which can be either an array or an object implementing
/// the Traversable interface.
#[derive(Debug)]
pub enum Iterable<'a> {
    Array(&'a ZendHashTable),
    Traversable(&'a mut ZendIterator),
}

impl<'a> Iterable<'a> {
    /// Creates a new rust iterator from a PHP iterable.
    pub fn iter(&mut self) -> Option<Iter> {
        match self {
            Iterable::Array(array) => Some(Iter::Array(array.iter())),
            Iterable::Traversable(traversable) => Some(Iter::Traversable(traversable.iter()?)),
        }
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
    type Item = (IterKey, &'a Zval);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Array(array) => array.next(),
            Iter::Traversable(traversable) => traversable.next(),
        }
    }
}
