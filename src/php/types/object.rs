//! Represents an object in PHP. Allows for overriding the internal object used by classes,
//! allowing users to store Rust data inside a PHP object.

use std::{
    convert::TryInto,
    fmt::Debug,
    mem,
    ops::{Deref, DerefMut},
};

use crate::{
    bindings::{
        ext_php_rs_zend_object_alloc, object_properties_init, std_object_handlers, zend_object,
        zend_object_handlers, zend_object_std_init, ZEND_ISEMPTY, ZEND_PROPERTY_EXISTS,
        ZEND_PROPERTY_ISSET,
    },
    errors::{Error, Result},
    php::{class::ClassEntry, execution_data::ExecutionData, types::string::ZendString},
};

use super::{array::ZendHashTable, zval::Zval};

pub type ZendObject = zend_object;
pub type ZendObjectHandlers = zend_object_handlers;

/// Different ways to query if a property exists.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PropertyQuery {
    /// Property exists and is not NULL.
    Isset = ZEND_PROPERTY_ISSET,
    /// Property is not empty.
    NotEmpty = ZEND_ISEMPTY,
    /// Property exists.
    Exists = ZEND_PROPERTY_EXISTS,
}

impl ZendObject {
    /// Attempts to retrieve the class name of the object.
    pub fn get_class_name(&self) -> Result<String> {
        let name = unsafe {
            ZendString::from_ptr(
                self.handlers()?.get_class_name.ok_or(Error::InvalidScope)?(self),
                false,
            )
        }?;

        name.try_into()
    }

    /// Attempts to read a property from the Object. Returns a result returning an
    /// immutable reference to the [`Zval`] if the property exists and can be read,
    /// and an [`Error`] otherwise.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the property.
    /// * `query` - The type of query to use when attempting to get a property.
    pub fn get_property(&self, name: impl AsRef<str>) -> Result<&Zval> {
        let name = name.as_ref();

        if !self.has_property(name, PropertyQuery::Exists)? {
            return Err(Error::InvalidProperty);
        }

        let name = ZendString::new(name, false);
        let mut rv = Zval::new();

        unsafe {
            self.handlers()?.read_property.ok_or(Error::InvalidScope)?(
                self.mut_ptr(),
                name.borrow_ptr(),
                1,
                std::ptr::null_mut(),
                &mut rv,
            )
            .as_ref()
        }
        .ok_or(Error::InvalidScope)
    }

    /// Attempts to set a property on the object, returning an immutable reference to
    /// the [`Zval`] if the property can be set.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the property.
    /// * `value` - The value to set the property to.
    pub fn set_property(&mut self, name: impl AsRef<str>, value: impl Into<Zval>) -> Result<&Zval> {
        let name = ZendString::new(name, false);
        let mut value = value.into();

        if value.is_string() {
            value.set_refcount(0);
        }

        unsafe {
            self.handlers()?.write_property.ok_or(Error::InvalidScope)?(
                self,
                name.borrow_ptr(),
                &mut value,
                std::ptr::null_mut(),
            )
            .as_ref()
        }
        .ok_or(Error::InvalidScope)
    }

    /// Checks if a property exists on an object. Takes a property name and query parameter,
    /// which defines what classifies if a property exists or not. See [`PropertyQuery`] for
    /// more information.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the property.
    /// * `query` - The 'query' to classify if a property exists.
    pub fn has_property(&self, name: impl AsRef<str>, query: PropertyQuery) -> Result<bool> {
        let name = ZendString::new(name.as_ref(), false);

        Ok(unsafe {
            self.handlers()?.has_property.ok_or(Error::InvalidScope)?(
                self.mut_ptr(),
                name.borrow_ptr(),
                query as _,
                std::ptr::null_mut(),
            )
        } > 0)
    }

    /// Attempts to retrieve the properties of the object. Returned inside a Zend Hashtable.
    pub fn get_properties(&self) -> Result<ZendHashTable> {
        unsafe {
            ZendHashTable::from_ptr(
                self.handlers()?.get_properties.ok_or(Error::InvalidScope)?(self.mut_ptr()),
                false,
            )
        }
    }

    /// Attempts to retrieve a reference to the object handlers.
    #[inline]
    unsafe fn handlers(&self) -> Result<&ZendObjectHandlers> {
        self.handlers.as_ref().ok_or(Error::InvalidScope)
    }

