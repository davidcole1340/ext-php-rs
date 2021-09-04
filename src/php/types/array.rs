//! Represents an array in PHP. As all arrays in PHP are associative arrays, they are represented
//! by hash tables.

use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    ffi::CString,
    fmt::Debug,
    marker::PhantomData,
    u64,
};

use crate::{
    bindings::{
        HashTable, _Bucket, _zend_new_array, zend_array_destroy, zend_array_dup, zend_hash_clean,
        zend_hash_index_del, zend_hash_index_find, zend_hash_index_update,
        zend_hash_next_index_insert, zend_hash_str_del, zend_hash_str_find, zend_hash_str_update,
        HT_MIN_SIZE,
    },
    errors::{Error, Result},
    php::enums::DataType,
};

use super::{
    string::ZendString,
    zval::{IntoZval, Zval},
};

/// Result type returned after attempting to insert an element into a hash table.
#[derive(Debug)]
pub enum HashTableInsertResult<'a> {
    /// The element was inserted into the hash table successfully.
    Ok,
    /// The element was inserted into the hash table successfully, over-writing an existing element.
    OkWithOverwrite(&'a Zval),
}

/// A PHP array, which internally is a hash table.
pub struct ZendHashTable<'a> {
    ptr: *mut HashTable,
    free: bool,
    phantom: PhantomData<&'a HashTable>,
}

impl<'a> ZendHashTable<'a> {
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
        Self {
            ptr,
            free: true,
            phantom: PhantomData,
        }
    }

    /// Creates a new hash table wrapper.
    /// This _will not_ be freed when it goes out of scope in Rust.
    ///
    /// # Parameters
    ///
    /// * `ptr` - The pointer of the actual hash table.
    /// * `free` - Whether the pointer should be freed when the resulting [`ZendHashTable`]
    /// goes out of scope.
    ///
    /// # Safety
    ///
    /// As a raw pointer is given this function is unsafe, you must ensure that the pointer is valid when calling
    /// the function. A simple null check is done but this is not sufficient in most cases.
    pub unsafe fn from_ptr(ptr: *mut HashTable, free: bool) -> Result<Self> {
        if ptr.is_null() {
            return Err(Error::InvalidPointer);
        }

        Ok(Self {
            ptr,
            free,
            phantom: PhantomData,
        })
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
    /// * `Some(&Zval)` - A reference to the zval at the position in the hash table.
    /// * `None` - No value at the given position was found.
    pub fn get(&self, key: &str) -> Option<&Zval> {
        let str = CString::new(key).ok()?;
        unsafe { zend_hash_str_find(self.ptr, str.as_ptr(), key.len() as _).as_ref() }
    }

    /// Attempts to retrieve a value from the hash table with an index.
    ///
    /// # Parameters
    ///
    /// * `key` - The key to search for in the hash table.
    ///
    /// # Returns
    ///
    /// * `Some(&Zval)` - A reference to the zval at the position in the hash table.
    /// * `None` - No value at the given position was found.
    pub fn get_index(&self, key: u64) -> Option<&Zval> {
        unsafe { zend_hash_index_find(self.ptr, key).as_ref() }
    }

    /// Attempts to remove a value from the hash table with a string key.
    ///
    /// # Parameters
    ///
    /// * `key` - The key to remove from the hash table.
    ///
    /// # Returns
    ///
    /// * `Some(())` - Key was successfully removed.
    /// * `None` - No key was removed, did not exist.
    pub fn remove<K>(&self, key: &str) -> Option<()> {
        let result = unsafe {
            zend_hash_str_del(self.ptr, CString::new(key).ok()?.as_ptr(), key.len() as _)
        };

        if result < 0 {
            None
        } else {
            Some(())
        }
    }

    /// Attempts to remove a value from the hash table with a string key.
    ///
    /// # Parameters
    ///
    /// * `key` - The key to remove from the hash table.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Key was successfully removed.
    /// * `None` - No key was removed, did not exist.
    pub fn remove_index(&self, key: u64) -> Option<()> {
        let result = unsafe { zend_hash_index_del(self.ptr, key) };

        if result < 0 {
            None
        } else {
            Some(())
        }
    }

    /// Attempts to insert an item into the hash table, or update if the key already exists.
    /// Returns a result containing a [`HashTableInsertResult`], which will indicate a successful
    /// insert, with the insert result variants either containing the overwritten value or nothing.
    ///
    /// # Parameters
    ///
    /// * `key` - The key to insert the value at in the hash table.
    /// * `value` - The value to insert into the hash table.
    pub fn insert<V>(&mut self, key: &str, val: V) -> Result<HashTableInsertResult>
    where
        V: IntoZval,
    {
        let mut val = val.as_zval(false)?;
        let existing_ptr = unsafe {
            zend_hash_str_update(
                self.ptr,
                CString::new(key)?.as_ptr(),
                key.len() as u64,
                &mut val,
            )
        };
        val.release();

        // Should we be claiming this Zval into rust?
        // I'm not sure if the PHP GC will collect this.

        // SAFETY: The `zend_hash_str_update` function will either return a valid pointer or a null pointer.
        // In the latter case, `as_ref()` will return `None`.
        Ok(match unsafe { existing_ptr.as_ref() } {
            Some(ptr) => HashTableInsertResult::OkWithOverwrite(ptr),
            None => HashTableInsertResult::Ok,
        })
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
    /// * `Some(&Zval)` - The existing value in the hash table that was overriden.
    /// * `None` - The element was inserted.
    pub fn insert_at_index<V>(&mut self, key: u64, val: V) -> Result<HashTableInsertResult>
    where
        V: IntoZval,
    {
        let mut val = val.as_zval(false)?;
        let existing_ptr = unsafe { zend_hash_index_update(self.ptr, key, &mut val) };
        val.release();

        // SAFETY: The `zend_hash_str_update` function will either return a valid pointer or a null pointer.
        // In the latter case, `as_ref()` will return `None`.
        Ok(match unsafe { existing_ptr.as_ref() } {
            Some(ptr) => HashTableInsertResult::OkWithOverwrite(ptr),
            None => HashTableInsertResult::Ok,
        })
    }

    /// Pushes an item onto the end of the hash table. Returns a result containing nothing if the
    /// element was sucessfully inserted.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to insert into the hash table.
    pub fn push<V>(&mut self, val: V) -> Result<()>
    where
        V: IntoZval,
    {
        let mut val = val.as_zval(false)?;
        unsafe { zend_hash_next_index_insert(self.ptr, &mut val) };
        val.release();

        Ok(())
    }

    /// Returns an iterator over the hash table.
    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        Iter::new(self)
    }

    /// Converts the hash table into a raw pointer to be passed to Zend.
    pub(crate) fn into_ptr(mut self) -> *mut HashTable {
        self.free = false;
        self.ptr
    }
}

