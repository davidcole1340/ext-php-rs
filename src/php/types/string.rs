//! Represents a string in the PHP world. Similar to a C string, but is reference counted and
//! contains the length of the string, meaning the string can contain the NUL character.

use std::{
    borrow::{Borrow, Cow},
    convert::TryFrom,
    ffi::{CStr, CString},
    fmt::Debug,
    mem::ManuallyDrop,
    ops::Deref,
    ptr::NonNull,
    slice,
};

use parking_lot::{
    lock_api::{Mutex, RawMutex},
    RawMutex as RawMutexStruct,
};

use crate::{
    bindings::{
        ext_php_rs_zend_string_init, ext_php_rs_zend_string_release, zend_string_init_interned,
    },
    errors::{Error, Result},
};

/// A borrowed Zend-string.
///
/// Although this object does implement [`Sized`], it is in fact not sized. As C cannot represent unsized
/// types, an array of size 1 is used at the end of the type to represent the contents of the string, therefore
/// this type is actually unsized and has no valid constructors. See the owned variant [`ZendString`] to
/// create an owned version of a [`ZendStr`].
///
/// Once the `ptr_metadata` feature lands in stable rust, this type can potentially be changed to a DST using
/// slices and metadata. See the tracking issue here: https://github.com/rust-lang/rust/issues/81513
pub use crate::bindings::zend_string as ZendStr;

impl ZendStr {
    /// Returns the length of the string.
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns true if the string is empty, false otherwise.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the underlying [`CStr`] inside the Zend string.
    pub fn as_c_str(&self) -> &CStr {
        // SAFETY: Zend strings store their readable length in a fat pointer.
        unsafe {
            let slice = slice::from_raw_parts(self.val.as_ptr() as *const u8, self.len());
            CStr::from_bytes_with_nul_unchecked(slice)
        }
    }

    /// Attempts to return a reference to the underlying [`str`] inside the Zend string.
    ///
    /// Returns the [`None`] variant if the [`CStr`] contains non-UTF-8 characters.
    pub fn as_str(&self) -> Option<&str> {
        self.as_c_str().to_str().ok()
    }
}

impl Debug for ZendStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_c_str().fmt(f)
    }
}

impl ToOwned for ZendStr {
    type Owned = ZendString;

    fn to_owned(&self) -> Self::Owned {
        Self::Owned::from_c_str(self.as_c_str(), false)
    }
}

impl PartialEq for ZendStr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_c_str().eq(other.as_c_str())
    }
}

impl<'a> From<&'a ZendStr> for &'a CStr {
    fn from(value: &'a ZendStr) -> Self {
        value.as_c_str()
    }
}

impl<'a> TryFrom<&'a ZendStr> for &'a str {
    type Error = Error;

    fn try_from(value: &'a ZendStr) -> Result<Self> {
        value.as_str().ok_or(Error::InvalidCString)
    }
}

impl<'a> TryFrom<&ZendStr> for String {
    type Error = Error;

    fn try_from(value: &ZendStr) -> Result<Self> {
        value
            .as_str()
            .map(|s| s.to_string())
            .ok_or(Error::InvalidCString)
    }
}

impl<'a> From<&'a ZendStr> for Cow<'a, ZendStr> {
    fn from(value: &'a ZendStr) -> Self {
        Cow::Borrowed(value)
    }
}

/// A type representing an owned Zend string, commonly used throughout the PHP API.
///
/// The type contains an inner pointer to a [`ZendStr`], which is the DST that contains the contents
/// of the string. This type simply provides the required functions to handle the creation and deletion
/// of the internal string.
pub struct ZendString {
    inner: NonNull<ZendStr>,
}

// Adding to the Zend interned string hashtable is not atomic and can be contested when PHP is compiled with ZTS,
// so an empty mutex is used to ensure no collisions occur on the Rust side. Not much we can do about collisions
// on the PHP side.
static INTERNED_LOCK: Mutex<RawMutexStruct, ()> = Mutex::const_new(RawMutex::INIT, ());

impl ZendString {
    /// Creates a new Zend string from a [`str`].
    ///
    /// # Parameters
    ///
    /// * `str` - String content.
    /// * `persistent` - Whether the string should persist through the request boundary.
    ///
    /// # Returns
    ///
    /// Returns a result containing the Zend string if successful. Returns an error if the given
    /// string contains NUL bytes, which cannot be contained inside a C string.
    ///
    /// # Panics
    ///
    /// Panics if the function was unable to allocate memory for the Zend string.
    pub fn new(str: &str, persistent: bool) -> Result<Self> {
        Ok(Self::from_c_str(&CString::new(str)?, persistent))
    }

