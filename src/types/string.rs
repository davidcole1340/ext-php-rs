//! Represents a string in the PHP world. Similar to a C string, but is
//! reference counted and contains the length of the string, meaning the string
//! can contain the NUL character.

use std::{
    borrow::Cow,
    convert::TryFrom,
    ffi::{CStr, CString},
    fmt::Debug,
    slice,
};

use parking_lot::{
    lock_api::{Mutex, RawMutex},
    RawMutex as RawMutexStruct,
};

use crate::{
    boxed::{ZBox, ZBoxable},
    convert::{FromZval, IntoZval},
    error::{Error, Result},
    ffi::{
        ext_php_rs_zend_string_init, ext_php_rs_zend_string_release, zend_string,
        zend_string_init_interned,
    },
    flags::DataType,
    macros::try_from_zval,
    types::Zval,
};

/// A borrowed Zend-string.
///
/// Although this object does implement [`Sized`], it is in fact not sized. As C
/// cannot represent unsized types, an array of size 1 is used at the end of the
/// type to represent the contents of the string, therefore this type is
/// actually unsized. All constructors return [`ZBox<ZendStr>`], the owned
/// varaint.
///
/// Once the `ptr_metadata` feature lands in stable rust, this type can
/// potentially be changed to a DST using slices and metadata. See the tracking issue here: <https://github.com/rust-lang/rust/issues/81513>
pub type ZendStr = zend_string;

// Adding to the Zend interned string hashtable is not atomic and can be
// contested when PHP is compiled with ZTS, so an empty mutex is used to ensure
// no collisions occur on the Rust side. Not much we can do about collisions
// on the PHP side.
static INTERNED_LOCK: Mutex<RawMutexStruct, ()> = Mutex::const_new(RawMutex::INIT, ());

// Clippy complains about there being no `is_empty` function when implementing
// on the alias `ZendStr` :( <https://github.com/rust-lang/rust-clippy/issues/7702>
#[allow(clippy::len_without_is_empty)]
impl ZendStr {
    /// Creates a new Zend string from a [`str`].
    ///
    /// # Parameters
    ///
    /// * `str` - String content.
    /// * `persistent` - Whether the string should persist through the request
    ///   boundary.
    ///
    /// # Returns
    ///
    /// Returns a result containing the Zend string if successful. Returns an
    /// error if the given string contains NUL bytes, which cannot be
    /// contained inside a C string.
    ///
    /// # Panics
    ///
    /// Panics if the function was unable to allocate memory for the Zend
    /// string.
    pub fn new(str: &str, persistent: bool) -> Result<ZBox<Self>> {
        Ok(Self::from_c_str(&CString::new(str)?, persistent))
    }

    /// Creates a new Zend string from a [`CStr`].
    ///
    /// # Parameters
    ///
    /// * `str` - String content.
    /// * `persistent` - Whether the string should persist through the request
    ///   boundary.
    ///
    /// # Panics
    ///
    /// Panics if the function was unable to allocate memory for the Zend
    /// string.
    pub fn from_c_str(str: &CStr, persistent: bool) -> ZBox<Self> {
        unsafe {
            let ptr =
                ext_php_rs_zend_string_init(str.as_ptr(), str.to_bytes().len() as _, persistent);

            ZBox::from_raw(
                ptr.as_mut()
                    .expect("Failed to allocate memory for new Zend string"),
            )
        }
    }

    /// Creates a new interned Zend string from a [`str`].
    ///
    /// An interned string is only ever stored once and is immutable. PHP stores
    /// the string in an internal hashtable which stores the interned
    /// strings.
    ///
    /// As Zend hashtables are not thread-safe, a mutex is used to prevent two
    /// interned strings from being created at the same time.
    ///
    /// # Parameters
    ///
    /// * `str` - String content.
    /// * `persistent` - Whether the string should persist through the request
    ///   boundary.
    ///
    /// # Returns
    ///
    /// Returns a result containing the Zend string if successful. Returns an
    /// error if the given string contains NUL bytes, which cannot be
    /// contained inside a C string.
    ///
    /// # Panics
    ///
    /// Panics if the function was unable to allocate memory for the Zend
    /// string.
    pub fn new_interned(str: &str, persistent: bool) -> Result<ZBox<Self>> {
        Ok(Self::interned_from_c_str(&CString::new(str)?, persistent))
    }

