//! Represents a string in the PHP world. Similar to a C string, but is reference counted and
//! contains the length of the string, meaning the string can contain the NUL character.

use core::slice;
use std::{convert::TryInto, fmt::Debug};

use crate::{
    bindings::{ext_php_rs_zend_string_init, zend_string, zend_string_init_interned},
    functions::c_str,
};

/// String type used in the Zend internals.
/// The actual size of the 'string' differs, as the
/// end of this struct is only 1 char long, but the length
/// inside the struct defines how many characters are in the string.
pub type ZendString = zend_string;

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