    /// Creates a new Zend string from a [`CStr`].
    ///
    /// # Parameters
    ///
    /// * `str` - String content.
    /// * `persistent` - Whether the string should persist through the request boundary.
    ///
    /// # Panics
    ///
    /// Panics if the function was unable to allocate memory for the Zend string.
    pub fn from_c_str(str: &CStr, persistent: bool) -> Self {
        let ptr = unsafe {
            ext_php_rs_zend_string_init(str.as_ptr(), str.to_bytes().len() as _, persistent)
        };

        Self {
            inner: NonNull::new(ptr).expect("Failed to allocate for Zend string"),
        }
    }

    /// Creates a new interned Zend string from a [`str`].
    ///
    /// An interned string is only ever stored once and is immutable. PHP stores the string in an
    /// internal hashtable which stores the interned strings.
    ///
    /// As Zend hashtables are not thread-safe, a mutex is used to prevent two interned strings from
    /// being created at the same time.
    ///
    /// # Parameters
    ///
    /// * `str` - String content.
    /// * `persistent` - Whether the string should persist through the request boundary.
    ///
    /// # Returns
    ///
    /// Returns a result containing the Zend string if successful. Returns an error if the given
    /// string contains NUL bytes, which cannot be contained inside a C string.
    ///
    /// # Panics
    ///
    /// Panics if the function was unable to allocate memory for the Zend string.
    pub fn new_interned(str: &str, persistent: bool) -> Result<Self> {
        Ok(Self::interned_from_c_str(&CString::new(str)?, persistent))
    }

    /// Creates a new interned Zend string from a [`CStr`].
    ///
    /// An interned string is only ever stored once and is immutable. PHP stores the string in an
    /// internal hashtable which stores the interned strings.
    ///
    /// As Zend hashtables are not thread-safe, a mutex is used to prevent two interned strings from
    /// being created at the same time.
    ///
    /// # Parameters
    ///
    /// * `str` - String content.
    /// * `persistent` - Whether the string should persist through the request boundary.
    ///
    /// # Panics
    ///
    /// Panics under the following circumstances:
    ///
    /// * The function used to create interned strings has not been set.
    /// * The function could not allocate enough memory for the Zend string.
    pub fn interned_from_c_str(str: &CStr, persistent: bool) -> Self {
        let _lock = INTERNED_LOCK.lock();
        let ptr = unsafe {
            zend_string_init_interned.expect("`zend_string_init_interned` not ready")(
                str.as_ptr(),
                str.to_bytes().len() as _,
                persistent,
            )
        };

        Self {
            inner: NonNull::new(ptr).expect("Failed to allocate for Zend string"),
        }
    }

    /// Returns a reference to the internal [`ZendStr`].
    pub fn as_zend_str(&self) -> &ZendStr {
        // SAFETY: All constructors ensure a valid internal pointer.
        unsafe { self.inner.as_ref() }
    }

    /// Converts the owned Zend string into the internal pointer, bypassing the [`Drop`]
    /// implementation.
    ///
    /// The caller is responsible for freeing the resulting pointer using the `zend_string_release`
    /// function.
    pub fn into_inner(self) -> *mut ZendStr {
        let this = ManuallyDrop::new(self);
        this.inner.as_ptr()
    }
}

impl Drop for ZendString {
    fn drop(&mut self) {
        // SAFETY: All constructors ensure a valid internal pointer.
        unsafe { ext_php_rs_zend_string_release(self.inner.as_ptr()) };
    }
}

impl Deref for ZendString {
    type Target = ZendStr;

    fn deref(&self) -> &Self::Target {
        self.as_zend_str()
    }
}

impl Borrow<ZendStr> for ZendString {
    #[inline]
    fn borrow(&self) -> &ZendStr {
        self.deref()
    }
}

impl AsRef<ZendStr> for ZendString {
    #[inline]
    fn as_ref(&self) -> &ZendStr {
        self
    }
}

impl From<&CStr> for ZendString {
    fn from(value: &CStr) -> Self {
        Self::from_c_str(value, false)
    }
}

impl From<CString> for ZendString {
    fn from(value: CString) -> Self {
        Self::from_c_str(&value, false)
    }
}

impl TryFrom<&str> for ZendString {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        Self::new(value, false)
    }
}

impl TryFrom<String> for ZendString {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        Self::new(value.as_str(), false)
    }
}

impl From<ZendString> for Cow<'_, ZendStr> {
    fn from(value: ZendString) -> Self {
        Cow::Owned(value)
    }
}

impl From<Cow<'_, ZendStr>> for ZendString {
    fn from(value: Cow<'_, ZendStr>) -> Self {
        value.into_owned()
    }
}
