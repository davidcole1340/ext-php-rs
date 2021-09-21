//! Represents an array in PHP. As all arrays in PHP are associative arrays, they are represented
//! by hash tables.

use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    ffi::CString,
    fmt::Debug,
    iter::FromIterator,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    ptr::NonNull,
    u64,
};

use crate::{
    bindings::{
        _Bucket, _zend_new_array, zend_array_destroy, zend_array_dup, zend_hash_clean,
        zend_hash_index_del, zend_hash_index_find, zend_hash_index_update,
        zend_hash_next_index_insert, zend_hash_str_del, zend_hash_str_find, zend_hash_str_update,
        HT_MIN_SIZE,
    },
    errors::{Error, Result},
    php::enums::DataType,
};

use super::{
    string::ZendString,
    zval::{FromZval, IntoZval, Zval},
};

/// PHP array, which is represented in memory as a hashtable.
pub use crate::bindings::HashTable;

impl HashTable {
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
        unsafe { zend_hash_clean(self) }
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
    pub fn get(&self, key: &'_ str) -> Option<&Zval> {
        let str = CString::new(key).ok()?;
        unsafe { zend_hash_str_find(self, str.as_ptr(), key.len() as _).as_ref() }
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
        unsafe { zend_hash_index_find(self, key).as_ref() }
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
    pub fn remove<K>(&mut self, key: &str) -> Option<()> {
        let result =
            unsafe { zend_hash_str_del(self, CString::new(key).ok()?.as_ptr(), key.len() as _) };

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
    pub fn remove_index(&mut self, key: u64) -> Option<()> {
        let result = unsafe { zend_hash_index_del(self, key) };

        if result < 0 {
            None
        } else {
            Some(())
        }
    }

    /// Attempts to insert an item into the hash table, or update if the key already exists.
    /// Returns nothing in a result if successful.
    ///
    /// # Parameters
    ///
    /// * `key` - The key to insert the value at in the hash table.
    /// * `value` - The value to insert into the hash table.
    pub fn insert<V>(&mut self, key: &str, val: V) -> Result<()>
    where
        V: IntoZval,
    {
        let mut val = val.into_zval(false)?;
        unsafe {
            zend_hash_str_update(
                self,
                CString::new(key)?.as_ptr(),
                key.len() as u64,
                &mut val,
            )
        };
        val.release();
        Ok(())
    }

    /// Inserts an item into the hash table at a specified index, or updates if the key already exists.
    /// Returns nothing in a result if successful.
    ///
    /// # Parameters
    ///
    /// * `key` - The index at which the value should be inserted.
    /// * `val` - The value to insert into the hash table.
    pub fn insert_at_index<V>(&mut self, key: u64, val: V) -> Result<()>
    where
        V: IntoZval,
    {
        let mut val = val.into_zval(false)?;
        unsafe { zend_hash_index_update(self, key, &mut val) };
        val.release();
        Ok(())
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
        let mut val = val.into_zval(false)?;
        unsafe { zend_hash_next_index_insert(self, &mut val) };
        val.release();

        Ok(())
    }

    /// Returns an iterator over the key(s) and value contained inside the hashtable.
    #[inline]
    pub fn iter(&self) -> Iter {
        Iter::new(self)
    }

    /// Returns an iterator over the values contained inside the hashtable, as if it was a set or list.
    #[inline]
    pub fn values(&self) -> Values {
        Values::new(self)
    }

    /// Clones the hash table, returning an [`OwnedHashTable`].
    pub fn to_owned(&self) -> OwnedHashTable {
        let ptr = unsafe { zend_array_dup(self as *const HashTable as *mut HashTable) };
        unsafe { OwnedHashTable::from_ptr(ptr) }
    }
}

impl Debug for HashTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(
                self.iter()
                    .map(|(k, k2, v)| (k2.unwrap_or_else(|| k.to_string()), v)),
            )
            .finish()
    }
}

/// Immutable iterator upon a reference to a hashtable.
pub struct Iter<'a> {
    ht: &'a HashTable,
    pos: Option<NonNull<_Bucket>>,
    end: Option<NonNull<_Bucket>>,
}

