use crate::convert::{FromZval, FromZvalMut};
use crate::ffi::{zend_object_iterator, ZEND_RESULT_CODE_SUCCESS};
use crate::flags::DataType;
use crate::types::Zval;
use crate::zend::ExecutorGlobals;
use std::fmt::{Debug, Display, Formatter};

/// A PHP Iterator.
///
/// In PHP, iterators are represented as zend_object_iterator. This allow user
/// to iterate over object implementing Traversable interface using foreach.
pub type ZendIterator = zend_object_iterator;

impl ZendIterator {
    /// Creates a new rust iterator from a zend_object_iterator.
    ///
    /// # Returns
    ///
    /// Returns a iterator over the zend_object_iterator.
    pub fn iter(&mut self) -> Option<Iter> {
        self.index = 0;

        if self.rewind() {
            return Some(Iter { zi: self });
        }

        None
    }

    /// Check if the current position of the iterator is valid.
    ///
    /// As an example this will call the user defined valid method of the
    /// ['\Iterator'] interface. see <https://www.php.net/manual/en/iterator.valid.php>
    pub fn valid(&mut self) -> bool {
        if let Some(valid) = unsafe { (*self.funcs).valid } {
            let valid = unsafe { valid(&mut *self) == ZEND_RESULT_CODE_SUCCESS };

            if ExecutorGlobals::has_exception() {
                return false;
            }

            valid
        } else {
            true
        }
    }

    /// Rewind the iterator to the first element.
    ///
    /// As an example this will call the user defined rewind method of the
    /// ['\Iterator'] interface. see <https://www.php.net/manual/en/iterator.rewind.php>
    ///
    /// # Returns
    ///
    /// Returns true if the iterator was successfully rewind, false otherwise.
    /// (when there is an exception during rewind)
    pub fn rewind(&mut self) -> bool {
        if let Some(rewind) = unsafe { (*self.funcs).rewind } {
            unsafe {
                rewind(&mut *self);
            }
        }

        !ExecutorGlobals::has_exception()
    }

    /// Move the iterator forward to the next element.
    ///
    /// As an example this will call the user defined next method of the
    /// ['\Iterator'] interface. see <https://www.php.net/manual/en/iterator.next.php>
    ///
    /// # Returns
    ///
    /// Returns true if the iterator was successfully move, false otherwise.
    /// (when there is an exception during next)
    pub fn move_forward(&mut self) -> bool {
        if let Some(move_forward) = unsafe { (*self.funcs).move_forward } {
            unsafe {
                move_forward(&mut *self);
            }
        }

        !ExecutorGlobals::has_exception()
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

        if ExecutorGlobals::has_exception() {
            return None;
        }

        Some(value)
    }

    /// Get the current key of the iterator.
    ///
    /// # Returns
    ///
    /// Returns a new ['Zval'] containing the current key of the iterator if
    /// available , ['None'] otherwise.
    pub fn get_current_key(&mut self) -> Option<Zval> {
        let get_current_key = unsafe { (*self.funcs).get_current_key? };
        let mut key = Zval::new();

        unsafe {
            get_current_key(&mut *self, &mut key);
        }

        if ExecutorGlobals::has_exception() {
            return None;
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
        // Call next when index > 0, so next is really called at the start of each
        // iteration, which allow to work better with generator iterator
        if self.zi.index > 0 && !self.zi.move_forward() {
            return None;
        }

        if !self.zi.valid() {
            return None;
        }

        self.zi.index += 1;

        let real_index = self.zi.index - 1;

        let key = match self.zi.get_current_key() {
            None => IterKey::Long(real_index),
            Some(key) => match IterKey::from_zval(&key) {
                Some(key) => key,
                None => IterKey::Long(real_index),
            },
        };

        self.zi.get_current_data().map(|value| (key, value))
    }
}

impl<'a> FromZvalMut<'a> for &'a mut ZendIterator {
    const TYPE: DataType = DataType::Object(Some("Traversable"));

    fn from_zval_mut(zval: &'a mut Zval) -> Option<Self> {
        zval.object()?.get_class_entry().get_iterator(zval, false)
    }
}

#[cfg(test)]
#[cfg(feature = "embed")]
mod tests {
    use crate::embed::Embed;
    use crate::types::iterator::IterKey;

    #[test]
    fn test_generator() {
        Embed::run(|| {
            let result = Embed::run_script("src/types/iterator.test.php");

            assert!(result.is_ok());

            let generator = Embed::eval("$generator;");

            assert!(generator.is_ok());

            let zval = generator.unwrap();

            assert!(zval.is_traversable());

            let iterator = zval.traversable().unwrap();

            assert!(iterator.valid());

            {
                let mut iter = iterator.iter().unwrap();

                let (key, value) = iter.next().unwrap();

                assert_eq!(key, IterKey::Long(0));
                assert!(value.is_long());
                assert_eq!(value.long().unwrap(), 1);

                let (key, value) = iter.next().unwrap();

                assert_eq!(key, IterKey::Long(1));
                assert!(value.is_long());
                assert_eq!(value.long().unwrap(), 2);

                let (key, value) = iter.next().unwrap();

                assert_eq!(key, IterKey::Long(2));
                assert!(value.is_long());
                assert_eq!(value.long().unwrap(), 3);

                let next = iter.next();

                assert!(next.is_none());
            }
        });
    }

    #[test]
    fn test_iterator() {
        Embed::run(|| {
            let result = Embed::run_script("src/types/iterator.test.php");

            assert!(result.is_ok());

            let generator = Embed::eval("$iterator;");

            assert!(generator.is_ok());

            let zval = generator.unwrap();

            assert!(zval.is_traversable());

            let iterator = zval.traversable().unwrap();

            assert!(iterator.valid());

            {
                let mut iter = iterator.iter().unwrap();

                let (key, value) = iter.next().unwrap();

                assert!(!key.is_numerical());
                assert_eq!(key, IterKey::String("key".to_string()));
                assert!(value.is_string());
                assert_eq!(value.string().unwrap(), "foo");

                let (key, value) = iter.next().unwrap();

                assert!(key.is_numerical());
                assert_eq!(key, IterKey::Long(10));
                assert!(value.is_string());
                assert_eq!(value.string().unwrap(), "bar");

                let (key, value) = iter.next().unwrap();

                assert_eq!(key, IterKey::Long(2));
                assert!(value.is_string());
                assert_eq!(value.string().unwrap(), "baz");

                let next = iter.next();

                assert!(next.is_none());
            }

            // Test rewind
            {
                let mut iter = iterator.iter().unwrap();

                let (key, value) = iter.next().unwrap();

                assert_eq!(key, IterKey::String("key".to_string()));
                assert!(value.is_string());
                assert_eq!(value.string().unwrap(), "foo");

                let (key, value) = iter.next().unwrap();

                assert_eq!(key, IterKey::Long(10));
                assert!(value.is_string());
                assert_eq!(value.string().unwrap(), "bar");

                let (key, value) = iter.next().unwrap();

                assert_eq!(key, IterKey::Long(2));
                assert!(value.is_string());
                assert_eq!(value.string().unwrap(), "baz");

                let next = iter.next();

                assert!(next.is_none());
            }
        });
    }
}
