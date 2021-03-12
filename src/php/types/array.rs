use std::{collections::HashMap, u64};

use crate::{
    bindings::{
        HashTable, _Bucket, _zend_new_array, zend_array_destroy, zend_hash_clean,
        zend_hash_index_find, zend_hash_index_update, zend_hash_next_index_insert,
        zend_hash_str_find, zend_hash_str_update, HT_MIN_SIZE,
    },
    functions::c_str,
};

use super::zval::Zval;

/// A PHP array, which internally is a hash table.
pub struct ZendHashTable {
    ptr: *mut HashTable,
    free: bool,
}

impl ZendHashTable {
    /// Creates a new, empty, PHP associative array.
    pub fn new() -> Self {
        Self::with_capacity(HT_MIN_SIZE)
    }

    /// Creates a new, empty, PHP associative array with an initial size.
    ///
    /// # Parameters
    ///
    /// * `size` - The size to initialize the array with.
    pub fn with_capacity(size: u32) -> Self {
        // SAFETY: PHP allocater handles the creation of the
        // array.
        let ptr = unsafe { _zend_new_array(size) };
        Self { ptr, free: true }
    }

    /// Creates a new hash table wrapper.
    /// This _will not_ be freed when it goes out of scope in Rust.
    ///
    /// # Parameters
    ///
    /// * `ptr` - The pointer of the actual hash table.
    pub(crate) fn from_ptr(ptr: *mut HashTable) -> Self {
        Self { ptr, free: false }
    }

    /// Returns the current number of elements in the array.
    pub fn len(&self) -> usize {
        unsafe { *self.ptr }.nNumOfElements as usize
    }

    /// Returns whether the hash table is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears the hash table, removing all values.
    pub fn clear(&mut self) {
        unsafe { zend_hash_clean(self.ptr) }
    }

    /// Attempts to retrieve a value from the hash table with a string key.
    ///
    /// # Parameters
    ///
    /// * `key` - The key to search for in the hash table.
    ///
    /// # Returns
    ///
    /// `Some(&Zval)` - A reference to the zval at the position in the hash table.
    /// `None` - No value at the given position was found.
    pub fn get<K>(&self, key: K) -> Option<&Zval>
    where
        K: Into<String>,
    {
        let _key = key.into();
        let len = _key.len();
        unsafe { zend_hash_str_find(self.ptr, c_str(_key), len as u64).as_ref() }
    }

    /// Attempts to retrieve a value from the hash table with an index.
    ///
    /// # Parameters
    ///
    /// * `key` - The key to search for in the hash table.
    ///
    /// # Returns
    ///
    /// `Some(&Zval)` - A reference to the zval at the position in the hash table.
    /// `None` - No value at the given position was found.
    pub fn get_index(&self, key: u64) -> Option<&Zval> {
        unsafe { zend_hash_index_find(self.ptr, key).as_ref() }
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
                self.ptr,
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
            unsafe { zend_hash_index_update(self.ptr, key, Box::into_raw(Box::new(val))) };

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

        unsafe { zend_hash_next_index_insert(self.ptr, Box::into_raw(Box::new(val))) };
    }

    /// Converts the hash table into a raw pointer to be passed to Zend.
    pub(crate) fn into_ptr(mut self) -> *mut HashTable {
        self.free = false;
        self.ptr
    }
}

impl Default for ZendHashTable {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ZendHashTable {
    fn drop(&mut self) {
        if self.free {
            unsafe { zend_array_destroy(self.ptr) };
        }
    }
}

impl IntoIterator for ZendHashTable {
    type Item = (u64, Option<String>, Zval);
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

/// Iterator for a Zend hashtable/array.
pub struct Iter {
    ht: ZendHashTable,
    pos: *mut _Bucket,
    end: *mut _Bucket,
}

impl Iter {
    pub fn new(ht: ZendHashTable) -> Self {
        let ptr = unsafe { *ht.ptr };
        let pos = ptr.arData;
        let end = unsafe { ptr.arData.offset(ptr.nNumUsed as isize) };
        Self { ht, pos, end }
    }
}

impl Iterator for Iter {
    type Item = (u64, Option<String>, Zval);

    fn next(&mut self) -> Option<Self::Item> {
        // iterator complete
        if self.pos == self.end {
            return None;
        }

        let result = if let Some(val) = unsafe { self.pos.as_ref() } {
            // SAFETY: We can ensure safety further by checking if it is null before
            // converting it to a reference (val.key.as_ref() returns None if ptr == null)
            let str_key: Option<String> = unsafe { val.key.as_ref() }.map(|key| key.into());

            Some((val.h, str_key, val.val))
        } else {
            None
        };

        self.pos = unsafe { self.pos.offset(1) };
        result
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        unsafe { *self.ht.ptr }.nNumOfElements as usize
    }
}

/// Implementation converting a ZendHashTable into a Rust HashTable.
impl<'a> From<ZendHashTable> for HashMap<String, Zval> {
    fn from(zht: ZendHashTable) -> Self {
        let mut hm = HashMap::new();

        for (idx, key, val) in zht.into_iter() {
            hm.insert(key.unwrap_or_else(|| idx.to_string()), val);
        }

        hm
    }
}

/// Implementation converting a Rust HashTable into a ZendHashTable.
impl<'a, K, V> From<HashMap<K, V>> for ZendHashTable
where
    K: Into<String>,
    V: Into<Zval>,
{
    fn from(hm: HashMap<K, V>) -> Self {
        let mut ht = ZendHashTable::new();

        for (k, v) in hm {
            ht.insert(k.into(), v.into());
        }

        ht
    }
}

/// Implementation for converting a Rust Vec into a ZendHashTable.
impl<'a, V> From<Vec<V>> for ZendHashTable
where
    V: Into<Zval>,
{
    fn from(vec: Vec<V>) -> Self {
        let mut ht = ZendHashTable::new();

        for v in vec {
            ht.push(v);
        }

        ht
    }
}
