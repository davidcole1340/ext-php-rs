//! Functions relating to the Zend Memory Manager, used to allocate
//! request-bound memory.

use cfg_if::cfg_if;

use crate::ffi::{_efree, _emalloc};
use std::{alloc::Layout, ffi::c_void};

/// Uses the PHP memory allocator to allocate request-bound memory.
///
/// # Parameters
///
/// * `layout` - The layout of the requested memory.
///
/// # Returns
///
/// A pointer to the memory allocated.
#[must_use]
pub fn emalloc(layout: Layout) -> *mut u8 {
    // TODO account for alignment
    let size = layout.size();

    (unsafe {
        cfg_if! {
            if #[cfg(php_debug)] {
                #[allow(clippy::used_underscore_items)]
                _emalloc(size as _, std::ptr::null_mut(), 0, std::ptr::null_mut(), 0)
            } else {
                #[allow(clippy::used_underscore_items)]
                _emalloc(size as _)
            }
        }
    })
    .cast::<u8>()
}

/// Frees a given memory pointer which was allocated through the PHP memory
/// manager.
///
/// # Parameters
///
/// * `ptr` - The pointer to the memory to free.
///
/// # Safety
///
/// Caller must guarantee that the given pointer is valid (aligned and non-null)
/// and was originally allocated through the Zend memory manager.
pub unsafe fn efree(ptr: *mut u8) {
    cfg_if! {
        if #[cfg(php_debug)] {
            #[allow(clippy::used_underscore_items)]
            _efree(
                ptr.cast::<c_void>(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                0,
            )
        } else {
            #[allow(clippy::used_underscore_items)]
            _efree(ptr.cast::<c_void>());
        }
    }
}