impl<'a> Debug for ZendHashTable<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(
                self.iter()
                    .map(|(k, k2, v)| (k2.unwrap_or_else(|| k.to_string()), v)),
            )
            .finish()
    }
}

impl<'a> Default for ZendHashTable<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Drop for ZendHashTable<'a> {
    fn drop(&mut self) {
        if self.free {
            unsafe { zend_array_destroy(self.ptr) };
        }
    }
}

impl<'a> Clone for ZendHashTable<'a> {
    fn clone(&self) -> Self {
        // SAFETY: If this fails then `emalloc` failed - we are doomed anyway?
        // `from_ptr()` checks if the ptr is null.
        unsafe {
            let ptr = zend_array_dup(self.ptr);
            Self::from_ptr(ptr, true).expect("ZendHashTable cloning failed when duplicating array.")
        }
    }
}

impl<'a> IntoIterator for ZendHashTable<'a> {
    type Item = (u64, Option<String>, &'a Zval);
    type IntoIter = IntoIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

macro_rules! build_iter {
    ($name: ident, $ht: ty) => {
        pub struct $name<'a> {
            ht: $ht,
            pos: *mut _Bucket,
            end: *mut _Bucket,
        }

        impl<'a> $name<'a> {
            pub fn new(ht: $ht) -> Self {
                let ptr = unsafe { *ht.ptr };
                let pos = ptr.arData;
                let end = unsafe { ptr.arData.offset(ptr.nNumUsed as isize) };
                Self { ht, pos, end }
            }
        }

        impl<'a> Iterator for $name<'a> {
            type Item = (u64, Option<String>, &'a Zval);

            fn next(&mut self) -> Option<Self::Item> {
                // iterator complete
                if self.pos == self.end {
                    return None;
                }

                let result = if let Some(val) = unsafe { self.pos.as_ref() } {
                    // SAFETY: We can ensure safety further by checking if it is null before
                    // converting it to a reference (val.key.as_ref() returns None if ptr == null)
                    let str_key = unsafe { ZendString::from_ptr(val.key, false) }
                        .and_then(|s| s.try_into())
                        .ok();

                    Some((val.h, str_key, &val.val))
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
    };
}

build_iter!(Iter, &'a ZendHashTable<'a>);
build_iter!(IntoIter, ZendHashTable<'a>);

impl<'a, V> TryFrom<ZendHashTable<'a>> for HashMap<String, V>
where
    V: TryFrom<&'a Zval>,
{
    type Error = Error;

    fn try_from(zht: ZendHashTable<'a>) -> Result<Self> {
        let mut hm = HashMap::with_capacity(zht.len());

        for (idx, key, val) in zht.into_iter() {
            hm.insert(
                key.unwrap_or_else(|| idx.to_string()),
                val.try_into()
                    .map_err(|_| Error::ZvalConversion(DataType::Null))?,
            );
        }

        Ok(hm)
    }
}

impl<'a, K, V> TryFrom<HashMap<K, V>> for ZendHashTable<'a>
where
    K: AsRef<str>,
    V: IntoZval,
{
    type Error = Error;

    fn try_from(hm: HashMap<K, V>) -> Result<Self> {
        let mut ht =
            ZendHashTable::with_capacity(hm.len().try_into().map_err(|_| Error::IntegerOverflow)?);

        for (k, v) in hm.into_iter() {
            ht.insert(k.as_ref(), v)?;
        }

        Ok(ht)
    }
}

/// Implementation converting a Rust HashTable into a ZendHashTable.
impl<'a, 'b, K, V> TryFrom<&'a HashMap<K, V>> for ZendHashTable<'b>
where
    K: AsRef<str>,
    V: IntoZval + Clone,
{
    type Error = Error;

    fn try_from(hm: &'a HashMap<K, V>) -> Result<Self> {
        let mut ht =
            ZendHashTable::with_capacity(hm.len().try_into().map_err(|_| Error::IntegerOverflow)?);

        for (k, v) in hm.iter() {
            ht.insert(k.as_ref(), v.clone())?;
        }

        Ok(ht)
    }
}

/// Implementation for converting a reference to `ZendHashTable` into a `Vec` of given type.
/// Will return an error type if one of the values inside the array cannot be converted into
/// a type `T`.
impl<'a, V> TryFrom<ZendHashTable<'a>> for Vec<V>
where
    V: TryFrom<&'a Zval>,
{
    type Error = Error;

    fn try_from(ht: ZendHashTable<'a>) -> Result<Self> {
        ht.into_iter()
            .map(|(_, _, v)| {
                v.try_into()
                    .map_err(|_| Error::ZvalConversion(DataType::Null))
            })
            .collect::<Result<Vec<_>>>()
    }
}

impl<'a, V> TryFrom<Vec<V>> for ZendHashTable<'a>
where
    V: IntoZval,
{
    type Error = Error;

    fn try_from(vec: Vec<V>) -> Result<Self> {
        let mut ht =
            ZendHashTable::with_capacity(vec.len().try_into().map_err(|_| Error::IntegerOverflow)?);

        for val in vec.into_iter() {
            ht.push(val)?;
        }

        Ok(ht)
    }
}

impl<'a, V> TryFrom<&Vec<V>> for ZendHashTable<'a>
where
    V: IntoZval + Clone,
{
    type Error = Error;

    fn try_from(vec: &Vec<V>) -> Result<Self> {
        let mut ht =
            ZendHashTable::with_capacity(vec.len().try_into().map_err(|_| Error::IntegerOverflow)?);

        for val in vec.iter() {
            ht.push(val.clone())?;
        }

        Ok(ht)
    }
}