impl<'a> Iter<'a> {
    /// Creates a new iterator over a hashtable.
    ///
    /// # Parameters
    ///
    /// * `ht` - The hashtable to iterate.
    pub fn new(ht: &'a HashTable) -> Self {
        Self {
            ht,
            pos: NonNull::new(ht.arData),
            end: NonNull::new(unsafe { ht.arData.offset(ht.nNumUsed as isize) }),
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = (u64, Option<String>, &'a Zval);

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.pos?;

        if pos == self.end? {
            return None;
        }

        let bucket = unsafe { pos.as_ref() };
        let key = unsafe { ZendString::from_ptr(bucket.key, false) }
            .and_then(|s| s.try_into())
            .ok();

        self.pos = NonNull::new(unsafe { pos.as_ptr().offset(1) });

        Some((bucket.h, key, &bucket.val))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.ht.len()
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    fn len(&self) -> usize {
        self.ht.len()
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let end = self.end?;

        if end == self.pos? {
            return None;
        }

        let new_end = NonNull::new(unsafe { end.as_ptr().offset(-1) })?;
        let bucket = unsafe { new_end.as_ref() };
        let key = unsafe { ZendString::from_ptr(bucket.key, false) }
            .and_then(|s| s.try_into())
            .ok();
        self.end = Some(new_end);

        Some((bucket.h, key, &bucket.val))
    }
}

/// Immutable iterator which iterates over the values of the hashtable, as it was a set or list.
pub struct Values<'a>(Iter<'a>);

impl<'a> Values<'a> {
    /// Creates a new iterator over a hashtables values.
    ///
    /// # Parameters
    ///
    /// * `ht` - The hashtable to iterate.
    pub fn new(ht: &'a HashTable) -> Self {
        Self(Iter::new(ht))
    }
}

impl<'a> Iterator for Values<'a> {
    type Item = &'a Zval;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, _, zval)| zval)
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.0.count()
    }
}

impl<'a> ExactSizeIterator for Values<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> DoubleEndedIterator for Values<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(_, _, zval)| zval)
    }
}

/// A container used to 'own' a Zend hashtable. Dereferences to a reference to [`HashTable`].
///
/// When this struct is dropped, it will also destroy the internal hashtable, unless the `into_raw`
/// function is used.
pub struct OwnedHashTable {
    ptr: NonNull<HashTable>,
}

impl OwnedHashTable {
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
        // SAFETY: PHP allocater handles the creation of the array.
        unsafe {
            let ptr = _zend_new_array(size);
            Self::from_ptr(ptr)
        }
    }

    /// Creates an owned hashtable from a hashtable pointer, which will be freed when the
    /// resulting Rust object is dropped.
    ///
    /// # Parameters
    ///
    /// * `ptr` - Hashtable pointer.
    ///
    /// # Panics
    ///
    /// Panics if the given pointer is null.
    ///
    /// # Safety
    ///
    /// Caller must ensure that the given pointer is a valid hashtable pointer, including
    /// non-null and properly aligned.
    pub unsafe fn from_ptr(ptr: *mut HashTable) -> Self {
        Self {
            ptr: NonNull::new(ptr).expect("Invalid hashtable pointer given"),
        }
    }

    /// Returns the inner pointer to the hashtable, without destroying the
    pub fn into_inner(self) -> *mut HashTable {
        let this = ManuallyDrop::new(self);
        this.ptr.as_ptr()
    }
}

impl Deref for OwnedHashTable {
    type Target = HashTable;

    fn deref(&self) -> &Self::Target {
        // SAFETY: all constructors ensure a valid ptr is present
        unsafe { self.ptr.as_ref() }
    }
}

impl DerefMut for OwnedHashTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: all constructors ensure a valid, owned ptr is present
        unsafe { self.ptr.as_mut() }
    }
}

impl Debug for OwnedHashTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

impl Default for OwnedHashTable {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for OwnedHashTable {
    fn clone(&self) -> Self {
        self.deref().to_owned()
    }
}

impl Drop for OwnedHashTable {
    fn drop(&mut self) {
        unsafe { zend_array_destroy(self.ptr.as_mut()) };
    }
}

impl IntoZval for OwnedHashTable {
    const TYPE: DataType = DataType::Array;

    fn set_zval(self, zv: &mut Zval, _: bool) -> Result<()> {
        zv.set_hashtable(self);
        Ok(())
    }
}

impl<'a> FromZval<'a> for &'a HashTable {
    const TYPE: DataType = DataType::Array;

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        zval.array()
    }
}

///////////////////////////////////////////
//// HashMap
///////////////////////////////////////////

