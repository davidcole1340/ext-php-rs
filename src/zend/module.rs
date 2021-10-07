//! Builder and objects for creating modules in PHP. A module is the base of a PHP extension.

use crate::ffi::zend_module_entry;

/// A Zend module entry. Alias.
pub type ModuleEntry = zend_module_entry;

impl ModuleEntry {
    /// Converts the module entry into a raw pointer, releasing it to the C world.
    pub fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }
}
