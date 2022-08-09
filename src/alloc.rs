//! Functions relating to the Zend Memory Manager, used to allocate
//! request-bound memory.

use crate::ffi::{ext_php_rs_efree as _efree, ext_php_rs_emalloc as _emalloc};
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
pub fn emalloc(layout: Layout) -> *mut u8 {
    // TODO account for alignment
    let size = layout.size();

    unsafe {
        #[cfg(php_debug)]
        {
            _(emalloc(
                size as _,
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                0,
            )) as *mut u8
        }
        #[cfg(not(php_debug))]
        {
            let ptr = _emalloc(size as _);

            print!("{:?}", *ptr);

            if *ptr == 0 {
                panic!()
            }

            ptr as *mut u8
        }
    }
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
    #[cfg(php_debug)]
    {
        _efree(
            ptr as *mut c_void,
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            0,
        )
    }
    #[cfg(not(php_debug))]
    {
        let ptr = _efree(ptr as *mut c_void);

        if *ptr == 0 {
            panic!();
        }
    }
}
