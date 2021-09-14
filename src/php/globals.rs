//! Types related to the PHP executor globals.

use crate::bindings::{_zend_executor_globals, ext_php_rs_executor_globals};

use super::types::{array::ZendHashTable, object::ZendObject};

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
    pub fn class_table(&self) -> Option<ZendHashTable> {
        if self.class_table.is_null() {
            return None;
        }

        unsafe { ZendHashTable::from_ptr(self.class_table, false) }.ok()
    }

    pub fn take_exception() -> Option<Box<ZendObject>> {
        let globals = Self::get_mut();

        let mut exception_ptr = std::ptr::null_mut();
        std::mem::swap(&mut exception_ptr, &mut globals.exception);

        if !exception_ptr.is_null() {
            Some(unsafe { Box::from_raw(exception_ptr) })
        } else {
            None
        }
    }
}