    /// Returns a mutable pointer to `self`, regardless of the type of reference.
    /// Only to be used in situations where a C function requires a mutable pointer
    /// but does not modify the underlying data.
    #[inline]
    fn mut_ptr(&self) -> *mut Self {
        (self as *const Self) as *mut Self
    }
}

impl Debug for ZendObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_struct(
            self.get_class_name()
                .unwrap_or_else(|_| "ZendObject".to_string())
                .as_str(),
        );

        if let Ok(props) = self.get_properties() {
            for (id, key, val) in props.into_iter() {
                dbg.field(key.unwrap_or_else(|| id.to_string()).as_str(), val);
            }
        }

        dbg.finish()
    }
}

/// Implemented by the [`ZendObjectHandler`](ext_php_rs_derive::ZendObjectHandler) macro on a type T
/// which is used as the T type for [`ZendClassObject`].
/// Implements a function `create_object` which is passed to a PHP class entry to instantiate the
/// object that will represent an object.
pub trait ZendObjectOverride {
    /// Creates a new Zend object. Also allocates space for type T on which the trait is
    /// implemented on.
    ///
    /// # Parameters
    ///
    /// * `ce` - The class entry that we are creating an object for.
    extern "C" fn create_object(ce: *mut ClassEntry) -> *mut ZendObject;
}

/// A Zend class object which is allocated when a PHP
/// class object is instantiated. Overrides the default
/// handler when the user provides a type T of the struct
/// they want to override with.
#[repr(C)]
pub struct ZendClassObject<T: Default> {
    obj: T,
    std: zend_object,
}

impl<T: Default> ZendClassObject<T> {
    /// Allocates a new object when an instance of the class is created in the PHP world.
    ///
    /// Internal function. The end user functions are generated by the
    /// [`ZendObjectHandler`](ext_php_rs_derive::ZendObjectHandler) derive macro which generates a
    /// function that wraps this function to be exported to C.
    ///
    /// # Parameters
    ///
    /// * `ce` - The class entry that was created.
    /// * `handlers` - A pointer to the object handlers for the class.
    ///
    /// # Safety
    ///
    /// This function is an internal function which is only called from code which is derived using
    /// the [`ZendObjectHandler`](ext_php_rs_derive::ZendObjectHandler) derive macro. PHP will
    /// guarantee that any pointers given to this function will be valid, therefore we can Unwrap
    /// them with safety.
    pub unsafe fn new_ptr(
        ce: *mut ClassEntry,
        handlers: *mut ZendObjectHandlers,
    ) -> *mut zend_object {
        let obj = {
            let obj = (ext_php_rs_zend_object_alloc(std::mem::size_of::<Self>() as _, ce)
                as *mut Self)
                .as_mut()
                .unwrap();

            zend_object_std_init(&mut obj.std, ce);
            object_properties_init(&mut obj.std, ce);
            obj
        };

        obj.obj = T::default();
        obj.std.handlers = handlers;
        &mut obj.std
    }

    /// Attempts to retrieve the Zend class object container from the
    /// zend object contained in the execution data of a function.
    ///
    /// # Parameters
    ///
    /// * `ex` - The execution data of the function.
    pub fn get(ex: &ExecutionData) -> Option<&'static mut Self> {
        // cast to u8 to work in terms of bytes
        let ptr = (ex.This.object()? as *const ZendObject) as *mut u8;
        let offset = std::mem::size_of::<T>();
        unsafe {
            let ptr = ptr.offset(0 - offset as isize);
            (ptr as *mut Self).as_mut()
        }
    }
}

impl<T: Default + Debug> Debug for ZendClassObject<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.obj.fmt(f)
    }
}

impl<T: Default> Deref for ZendClassObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

impl<T: Default> DerefMut for ZendClassObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.obj
    }
}

impl ZendObjectHandlers {
    /// Creates a new set of object handlers from the standard object handlers, returning a pointer
    /// to the handlers.
    pub fn init<T>() -> *mut ZendObjectHandlers {
        // SAFETY: We are allocating memory for the handlers ourselves, which ensures that
        // we can copy to the allocated memory. We can also copy from the standard handlers
        // as the `std_object_handlers` are not modified.
        unsafe {
            let s = mem::size_of::<Self>();
            let ptr = libc::malloc(s) as *mut Self;
            libc::memcpy(
                ptr as *mut _,
                (&std_object_handlers as *const Self) as *mut _,
                s,
            );
            let offset = mem::size_of::<T>();
            (*ptr).offset = offset as i32;
            ptr
        }
    }
}
