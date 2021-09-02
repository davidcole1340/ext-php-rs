#![doc(hidden)]

use ext_php_rs::php::alloc;
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
        let ptr = alloc::emalloc(layout);
        eprintln!("allocating {:?}: {} bytes", ptr, layout.size());
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        eprintln!("freeing {:?}: {} bytes", ptr, layout.size());
        alloc::efree(ptr)
    }
}
