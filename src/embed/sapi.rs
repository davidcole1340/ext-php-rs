//! Builder and objects for creating modules in PHP. A module is the base of a
//! PHP extension.

use crate::ffi::{sapi_module, sapi_module_struct};

/// A Zend module entry, also known as an extension.
pub type SapiModule = sapi_module_struct;

pub fn set_global_module(module: SapiModule) {
    // leak the module to the global scope
    let module = Box::leak(Box::new(module));

    unsafe {
        sapi_module = *module;
    }
}

pub fn get_global_module() -> Option<&'static SapiModule> {
    let module = unsafe { &*&raw const sapi_module };

    if module.name.is_null() {
        return None;
    }

    Some(module)
}

impl SapiModule {
    /// Allocates the module entry on the heap, returning a pointer to the
    /// memory location. The caller is responsible for the memory pointed to.
    pub fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }

    pub fn name(&self) -> &str {
        unsafe { std::ffi::CStr::from_ptr(self.name).to_str().unwrap() }
    }
}

#[cfg(test)]
mod tests {
    use crate::embed::Embed;
    use super::*;

    #[test]
    fn test_get_global_module() {
        Embed::run(|| {
            let module = get_global_module();

            assert!(module.is_some());
            let module = module.unwrap();

            assert_eq!(module.name(), "embed");
        })
    }
}