    /// Creates a new interned Zend string from a [`CStr`].
    ///
    /// An interned string is only ever stored once and is immutable. PHP stores
    /// the string in an internal hashtable which stores the interned
    /// strings.
    ///
    /// As Zend hashtables are not thread-safe, a mutex is used to prevent two
    /// interned strings from being created at the same time.
    ///
    /// # Parameters
    ///
    /// * `str` - String content.
    /// * `persistent` - Whether the string should persist through the request
    ///   boundary.
    ///
    /// # Panics
    ///
    /// Panics under the following circumstances:
    ///
    /// * The function used to create interned strings has not been set.
    /// * The function could not allocate enough memory for the Zend string.
    pub fn interned_from_c_str(str: &CStr, persistent: bool) -> ZBox<Self> {
        let _lock = INTERNED_LOCK.lock();

        unsafe {
            let ptr = zend_string_init_interned.expect("`zend_string_init_interned` not ready")(
                str.as_ptr(),
                str.to_bytes().len() as _,
                persistent,
            );

            ZBox::from_raw(
                ptr.as_mut()
                    .expect("Failed to allocate memory for new Zend string"),
            )
        }
    }

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
            let slice = slice::from_raw_parts(self.val.as_ptr() as *const u8, self.len() + 1);
            CStr::from_bytes_with_nul_unchecked(slice)
        }
    }

    /// Attempts to return a reference to the underlying [`str`] inside the Zend
    /// string.
    ///
    /// Returns the [`None`] variant if the [`CStr`] contains non-UTF-8
    /// characters.
    pub fn as_str(&self) -> Option<&str> {
        self.as_c_str().to_str().ok()
    }
}

unsafe impl ZBoxable for ZendStr {
    fn free(&mut self) {
        unsafe { ext_php_rs_zend_string_release(self) };
    }
}

impl Debug for ZendStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_c_str().fmt(f)
    }
}

impl ToOwned for ZendStr {
    type Owned = ZBox<ZendStr>;

    fn to_owned(&self) -> Self::Owned {
        Self::from_c_str(self.as_c_str(), false)
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

impl From<&CStr> for ZBox<ZendStr> {
    fn from(value: &CStr) -> Self {
        ZendStr::from_c_str(value, false)
    }
}

impl From<CString> for ZBox<ZendStr> {
    fn from(value: CString) -> Self {
        ZendStr::from_c_str(&value, false)
    }
}

impl TryFrom<&str> for ZBox<ZendStr> {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        ZendStr::new(value, false)
    }
}

impl TryFrom<String> for ZBox<ZendStr> {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        ZendStr::new(value.as_str(), false)
    }
}

impl From<ZBox<ZendStr>> for Cow<'_, ZendStr> {
    fn from(value: ZBox<ZendStr>) -> Self {
        Cow::Owned(value)
    }
}

impl From<Cow<'_, ZendStr>> for ZBox<ZendStr> {
    fn from(value: Cow<'_, ZendStr>) -> Self {
        value.into_owned()
    }
}

macro_rules! try_into_zval_str {
    ($type: ty) => {
        impl TryFrom<$type> for Zval {
            type Error = Error;

            fn try_from(value: $type) -> Result<Self> {
                let mut zv = Self::new();
                zv.set_string(&value, false)?;
                Ok(zv)
            }
        }

        impl IntoZval for $type {
            const TYPE: DataType = DataType::String;

            fn set_zval(self, zv: &mut Zval, persistent: bool) -> Result<()> {
                zv.set_string(&self, persistent)
            }
        }
    };
}

try_into_zval_str!(String);
try_into_zval_str!(&str);
try_from_zval!(String, string, String);

impl<'a> FromZval<'a> for &'a str {
    const TYPE: DataType = DataType::String;

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        zval.str()
    }
}
