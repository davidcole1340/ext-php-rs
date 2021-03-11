use crate::{
    bindings::{
        HashTable, _Bucket, _zend_new_array, zend_hash_index_update, zend_hash_next_index_insert,
        zend_hash_str_update, HT_MIN_SIZE,
    },
    functions::c_str,
};

use super::zval::Zval;

/// A PHP array, which internally is a hash table.
pub type ZendHashTable = HashTable;

impl ZendHashTable {
    /// Creates a new, empty, PHP associative array.
    pub fn new<'a>() -> Option<&'a mut Self> {
        Self::with_capacity(HT_MIN_SIZE)
    }

    /// Creates a new, empty, PHP associative array with an initial size.
    ///
    /// # Parameters
    ///
    /// * `size` - The size to initialize the array with.
    pub fn with_capacity<'a>(size: u32) -> Option<&'a mut Self> {
        // SAFETY: PHP allocater handles the creation of the
        // array.
        unsafe { _zend_new_array(size).as_mut() }
    }

    /// Returns the current number of elements in the array.
    pub fn len(&self) -> usize {
        self.nNumOfElements as usize
    }

    /// Returns whether the hash table is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears the hash table, removing all values.
    pub fn clear(&mut self) {
        todo!();
    }

    /// Attempts to insert an item into the hash table, or update if the key already exists.
    ///
    /// # Parameters
    ///
    /// * `key` - The key to insert the value at in the hash table.
    /// * `value` - The value to insert into the hash table.
    ///
    /// # Returns
    ///
    /// * `Some(Zval)` - The existing value in the hash table that was overriden.
    /// * `None` - The element was inserted.
    pub fn insert<K, V>(&mut self, key: K, val: V) -> Option<&Zval>
    where
        K: Into<String>,
        V: Into<Zval>,
    {
        let key: String = key.into();
        let len = key.len();
        let val: Zval = val.into();

        let existing_ptr = unsafe {
            zend_hash_str_update(
                self as *mut Self,
                c_str(key),
                len as u64,
                Box::into_raw(Box::new(val)), // Do we really want to allocate the value on the heap?
                                              // I read somewhere that zvals are't usually (or never) allocated on the heap.
            )
        };

        // Should we be claiming this Zval into rust?
        // I'm not sure if the PHP GC will collect this.
        unsafe { existing_ptr.as_ref() }
    }

    /// Inserts an item into the hash table at a specified index,
    /// or updates if the key already exists.
    ///
    /// # Parameters
    ///
    /// * `key` - The index at which the value should be inserted.
    /// * `val` - The value to insert into the hash table.
    ///
    /// # Returns
    ///
    /// * `Some(Zval)` - The existing value in the hash table that was overriden.
    /// * `None` - The element was inserted.
    pub fn insert_at_index<V>(&mut self, key: u64, val: V) -> Option<&Zval>
    where
        V: Into<Zval>,
    {
        let val: Zval = val.into();

        let existing_ptr =
            unsafe { zend_hash_index_update(self as *mut Self, key, Box::into_raw(Box::new(val))) };

        // See `insert` function comment.
        unsafe { existing_ptr.as_ref() }
    }

    /// Pushes an item onto the end of the hash table.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to insert into the hash table.
    pub fn push<V>(&mut self, val: V)
    where
        V: Into<Zval>,
    {
        let val: Zval = val.into();

        unsafe { zend_hash_next_index_insert(self as *mut Self, Box::into_raw(Box::new(val))) };
    }
}

impl<'a> IntoIterator for &'a ZendHashTable {
    type Item = Zval;
    type IntoIter = ZendHashTableIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

/// Iterator for a Zend hashtable/array.
pub struct ZendHashTableIterator<'a> {
    ht: &'a ZendHashTable,
    pos: *mut _Bucket,
    end: *mut _Bucket,
}

impl<'a> ZendHashTableIterator<'a> {
    pub fn new(ht: &'a ZendHashTable) -> Self {
        let pos = ht.arData;
        let end = unsafe { ht.arData.offset(ht.nNumUsed as isize) };
        Self { ht, pos, end }
    }
}

impl<'a> Iterator for ZendHashTableIterator<'a> {
    type Item = Zval;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == self.end {
            return None;
        }

        let result = if let Some(val) = unsafe { self.pos.as_ref() } {
            Some(val.val)
        } else {
            None
        };

        self.pos = unsafe { self.pos.offset(1) };
        result
    }
}
