#![doc(hidden)]

use ext_php_rs::bindings::{_efree, _emalloc};
use std::alloc::GlobalAlloc;

/// Global allocator which uses the Zend memory management APIs to allocate memory.
///
/// At the moment, this should only be used for debugging memory leaks. You are not supposed to
/// allocate non-request-bound memory using the Zend memory management API.
#[derive(Default)]
pub struct PhpAllocator {}

impl PhpAllocator {
    /// Creates a new PHP allocator.
    pub const fn new() -> Self {
        Self {}
    }
}

unsafe impl GlobalAlloc for PhpAllocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let ptr = _emalloc(
            layout.size() as _,
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            0,
        ) as *mut u8;

        // eprintln!("allocating {} bytes at {:?}", layout.size(), ptr);

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        // eprintln!("deallocating {} bytes at {:?}", layout.size(), ptr);

        _efree(
            ptr as *mut _,
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            0,
        )
    }
}
