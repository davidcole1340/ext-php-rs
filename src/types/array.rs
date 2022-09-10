//! Represents an array in PHP. As all arrays in PHP are associative arrays,
//! they are represented by hash tables.

use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    ffi::CString,
    fmt::Debug,
    iter::FromIterator,
    ptr::NonNull,
    u64,
};

use crate::{
    boxed::{ZBox, ZBoxable},
    convert::{FromZval, IntoZval},
    error::{Error, Result},
    ffi::{
        _Bucket, _zend_new_array, zend_array_destroy, zend_array_dup, zend_hash_clean,
        zend_hash_index_del, zend_hash_index_find, zend_hash_index_update,
        zend_hash_next_index_insert, zend_hash_str_del, zend_hash_str_find, zend_hash_str_update,
        HT_MIN_SIZE,
    },
    flags::DataType,
    types::Zval,
};

/// A PHP hashtable.
///
/// In PHP, arrays are represented as hashtables. This allows you to push values
/// onto the end of the array like a vector, while also allowing you to insert
/// at arbitrary string key indexes.
///
/// A PHP hashtable stores values as [`Zval`]s. This allows you to insert
/// different types into the same hashtable. Types must implement [`IntoZval`]
/// to be able to be inserted into the hashtable.
///
/// # Examples
///
/// ```no_run
/// use ext_php_rs::types::ZendHashTable;
///
/// let mut ht = ZendHashTable::new();
/// ht.push(1);
/// ht.push("Hello, world!");
/// ht.insert("Like", "Hashtable");
///
/// assert_eq!(ht.len(), 3);
/// assert_eq!(ht.get_index(0).and_then(|zv| zv.long()), Some(1));
/// ```
pub type ZendHashTable = crate::ffi::HashTable;

// Clippy complains about there being no `is_empty` function when implementing
// on the alias `ZendStr` :( <https://github.com/rust-lang/rust-clippy/issues/7702>
#[allow(clippy::len_without_is_empty)]
impl ZendHashTable {
    /// Creates a new, empty, PHP hashtable, returned inside a [`ZBox`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ext_php_rs::types::ZendHashTable;
    ///
    /// let ht = ZendHashTable::new();
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if memory for the hashtable could not be allocated.
    pub fn new() -> ZBox<Self> {
        Self::with_capacity(HT_MIN_SIZE)
    }

    /// Creates a new, empty, PHP hashtable with an initial size, returned
    /// inside a [`ZBox`].
    ///
    /// # Parameters
    ///
    /// * `size` - The size to initialize the array with.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ext_php_rs::types::ZendHashTable;
    ///
    /// let ht = ZendHashTable::with_capacity(10);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if memory for the hashtable could not be allocated.
    pub fn with_capacity(size: u32) -> ZBox<Self> {
        unsafe {
            // SAFETY: PHP allocater handles the creation of the array.
            let ptr = _zend_new_array(size);

            // SAFETY: `as_mut()` checks if the pointer is null, and panics if it is not.
            ZBox::from_raw(
                ptr.as_mut()
                    .expect("Failed to allocate memory for hashtable"),
            )
        }
    }

