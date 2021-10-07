//! Types related to the PHP executor globals.

use crate::ffi::{_zend_executor_globals, ext_php_rs_executor_globals};

use crate::types::{HashTable, ZendObject};

/// Stores global variables used in the PHP executor.
pub type ExecutorGlobals = _zend_executor_globals;

impl ExecutorGlobals {
    /// Returns a static reference to the PHP executor globals.
    pub fn get() -> &'static Self {
        // SAFETY: PHP executor globals are statically declared therefore should never
        // return an invalid pointer.
        unsafe { ext_php_rs_executor_globals().as_ref() }
            .expect("Static executor globals were invalid")
    }

    fn get_mut() -> &'static mut Self {
        // SAFETY: PHP executor globals are statically declared therefore should never
        // return an invalid pointer.
        // TODO: Should this be syncronized?
        unsafe { ext_php_rs_executor_globals().as_mut() }
            .expect("Static executor globals were invalid")
    }

    /// Attempts to retrieve the global class hash table.
    pub fn class_table(&self) -> Option<&HashTable> {
        unsafe { self.class_table.as_ref() }
    }

    /// Attempts to extract the last PHP exception captured by the interpreter.
    ///
    /// Note that the caller is responsible for freeing the memory here or it'll leak.
    pub fn take_exception() -> Option<*mut ZendObject> {
        let globals = Self::get_mut();

        let mut exception_ptr = std::ptr::null_mut();
        std::mem::swap(&mut exception_ptr, &mut globals.exception);

        if !exception_ptr.is_null() {
            Some(exception_ptr)
        } else {
            None
        }
    }
}