impl<V> TryFrom<&HashTable> for HashMap<String, V>
where
    for<'a> V: FromZval<'a>,
{
    type Error = Error;

    fn try_from(value: &HashTable) -> Result<Self> {
        let mut hm = HashMap::with_capacity(value.len());

        for (idx, key, val) in value.iter() {
            hm.insert(
                key.unwrap_or_else(|| idx.to_string()),
                V::from_zval(val).ok_or_else(|| Error::ZvalConversion(val.get_type()))?,
            );
        }

        Ok(hm)
    }
}

impl<K, V> TryFrom<HashMap<K, V>> for OwnedHashTable
where
    K: AsRef<str>,
    V: IntoZval,
{
    type Error = Error;

    fn try_from(value: HashMap<K, V>) -> Result<Self> {
        let mut ht = OwnedHashTable::with_capacity(
            value.len().try_into().map_err(|_| Error::IntegerOverflow)?,
        );

        for (k, v) in value.into_iter() {
            ht.insert(k.as_ref(), v)?;
        }

        Ok(ht)
    }
}

impl<K, V> IntoZval for HashMap<K, V>
where
    K: AsRef<str>,
    V: IntoZval,
{
    const TYPE: DataType = DataType::Array;

    fn set_zval(self, zv: &mut Zval, _: bool) -> Result<()> {
        let arr = self.try_into()?;
        zv.set_hashtable(arr);
        Ok(())
    }
}

impl<T> FromZval<'_> for HashMap<String, T>
where
    for<'a> T: FromZval<'a>,
{
    const TYPE: DataType = DataType::Array;

    fn from_zval(zval: &Zval) -> Option<Self> {
        zval.array().and_then(|arr| arr.try_into().ok())
    }
}

///////////////////////////////////////////
//// Vec
///////////////////////////////////////////

impl<T> TryFrom<&HashTable> for Vec<T>
where
    for<'a> T: FromZval<'a>,
{
    type Error = Error;

    fn try_from(value: &HashTable) -> Result<Self> {
        let mut vec = Vec::with_capacity(value.len());

        for (_, _, val) in value.iter() {
            vec.push(T::from_zval(val).ok_or_else(|| Error::ZvalConversion(val.get_type()))?);
        }

        Ok(vec)
    }
}

impl<T> TryFrom<Vec<T>> for OwnedHashTable
where
    T: IntoZval,
{
    type Error = Error;

    fn try_from(value: Vec<T>) -> Result<Self> {
        let mut ht = OwnedHashTable::with_capacity(
            value.len().try_into().map_err(|_| Error::IntegerOverflow)?,
        );

        for val in value.into_iter() {
            ht.push(val)?;
        }

        Ok(ht)
    }
}

impl<T> IntoZval for Vec<T>
where
    T: IntoZval,
{
    const TYPE: DataType = DataType::Array;

    fn set_zval(self, zv: &mut Zval, _: bool) -> Result<()> {
        let arr = self.try_into()?;
        zv.set_hashtable(arr);
        Ok(())
    }
}

impl<T> FromZval<'_> for Vec<T>
where
    for<'a> T: FromZval<'a>,
{
    const TYPE: DataType = DataType::Array;

    fn from_zval(zval: &Zval) -> Option<Self> {
        zval.array().and_then(|arr| arr.try_into().ok())
    }
}

impl FromIterator<Zval> for OwnedHashTable {
    fn from_iter<T: IntoIterator<Item = Zval>>(iter: T) -> Self {
        let mut ht = OwnedHashTable::new();
        for item in iter.into_iter() {
            // Inserting a zval cannot fail, as `push` only returns `Err` if converting `val` to a zval
            // fails.
            let _ = ht.push(item);
        }
        ht
    }
}

impl FromIterator<(u64, Zval)> for OwnedHashTable {
    fn from_iter<T: IntoIterator<Item = (u64, Zval)>>(iter: T) -> Self {
        let mut ht = OwnedHashTable::new();
        for (key, val) in iter.into_iter() {
            // Inserting a zval cannot fail, as `push` only returns `Err` if converting `val` to a zval
            // fails.
            let _ = ht.insert_at_index(key, val);
        }
        ht
    }
}

impl<'a> FromIterator<(&'a str, Zval)> for OwnedHashTable {
    fn from_iter<T: IntoIterator<Item = (&'a str, Zval)>>(iter: T) -> Self {
        let mut ht = OwnedHashTable::new();
        for (key, val) in iter.into_iter() {
            // Inserting a zval cannot fail, as `push` only returns `Err` if converting `val` to a zval
            // fails.
            let _ = ht.insert(key, val);
        }
        ht
    }
}