    /// Returns the current number of elements in the array.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ext_php_rs::types::ZendHashTable;
    ///
    /// let mut ht = ZendHashTable::new();
    ///
    /// ht.push(1);
    /// ht.push("Hello, world");
    ///
    /// assert_eq!(ht.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.nNumOfElements as usize
    }

    /// Returns whether the hash table is empty.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ext_php_rs::types::ZendHashTable;
    ///
    /// let mut ht = ZendHashTable::new();
    ///
    /// assert_eq!(ht.is_empty(), true);
    ///
    /// ht.push(1);
    /// ht.push("Hello, world");
    ///
    /// assert_eq!(ht.is_empty(), false);
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears the hash table, removing all values.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ext_php_rs::types::ZendHashTable;
    ///
    /// let mut ht = ZendHashTable::new();
    ///
    /// ht.insert("test", "hello world");
    /// assert_eq!(ht.is_empty(), false);
    ///
    /// ht.clear();
    /// assert_eq!(ht.is_empty(), true);
    /// ```
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
    /// * `Some(&Zval)` - A reference to the zval at the position in the hash
    ///   table.
    /// * `None` - No value at the given position was found.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ext_php_rs::types::ZendHashTable;
    ///
    /// let mut ht = ZendHashTable::new();
    ///
    /// ht.insert("test", "hello world");
    /// assert_eq!(ht.get("test").and_then(|zv| zv.str()), Some("hello world"));
    /// ```
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
    /// * `Some(&Zval)` - A reference to the zval at the position in the hash
    ///   table.
    /// * `None` - No value at the given position was found.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ext_php_rs::types::ZendHashTable;
    ///
    /// let mut ht = ZendHashTable::new();
    ///
    /// ht.push(100);
    /// assert_eq!(ht.get_index(0).and_then(|zv| zv.long()), Some(100));
    /// ```
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
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ext_php_rs::types::ZendHashTable;
    ///
    /// let mut ht = ZendHashTable::new();
    ///
    /// ht.insert("test", "hello world");
    /// assert_eq!(ht.len(), 1);
    ///
    /// ht.remove("test");
    /// assert_eq!(ht.len(), 0);
    /// ```
    pub fn remove(&mut self, key: &str) -> Option<()> {
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
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ext_php_rs::types::ZendHashTable;
    ///
    /// let mut ht = ZendHashTable::new();
    ///
    /// ht.push("hello");
    /// assert_eq!(ht.len(), 1);
    ///
    /// ht.remove_index(0);
    /// assert_eq!(ht.len(), 0);
    /// ```
    pub fn remove_index(&mut self, key: u64) -> Option<()> {
        let result = unsafe { zend_hash_index_del(self, key) };

        if result < 0 {
            None
        } else {
            Some(())
        }
    }

    /// Attempts to insert an item into the hash table, or update if the key
    /// already exists. Returns nothing in a result if successful.
    ///
    /// # Parameters
    ///
    /// * `key` - The key to insert the value at in the hash table.
    /// * `value` - The value to insert into the hash table.
    ///
    /// # Returns
    ///
    /// Returns nothing in a result on success. Returns an error if the key
    /// could not be converted into a [`CString`], or converting the value into
    /// a [`Zval`] failed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ext_php_rs::types::ZendHashTable;
    ///
    /// let mut ht = ZendHashTable::new();
    ///
    /// ht.insert("a", "A");
    /// ht.insert("b", "B");
    /// ht.insert("c", "C");
    /// assert_eq!(ht.len(), 3);
    /// ```
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

    /// Inserts an item into the hash table at a specified index, or updates if
    /// the key already exists. Returns nothing in a result if successful.
    ///
    /// # Parameters
    ///
    /// * `key` - The index at which the value should be inserted.
    /// * `val` - The value to insert into the hash table.
    ///
    /// # Returns
    ///
    /// Returns nothing in a result on success. Returns an error if converting
    /// the value into a [`Zval`] failed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ext_php_rs::types::ZendHashTable;
    ///
    /// let mut ht = ZendHashTable::new();
    ///
    /// ht.insert_at_index(0, "A");
    /// ht.insert_at_index(5, "B");
    /// ht.insert_at_index(0, "C"); // notice overriding index 0
    /// assert_eq!(ht.len(), 2);
    /// ```
    pub fn insert_at_index<V>(&mut self, key: u64, val: V) -> Result<()>
    where
        V: IntoZval,
    {
        let mut val = val.into_zval(false)?;
        unsafe { zend_hash_index_update(self, key, &mut val) };
        val.release();
        Ok(())
    }

    /// Pushes an item onto the end of the hash table. Returns a result
    /// containing nothing if the element was successfully inserted.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to insert into the hash table.
    ///
    /// # Returns
    ///
    /// Returns nothing in a result on success. Returns an error if converting
    /// the value into a [`Zval`] failed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ext_php_rs::types::ZendHashTable;
    ///
    /// let mut ht = ZendHashTable::new();
    ///
    /// ht.push("a");
    /// ht.push("b");
    /// ht.push("c");
    /// assert_eq!(ht.len(), 3);
    /// ```
    pub fn push<V>(&mut self, val: V) -> Result<()>
    where
        V: IntoZval,
    {
        let mut val = val.into_zval(false)?;
        unsafe { zend_hash_next_index_insert(self, &mut val) };
        val.release();

        Ok(())
    }

    /// Checks if the hashtable only contains numerical keys.
    ///
    /// # Returns
    ///
    /// True if all keys on the hashtable are numerical.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ext_php_rs::types::ZendHashTable;
    ///
    /// let mut ht = ZendHashTable::new();
    ///
    /// ht.push(0);
    /// ht.push(3);
    /// ht.push(9);
    /// assert!(ht.has_numerical_keys());
    ///
    /// ht.insert("obviously not numerical", 10);
    /// assert!(!ht.has_numerical_keys());
    /// ```
    pub fn has_numerical_keys(&self) -> bool {
        !self.iter().any(|(_, k, _)| k.is_some())
    }

    /// Checks if the hashtable has numerical, sequential keys.
    ///
    /// # Returns
    ///
    /// True if all keys on the hashtable are numerical and are in sequential
    /// order (i.e. starting at 0 and not skipping any keys).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ext_php_rs::types::ZendHashTable;
    ///
    /// let mut ht = ZendHashTable::new();
    ///
    /// ht.push(0);
    /// ht.push(3);
    /// ht.push(9);
    /// assert!(ht.has_sequential_keys());
    ///
    /// ht.insert_at_index(90, 10);
    /// assert!(!ht.has_sequential_keys());
    /// ```
    pub fn has_sequential_keys(&self) -> bool {
        !self
            .iter()
            .enumerate()
            .any(|(i, (k, strk, _))| i as u64 != k || strk.is_some())
    }

    /// Returns an iterator over the key(s) and value contained inside the
    /// hashtable.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ext_php_rs::types::ZendHashTable;
    ///
    /// let mut ht = ZendHashTable::new();
    ///
    /// for (idx, key, val) in ht.iter() {
    /// //   ^ Index if inserted at an index.
    /// //        ^ Optional string key, if inserted like a hashtable.
    /// //             ^ Inserted value.
    ///
    ///     dbg!(idx, key, val);
    /// }
    #[inline]
    pub fn iter(&self) -> Iter {
        Iter::new(self)
    }

    /// Returns an iterator over the values contained inside the hashtable, as
    /// if it was a set or list.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ext_php_rs::types::ZendHashTable;
    ///
    /// let mut ht = ZendHashTable::new();
    ///
    /// for val in ht.values() {
    ///     dbg!(val);
    /// }
    #[inline]
    pub fn values(&self) -> Values {
        Values::new(self)
    }
}

