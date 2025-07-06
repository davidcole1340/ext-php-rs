//! Builder and objects for creating modules in PHP. A module is the base of a
//! PHP extension.

use crate::ffi::sapi_module_struct;

/// A Zend module entry, also known as an extension.
pub type SapiModule = sapi_module_struct;

unsafe impl Send for SapiModule {}
unsafe impl Sync for SapiModule {}

impl SapiModule {
    /// Allocates the module entry on the heap, returning a pointer to the
    /// memory location. The caller is responsible for the memory pointed to.
    #[must_use]
    pub fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }
}
