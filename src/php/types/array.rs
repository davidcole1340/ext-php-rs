use crate::bindings::{HashTable, _zend_new_array, HT_MIN_SIZE};

/// A PHP array, which internally is a hash table.
pub type ZendHashTable = HashTable;

impl ZendHashTable {
    /// Creates a new, empty, PHP associative array.
    pub fn new() -> *mut Self {
        Self::with_capacity(HT_MIN_SIZE)
    }

    /// Creates a new, empty, PHP associative array with an initial size.
    ///
    /// # Parameters
    ///
    /// * `size` - The size to initialize the array with.
    pub fn with_capacity(size: u32) -> *mut Self {
        // SAFETY: PHP allocater handles the creation of the
        // array.
        unsafe { _zend_new_array(size) }
    }

    /// Returns the current number of elements in the array.
    pub fn len() -> usize {
        todo!();
    }
}