unsafe impl ZBoxable for ZendHashTable {
    fn free(&mut self) {
        // SAFETY: ZBox has immutable access to `self`.
        unsafe { zend_array_destroy(self) }
    }
}

impl Debug for ZendHashTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(
                self.iter()
                    .map(|(k, k2, v)| (k2.unwrap_or_else(|| k.to_string()), v)),
            )
            .finish()
    }
}

impl ToOwned for ZendHashTable {
    type Owned = ZBox<ZendHashTable>;

    fn to_owned(&self) -> Self::Owned {
        unsafe {
            // SAFETY: FFI call does not modify `self`, returns a new hashtable.
            let ptr = zend_array_dup(self as *const ZendHashTable as *mut ZendHashTable);

            // SAFETY: `as_mut()` checks if the pointer is null, and panics if it is not.
            ZBox::from_raw(
                ptr.as_mut()
                    .expect("Failed to allocate memory for hashtable"),
            )
        }
    }
}

/// Immutable iterator upon a reference to a hashtable.
pub struct Iter<'a> {
    ht: &'a ZendHashTable,
    pos: Option<NonNull<_Bucket>>,
    end: Option<NonNull<_Bucket>>,
}

impl<'a> Iter<'a> {
    /// Creates a new iterator over a hashtable.
    ///
    /// # Parameters
    ///
    /// * `ht` - The hashtable to iterate.
    pub fn new(ht: &'a ZendHashTable) -> Self {
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
        let key = unsafe { bucket.key.as_ref() }.and_then(|s| s.try_into().ok());

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
        let key = unsafe { bucket.key.as_ref() }.and_then(|s| s.try_into().ok());
        self.end = Some(new_end);

        Some((bucket.h, key, &bucket.val))
    }
}

/// Immutable iterator which iterates over the values of the hashtable, as it
/// was a set or list.
pub struct Values<'a>(Iter<'a>);

