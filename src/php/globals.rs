//! Types related to the PHP executor globals.

use crate::bindings::{_zend_executor_globals, ext_php_rs_executor_globals};

use super::types::array::HashTable;

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

    /// Attempts to retrieve the global class hash table.
    pub fn class_table(&self) -> Option<&HashTable> {
        unsafe { self.class_table.as_ref() }
    }
}
