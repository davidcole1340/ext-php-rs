use crate::convert::{FromZval, FromZvalMut};
use crate::ffi::{zend_object_iterator, ZEND_RESULT_CODE_SUCCESS};
use crate::flags::DataType;
use crate::prelude::PhpResult;
use crate::types::Zval;
use crate::zend::ExecutorGlobals;
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
    pub fn iter(&mut self) -> PhpResult<Iter> {
        self.index = 0;
        self.rewind()?;

        Ok(Iter { zi: self })
    }

    /// Check if the current position of the iterator is valid.
    ///
    /// As an example this will call the user defined valid method of the ['\Iterator'] interface.
    /// see <https://www.php.net/manual/en/iterator.valid.php>
    pub fn valid(&mut self) -> PhpResult<bool> {
        if let Some(valid) = unsafe { (*self.funcs).valid } {
            let valid = unsafe { valid(&mut *self) == ZEND_RESULT_CODE_SUCCESS };

            ExecutorGlobals::throw_if_exception()?;

            Ok(valid)
        } else {
            Ok(true)
        }
    }

    /// Rewind the iterator to the first element.
    ///
    /// As an example this will call the user defined rewind method of the ['\Iterator'] interface.
    /// see <https://www.php.net/manual/en/iterator.rewind.php>
    pub fn rewind(&mut self) -> PhpResult<()> {
        if let Some(rewind) = unsafe { (*self.funcs).rewind } {
            unsafe {
                rewind(&mut *self);
            }
        }

        ExecutorGlobals::throw_if_exception()
    }

    /// Move the iterator forward to the next element.
    ///
    /// As an example this will call the user defined next method of the ['\Iterator'] interface.
    /// see <https://www.php.net/manual/en/iterator.next.php>
    pub fn move_forward(&mut self) -> PhpResult<()> {
        if let Some(move_forward) = unsafe { (*self.funcs).move_forward } {
            unsafe {
                move_forward(&mut *self);
            }
        }

        ExecutorGlobals::throw_if_exception()
    }

    /// Get the current data of the iterator.
    ///
    /// # Returns
    ///
    /// Returns a reference to the current data of the iterator if available
    /// , ['None'] otherwise.
    pub fn get_current_data<'a>(&mut self) -> PhpResult<Option<&'a Zval>> {
        let get_current_data = match unsafe { (*self.funcs).get_current_data } {
            Some(get_current_data) => get_current_data,
            None => return Ok(None),
        };
        let value = unsafe { &*get_current_data(&mut *self) };

        ExecutorGlobals::throw_if_exception()?;

        Ok(Some(value))
    }

    /// Get the current key of the iterator.
    ///
    /// # Returns
    ///
    /// Returns a new ['Zval'] containing the current key of the iterator if available
    /// , ['None'] otherwise.
    pub fn get_current_key(&mut self) -> PhpResult<Option<Zval>> {
        let get_current_key = match unsafe { (*self.funcs).get_current_key } {
            Some(get_current_key) => get_current_key,
            None => return Ok(None),
        };

        let mut key = Zval::new();

        unsafe {
            get_current_key(&mut *self, &mut key);
        }

        ExecutorGlobals::throw_if_exception()?;

        Ok(Some(key))
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
    type Item = PhpResult<(IterKey, &'a Zval)>;

    fn next(&mut self) -> Option<Self::Item> {
        // Call next when index > 0, so next is really called at the start of each iteration, which allow to work better with generator iterator
        if self.zi.index > 0 {
            if let Err(err) = self.zi.move_forward() {
                return Some(Err(err));
            }
        }

        match self.zi.valid() {
            Err(err) => return Some(Err(err)),
            Ok(false) => return None,
            Ok(true) => (),
        }

        self.zi.index += 1;

        let real_index = self.zi.index - 1;

        let key = match self.zi.get_current_key() {
            Err(err) => return Some(Err(err)),
            Ok(None) => IterKey::Long(real_index),
            Ok(Some(key)) => match IterKey::from_zval(&key) {
                Some(key) => key,
                None => IterKey::Long(real_index),
            },
        };

        match self.zi.get_current_data() {
            Err(err) => Some(Err(err)),
            Ok(None) => None,
            Ok(Some(value)) => Some(Ok((key, value))),
        }
    }
}

impl<'a> FromZvalMut<'a> for &'a mut ZendIterator {
    const TYPE: DataType = DataType::Object(Some("Traversable"));

    fn from_zval_mut(zval: &'a mut Zval) -> Option<Self> {
        zval.object()?.get_class_entry().get_iterator(zval, false)
    }
}
