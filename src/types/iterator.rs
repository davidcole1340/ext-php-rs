use crate::convert::FromZvalMut;
use crate::ffi::{zend_object_iterator, ZEND_RESULT_CODE_SUCCESS};
use crate::flags::DataType;
use crate::types::Zval;
use crate::zend::ExecutorGlobals;
use std::fmt::{Debug, Formatter};

/// A PHP Iterator.
///
/// In PHP, iterators are represented as zend_object_iterator. This allows user
/// to iterate over objects implementing Traversable interface using foreach.
///
/// Use ZendIterable to iterate over both iterators and arrays.
pub type ZendIterator = zend_object_iterator;

impl ZendIterator {
    /// Creates a new rust iterator from a zend_object_iterator.
    ///
    /// Returns a iterator over the zend_object_iterator, or None if the
    /// iterator cannot be rewound.
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

impl<'a> IntoIterator for &'a mut ZendIterator {
    type Item = (Zval, &'a Zval);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter().expect("Could not rewind iterator!")
    }
}

impl Debug for ZendIterator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZendIterator").finish()
    }
}

/// Immutable iterator upon a reference to a PHP iterator.
pub struct Iter<'a> {
    zi: &'a mut ZendIterator,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (Zval, &'a Zval);

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
            None => {
                let mut z = Zval::new();
                z.set_long(real_index as i64);
                z
            }
            Some(key) => key,
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
#[cfg(all(feature = "embed", any(php81, not(php_zts))))]
mod tests {
    use crate::embed::Embed;

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

                assert_eq!(key.long(), Some(0));
                assert!(value.is_long());
                assert_eq!(value.long().unwrap(), 1);

                let (key, value) = iter.next().unwrap();

                assert_eq!(key.long(), Some(1));
                assert!(value.is_long());
                assert_eq!(value.long().unwrap(), 2);

                let (key, value) = iter.next().unwrap();

                assert_eq!(key.long(), Some(2));
                assert!(value.is_long());
                assert_eq!(value.long().unwrap(), 3);

                let (key, value) = iter.next().unwrap();

                assert!(key.is_object());
                assert!(value.is_object());

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

                assert!(!key.is_long());
                assert_eq!(key.str(), Some("key"));
                assert!(value.is_string());
                assert_eq!(value.str(), Some("foo"));

                let (key, value) = iter.next().unwrap();

                assert!(key.is_long());
                assert_eq!(key.long(), Some(10));
                assert!(value.is_string());
                assert_eq!(value.string().unwrap(), "bar");

                let (key, value) = iter.next().unwrap();

                assert_eq!(key.long(), Some(2));
                assert!(value.is_string());
                assert_eq!(value.string().unwrap(), "baz");

                let (key, value) = iter.next().unwrap();

                assert!(key.is_object());
                assert!(value.is_object());

                let next = iter.next();

                assert!(next.is_none());
            }

            // Test rewind
            {
                let mut iter = iterator.iter().unwrap();

                let (key, value) = iter.next().unwrap();

                assert_eq!(key.str(), Some("key"));
                assert!(value.is_string());
                assert_eq!(value.string().unwrap(), "foo");

                let (key, value) = iter.next().unwrap();

                assert_eq!(key.long(), Some(10));
                assert!(value.is_string());
                assert_eq!(value.string().unwrap(), "bar");

                let (key, value) = iter.next().unwrap();

                assert_eq!(key.long(), Some(2));
                assert!(value.is_string());
                assert_eq!(value.string().unwrap(), "baz");

                let (key, value) = iter.next().unwrap();

                assert!(key.is_object());
                assert!(value.is_object());

                let next = iter.next();

                assert!(next.is_none());
            }
        });
    }
}