impl<'a> Values<'a> {
    /// Creates a new iterator over a hashtables values.
    ///
    /// # Parameters
    ///
    /// * `ht` - The hashtable to iterate.
    pub fn new(ht: &'a ZendHashTable) -> Self {
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

impl Default for ZBox<ZendHashTable> {
    fn default() -> Self {
        ZendHashTable::new()
    }
}

impl Clone for ZBox<ZendHashTable> {
    fn clone(&self) -> Self {
        (**self).to_owned()
    }
}

impl IntoZval for ZBox<ZendHashTable> {
    const TYPE: DataType = DataType::Array;

    fn set_zval(self, zv: &mut Zval, _: bool) -> Result<()> {
        zv.set_hashtable(self);
        Ok(())
    }
}

impl<'a> FromZval<'a> for &'a ZendHashTable {
    const TYPE: DataType = DataType::Array;

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        zval.array()
    }
}

///////////////////////////////////////////
//// HashMap
///////////////////////////////////////////

impl<'a, V> TryFrom<&'a ZendHashTable> for HashMap<String, V>
where
    V: FromZval<'a>,
{
    type Error = Error;

    fn try_from(value: &'a ZendHashTable) -> Result<Self> {
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

impl<K, V> TryFrom<HashMap<K, V>> for ZBox<ZendHashTable>
where
    K: AsRef<str>,
    V: IntoZval,
{
    type Error = Error;

    fn try_from(value: HashMap<K, V>) -> Result<Self> {
        let mut ht = ZendHashTable::with_capacity(
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

impl<'a, T> FromZval<'a> for HashMap<String, T>
where
    T: FromZval<'a>,
{
    const TYPE: DataType = DataType::Array;

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        zval.array().and_then(|arr| arr.try_into().ok())
    }
}

///////////////////////////////////////////
//// Vec
///////////////////////////////////////////

impl<'a, T> TryFrom<&'a ZendHashTable> for Vec<T>
where
    T: FromZval<'a>,
{
    type Error = Error;

    fn try_from(value: &'a ZendHashTable) -> Result<Self> {
        let mut vec = Vec::with_capacity(value.len());

        for (_, _, val) in value.iter() {
            vec.push(T::from_zval(val).ok_or_else(|| Error::ZvalConversion(val.get_type()))?);
        }

        Ok(vec)
    }
}

impl<T> TryFrom<Vec<T>> for ZBox<ZendHashTable>
where
    T: IntoZval,
{
    type Error = Error;

    fn try_from(value: Vec<T>) -> Result<Self> {
        let mut ht = ZendHashTable::with_capacity(
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

impl<'a, T> FromZval<'a> for Vec<T>
where
    T: FromZval<'a>,
{
    const TYPE: DataType = DataType::Array;

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        zval.array().and_then(|arr| arr.try_into().ok())
    }
}

impl FromIterator<Zval> for ZBox<ZendHashTable> {
    fn from_iter<T: IntoIterator<Item = Zval>>(iter: T) -> Self {
        let mut ht = ZendHashTable::new();
        for item in iter.into_iter() {
            // Inserting a zval cannot fail, as `push` only returns `Err` if converting
            // `val` to a zval fails.
            let _ = ht.push(item);
        }
        ht
    }
}

impl FromIterator<(u64, Zval)> for ZBox<ZendHashTable> {
    fn from_iter<T: IntoIterator<Item = (u64, Zval)>>(iter: T) -> Self {
        let mut ht = ZendHashTable::new();
        for (key, val) in iter.into_iter() {
            // Inserting a zval cannot fail, as `push` only returns `Err` if converting
            // `val` to a zval fails.
            let _ = ht.insert_at_index(key, val);
        }
        ht
    }
}

impl<'a> FromIterator<(&'a str, Zval)> for ZBox<ZendHashTable> {
    fn from_iter<T: IntoIterator<Item = (&'a str, Zval)>>(iter: T) -> Self {
        let mut ht = ZendHashTable::new();
        for (key, val) in iter.into_iter() {
            // Inserting a zval cannot fail, as `push` only returns `Err` if converting
            // `val` to a zval fails.
            let _ = ht.insert(key, val);
        }
        ht
    }
}
