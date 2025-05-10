//! Utility functions for testing
#![allow(clippy::must_use_candidate)]
use crate::{ffi::_zend_execute_data, types::Zval, zend::ModuleEntry};

/// Dummy function for testing
#[cfg(not(windows))]
pub extern "C" fn test_function(_: &mut _zend_execute_data, _: &mut Zval) {
    // Dummy function for testing
}

/// Dummy function for testing on windows
#[cfg(windows)]
pub extern "vectorcall" fn test_function(_: &mut _zend_execute_data, _: &mut Zval) {
    // Dummy function for testing
}

/// Dummy function for testing
pub extern "C" fn test_startup_shutdown_function(_type: i32, _module_number: i32) -> i32 {
    // Dummy function for testing
    0
}

/// Dummy function for testing
pub extern "C" fn test_info_function(_zend_module: *mut ModuleEntry) {
    // Dummy function for testing
}

/// Dummy function for testing
pub extern "C" fn test_deactivate_function() -> i32 {
    // Dummy function for testing
    0
}
