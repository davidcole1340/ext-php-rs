use crate::convert::{FromZval, FromZvalMut};
use crate::ffi::zend_object_iterator;
use crate::flags::DataType;
use crate::types::Zval;
use std::fmt::{Debug, Display, Formatter};

/// A PHP Iterator.
///
/// In PHP, iterators are represented as zend_object_iterator. This allow user to iterate
/// over object implementing Traversable interface using foreach.
pub type ZendIterator = zend_object_iterator;

impl ZendIterator {
    /// Creates a new rust iterator from a zend_object_iterator.
    ///
    /// # Returns
    ///
    /// Returns a iterator over the zend_object_iterator.
    pub fn iter(&mut self) -> Iter {
        self.index = 0;
        self.rewind();

        Iter { zi: self }
    }

    /// Check if the current position of the iterator is valid.
    ///
    /// As an example this will call the user defined valid method of the ['\Iterator'] interface.
    /// see <https://www.php.net/manual/en/iterator.valid.php>
    pub fn valid(&mut self) -> bool {
        if let Some(valid) = unsafe { (*self.funcs).valid } {
            unsafe { valid(&mut *self) != 0 }
        } else {
            true
        }
    }

    /// Rewind the iterator to the first element.
    ///
    /// As an example this will call the user defined rewind method of the ['\Iterator'] interface.
    /// see <https://www.php.net/manual/en/iterator.rewind.php>
    pub fn rewind(&mut self) {
        if let Some(rewind) = unsafe { (*self.funcs).rewind } {
            unsafe {
                rewind(&mut *self);
            }
        }
    }

    /// Move the iterator forward to the next element.
    ///
    /// As an example this will call the user defined next method of the ['\Iterator'] interface.
    /// see <https://www.php.net/manual/en/iterator.next.php>
    pub fn move_forward(&mut self) {
        if let Some(move_forward) = unsafe { (*self.funcs).move_forward } {
            unsafe {
                move_forward(&mut *self);
            }
        }
    }

    /// Get the current data of the iterator.
    ///
    /// # Returns
    ///
    /// Returns a reference to the current data of the iterator if available
    /// , ['None'] otherwise.
    pub fn get_current_data<'a>(&mut self) -> Option<&'a Zval> {
        let get_current_data = unsafe { (*self.funcs).get_current_data }?;
        let value = unsafe { &*get_current_data(&mut *self) };

        Some(value)
    }

    /// Get the current key of the iterator.
    ///
    /// # Returns
    ///
    /// Returns a new ['Zval'] containing the current key of the iterator if available
    /// , ['None'] otherwise.
    pub fn get_current_key(&mut self) -> Option<Zval> {
        let get_current_key = unsafe { (*self.funcs).get_current_key }?;
        let mut key = Zval::new();
        unsafe {
            get_current_key(&mut *self, &mut key);
        }

        Some(key)
    }
}

impl Debug for ZendIterator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZendIterator").finish()
    }
}

#[derive(Debug, PartialEq)]
pub enum IterKey {
    Long(u64),
    String(String),
}

/// Represent the key of a PHP iterator, which can be either a long or a string.
impl IterKey {
    /// Check if the key is numerical.
    ///
    /// # Returns
    ///
    /// Returns true if the key is numerical, false otherwise.
    pub fn is_numerical(&self) -> bool {
        match self {
            IterKey::Long(_) => true,
            IterKey::String(_) => false,
        }
    }
}

impl Display for IterKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IterKey::Long(key) => write!(f, "{}", key),
            IterKey::String(key) => write!(f, "{}", key),
        }
    }
}

impl FromZval<'_> for IterKey {
    const TYPE: DataType = DataType::String;

    fn from_zval(zval: &Zval) -> Option<Self> {
        match zval.long() {
            Some(key) => Some(IterKey::Long(key as u64)),
            None => zval.string().map(IterKey::String),
        }
    }
}

/// Immutable iterator upon a reference to a PHP iterator.
pub struct Iter<'a> {
    zi: &'a mut ZendIterator,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (IterKey, &'a Zval);

    fn next(&mut self) -> Option<Self::Item> {
        // Call next when index > 0, so next is really called at the start of each iteration, which allow to work better with generator iterator
        if self.zi.index > 0 {
            self.zi.move_forward();

            if !self.zi.valid() {
                return None;
            }
        }

        self.zi.index += 1;

        let key = self.zi.get_current_key();
        let value = self.zi.get_current_data()?;
        let real_index = self.zi.index - 1;

        Some(match key {
            Some(key) => match IterKey::from_zval(&key) {
                Some(key) => (key, value),
                None => (IterKey::Long(real_index), value),
            },
            None => (IterKey::Long(real_index), value),
        })
    }
}

impl<'a> FromZvalMut<'a> for &'a mut ZendIterator {
    const TYPE: DataType = DataType::Object(Some("Traversable"));

    fn from_zval_mut(zval: &'a mut Zval) -> Option<Self> {
        zval.object()?.get_class_entry().get_iterator(zval, false)
    }
}
