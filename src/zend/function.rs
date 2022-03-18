//! Builder for creating functions and methods in PHP.

use std::{fmt::Debug, os::raw::c_char, ptr};

use crate::ffi::zend_function_entry;

/// A Zend function entry.
pub type FunctionEntry = zend_function_entry;

impl Debug for FunctionEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("_zend_function_entry")
            .field("fname", &self.fname)
            .field("arg_info", &self.arg_info)
            .field("num_args", &self.num_args)
            .field("flags", &self.flags)
            .finish()
    }
}

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
