//! Builder and objects used to create functions and methods in PHP.

use std::{os::raw::c_char, ptr};

use crate::ffi::zend_function_entry;

/// A Zend function entry. Alias.
pub type FunctionEntry = zend_function_entry;

impl FunctionEntry {
    /// Returns an empty function entry, signifing the end of a function list.
    pub fn end() -> Self {
        Self {
            fname: ptr::null() as *const c_char,
            handler: None,
            arg_info: ptr::null(),
            num_args: 0,
            flags: 0,
        }
    }

    /// Converts the function entry into a raw and pointer, releasing it to the
    /// C world.
    pub fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }
}
