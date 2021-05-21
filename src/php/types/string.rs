//! Represents a string in the PHP world. Similar to a C string, but is reference counted and
//! contains the length of the string, meaning the string can contain the NUL character.

use core::slice;
use std::{
    convert::{TryFrom, TryInto},
    fmt::Debug,
};

use crate::{
    bindings::{
        ext_php_rs_zend_string_init, ext_php_rs_zend_string_release, zend_string,
        zend_string_init_interned,
    },
    errors::{Error, Result},
    functions::c_str,
};

/// String type used in the Zend internals.
/// The actual size of the 'string' differs, as the
/// end of this struct is only 1 char long, but the length
/// inside the struct defines how many characters are in the string.
pub type ZendString = zend_string;

// TODO: Encapsulate `zend_string` inside an object `ZendString`:
// pub struct ZendString {
//     ptr: *mut zend_string,
// }
// impl Drop for ZendString { ... }

// pub struct NewZendString {
//     pub(crate) ptr: *mut zend_string,
//     drop: bool,
// }

// impl NewZendString {
//     /// Creates a new Zend string.
//     ///
//     /// # Parameters
//     ///
//     /// * `str_` - The string to create a Zend string from.
//     /// * `persistent` - Whether the request should relive the request boundary.
//     pub fn new(str_: impl AsRef<str>, persistent: bool) -> Self {
//         let str_ = str_.as_ref();

//         Self {
//             ptr: unsafe { ext_php_rs_zend_string_init(c_str(str_), str_.len() as _, persistent) },
//             drop: true,
//         }
//     }

//     /// Creates a new interned Zend string.
//     ///
//     /// # Parameters
//     ///
//     /// * `str_` - The string to create a Zend string from.
//     pub fn new_interned(str_: impl AsRef<str>) -> Self {
//         let str_ = str_.as_ref();

//         Self {
//             ptr: unsafe { zend_string_init_interned.unwrap()(c_str(str_), str_.len() as _, true) },
//             drop: true,
//         }
//     }

//     /// Releases the Zend string, returning the raw pointer to the `zend_string` object
//     /// and consuming the internal Rust [`NewZendString`] container.
//     pub fn release(mut self) -> *mut zend_string {
//         self.drop = false;
//         self.ptr
//     }
// }

// impl Drop for NewZendString {
//     fn drop(&mut self) {
//         if self.drop && !self.ptr.is_null() {
//             unsafe { ext_php_rs_zend_string_release(self.ptr) };
//         }
//     }
// }

// impl TryFrom<&NewZendString> for String {
//     type Error = Error;

//     fn try_from(s: &NewZendString) -> Result<Self> {
//         let zs = unsafe { s.ptr.as_ref() }.ok_or(Error::InvalidPointer)?;

//         // SAFETY: Zend strings have a length that we know we can read.
//         // By reading this many bytes we will not run into any issues.
//         //
//         // We can safely cast our *const c_char into a *const u8 as both
//         // only occupy one byte.
//         std::str::from_utf8(unsafe {
//             slice::from_raw_parts(zs.val.as_ptr() as *const u8, zs.len as _)
//         })
//         .map(|s| s.to_string())
//         .map_err(|_| Error::InvalidPointer)
//     }
// }

// impl Debug for NewZendString {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let s: Result<String> = self.try_into();
//         match s {
//             Ok(s) => s.fmt(f),
//             Err(_) => Option::<()>::None.fmt(f),
//         }
//     }
// }

impl ZendString {
    /// Creates a new Zend string.
    ///
    /// Note that this returns a raw pointer, and will not be freed by
    /// Rust.
    ///
    /// # Parameters
    ///
    /// * `str_` - The string to create a Zend string from.
    /// * `peresistent` - Whether the request should relive the request boundary.
    pub fn new<S>(str_: S, persistent: bool) -> *mut Self
    where
        S: AsRef<str>,
    {
        let str_ = str_.as_ref();
        unsafe { ext_php_rs_zend_string_init(c_str(str_), str_.len() as u64, persistent) }
    }

    /// Creates a new interned Zend string.
    ///
    /// Note that this returns a raw pointer, and will not be freed by
    /// Rust.
    ///
    /// # Parameters
    ///
    /// * `str_` - The string to create a Zend string from.
    pub fn new_interned<S>(str_: S) -> *mut Self
    where
        S: AsRef<str>,
    {
        let str_ = str_.as_ref();
        unsafe {
            zend_string_init_interned.unwrap()(c_str(str_), str_.len().try_into().unwrap(), true)
        }
    }

    /// Drops a Zend string, releasing its memory.
    ///
    /// # Parameters
    ///
    /// * `ptr` - A pointer to the Zend string to drop.
    pub unsafe fn drop(ptr: *mut Self) {
        if ptr.is_null() {
            return;
        }

        ext_php_rs_zend_string_release(ptr);
    }
}

impl Debug for ZendString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = self.into();
        s.fmt(f)
    }
}

impl From<&ZendString> for String {
    fn from(zs: &ZendString) -> Self {
        let len = zs.len;
        let ptr = zs.val.as_ptr() as *const u8;

        // SAFETY: Zend strings have a length that we know we can read.
        // By reading this many bytes we will not run into any issues.
        //
        // We can safely cast our *const c_char into a *const u8 as both
        // only occupy one byte.
        std::str::from_utf8(unsafe { slice::from_raw_parts(ptr, len as usize) })
            .unwrap()
            .to_string()
    }
}
