//! Represents an object in PHP. Allows for overriding the internal object used by classes,
//! allowing users to store Rust data inside a PHP object.

use std::{alloc::GlobalAlloc, intrinsics::offset, ops::Deref};

use crate::{
    bindings::{php_rs_zend_object_alloc, zend_object, zend_object_std_init},
    php::class::ClassEntry,
};

/// A Zend class object which is allocated when a PHP
/// class object is instantiated. Overrides the default
/// handler when the user provides a type T of the struct
/// they want to override with.
#[repr(C)]
pub struct ZendClassObject<T: Default> {
    obj: T,
    std: *mut zend_object,
}

impl<T: Default> ZendClassObject<T> {
    /// Allocates a new object when an instance of the class is created
    /// in the PHP world.
    ///
    /// This function should not be called directly, but rather passed
    /// to PHP through the `create_object` parameter on a [`ClassEntry`].
    ///
    /// # Parameters
    ///
    /// * `ce` - The class entry that was created.
    #[no_mangle]
    pub extern "C" fn new(ce: *mut ClassEntry) -> *mut zend_object {
        // SAFETY: We allocate the memory required for the object through the Zend memory manager, therefore
        // we own the memory at this point in time.
        let obj = unsafe {
            let obj = (php_rs_zend_object_alloc(std::mem::size_of::<Self>() as u64, ce)
                as *mut Self)
                .as_mut()
                .unwrap();

            zend_object_std_init(obj.std, ce);
            obj
        };

        obj.obj = T::default();
        // TODO ojb->std.handlers = &object_handlers
        obj.std
    }
}

impl<T: Default> Deref for ZendClassObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}
