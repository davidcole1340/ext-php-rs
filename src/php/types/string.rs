//! Represents a string in the PHP world. Similar to a C string, but is reference counted and
//! contains the length of the string, meaning the string can contain the NUL character.

use std::{convert::TryFrom, ffi::CString, fmt::Debug};

use crate::{
    bindings::{
        ext_php_rs_zend_string_init, ext_php_rs_zend_string_release, zend_string,
        zend_string_init_interned,
    },
    errors::{Error, Result},
};

/// A wrapper around the [`zend_string`] used within the Zend API. Essentially a C string, except
/// that the structure contains the length of the string as well as the string being refcounted.
pub struct ZendString {
    ptr: *mut zend_string,
    free: bool,
}

impl ZendString {
    /// Creates a new Zend string. Returns a result containin the string.
    ///
    /// # Parameters
    ///
    /// * `str_` - The string to create a Zend string from.
    /// * `persistent` - Whether the request should relive the request boundary.
    pub fn new(str: &str, persistent: bool) -> Result<Self> {
        Ok(Self {
            ptr: unsafe {
                ext_php_rs_zend_string_init(CString::new(str)?.as_ptr(), str.len() as _, persistent)
            },
            free: true,
        })
    }

    /// Creates a new interned Zend string. Returns a result containing the interned string.
    ///
    /// # Parameters
    ///
    /// * `str_` - The string to create a Zend string from.
    #[allow(clippy::unwrap_used)]
    pub fn new_interned(str_: &str) -> Result<Self> {
        // Unwrap is OK here - `zend_string_init_interned` will be a valid function ptr by the time
        // our extension is loaded.
        Ok(Self {
            ptr: unsafe {
                zend_string_init_interned.unwrap()(
                    CString::new(str_)?.as_ptr(),
                    str_.len() as _,
                    true,
                )
            },
            free: true,
        })
    }

    /// Creates a new [`ZendString`] wrapper from a raw pointer to a [`zend_string`].
    ///
    /// # Parameters
    ///
    /// * `ptr` - A raw pointer to a [`zend_string`].
    /// * `free` - Whether the pointer should be freed when the resulting [`ZendString`] goes
    /// out of scope.
    ///
    /// # Safety
    ///
    /// As a raw pointer is given this function is unsafe, you must ensure the pointer is valid when calling
    /// the function. A simple null check is done but this is not sufficient in most places.
    pub unsafe fn from_ptr(ptr: *mut zend_string, free: bool) -> Result<Self> {
        if ptr.is_null() {
            return Err(Error::InvalidPointer);
        }

        Ok(Self { ptr, free })
    }

    /// Releases the Zend string, returning the raw pointer to the `zend_string` object
    /// and consuming the internal Rust [`ZendString`] container.
    pub fn release(mut self) -> *mut zend_string {
        self.free = false;
        self.ptr
    }

    /// Extracts a string slice containing the contents of the [`ZendString`].
    pub fn as_str(&self) -> Option<&str> {
        // SAFETY: Zend strings have a length that we know we can read.
        // By reading this many bytes we should not run into any issues.
        // The value of the string is represented in C as a `char` array of
        // length 1, but the data can be read up to `ptr.len` bytes.
        unsafe {
            let ptr = self.ptr.as_ref()?;
            let slice = std::slice::from_raw_parts(ptr.val.as_ptr() as *const u8, ptr.len as _);
            std::str::from_utf8(slice).ok()
        }
    }

    /// Borrows the underlying internal pointer of the Zend string.
    pub(crate) fn borrow_ptr(&self) -> *mut zend_string {
        self.ptr
    }
}

impl Drop for ZendString {
    fn drop(&mut self) {
        if self.free && !self.ptr.is_null() {
            unsafe { ext_php_rs_zend_string_release(self.ptr) };
        }
    }
}

impl TryFrom<String> for ZendString {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        ZendString::new(value.as_str(), false)
    }
}

impl TryFrom<ZendString> for String {
    type Error = Error;

    fn try_from(value: ZendString) -> Result<Self> {
        <String as TryFrom<&ZendString>>::try_from(&value)
    }
}

impl TryFrom<&ZendString> for String {
    type Error = Error;

    fn try_from(s: &ZendString) -> Result<Self> {
        s.as_str()
            .map(|s| s.to_string())
            .ok_or(Error::InvalidPointer)
    }
}

impl Debug for ZendString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_str() {
            Some(str) => str.fmt(f),
            None => Option::<()>::None.fmt(f),
        }
    }
}
