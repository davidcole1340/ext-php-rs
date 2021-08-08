//! Builder and objects for creating classes in the PHP world.

use crate::errors::{Error, Result};
use std::{mem, ptr};

use crate::{
    bindings::{
        zend_class_entry, zend_declare_class_constant, zend_declare_property,
        zend_register_internal_class_ex,
    },
    functions::c_str,
};

use super::{
    flags::{ClassFlags, MethodFlags, PropertyFlags},
    function::FunctionEntry,
    types::{
        object::{ZendObject, ZendObjectOverride},
        string::ZendString,
        zval::{IntoZval, Zval},
    },
};

/// A Zend class entry. Alias.
pub type ClassEntry = zend_class_entry;

/// Builds a class to be exported as a PHP class.
pub struct ClassBuilder<'a> {
    name: String,
    ptr: &'a mut ClassEntry,
    extends: Option<&'static ClassEntry>,
    methods: Vec<FunctionEntry>,
    object_override: Option<unsafe extern "C" fn(class_type: *mut ClassEntry) -> *mut ZendObject>,
    properties: Vec<(String, Zval, PropertyFlags)>,
    constants: Vec<(String, Zval)>,
}

impl<'a> ClassBuilder<'a> {
    /// Creates a new class builder, used to build classes
    /// to be exported to PHP.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the class.
    #[allow(clippy::unwrap_used)]
    pub fn new<N>(name: N) -> Self
    where
        N: AsRef<str>,
    {
        // SAFETY: Allocating temporary class entry. Will return a null-ptr if allocation fails,
        // which will cause the program to panic (standard in Rust). Unwrapping is OK - the ptr
        // will either be valid or null.
        let ptr = unsafe {
            (libc::calloc(1, mem::size_of::<ClassEntry>()) as *mut ClassEntry)
                .as_mut()
                .unwrap()
        };

        Self {
            name: name.as_ref().to_string(),
            ptr,
            extends: None,
            methods: vec![],
            object_override: None,
            properties: vec![],
            constants: vec![],
        }
    }

    /// Sets the class builder to extend another class.
    ///
    /// # Parameters
    ///
    /// * `parent` - The parent class to extend.
    pub fn extends(mut self, parent: &'static ClassEntry) -> Self {
        self.extends = Some(parent);
        self
    }

    /// Adds a method to the class.
    ///
    /// # Parameters
    ///
    /// * `func` - The function entry to add to the class.
    /// * `flags` - Flags relating to the function. See [`MethodFlags`].
    pub fn method(mut self, mut func: FunctionEntry, flags: MethodFlags) -> Self {
        func.flags = flags.bits();
        self.methods.push(func);
        self
    }

    /// Adds a property to the class. The initial type of the property is given by the type
    /// of the given default. Note that the user can change the type.
    ///
    /// Returns a result containing the class builder if the property was successfully added.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the property to add to the class.
    /// * `default` - The default value of the property.
    /// * `flags` - Flags relating to the property. See [`PropertyFlags`].
    pub fn property(
        mut self,
        name: impl AsRef<str>,
        default: impl IntoZval,
        flags: PropertyFlags,
    ) -> Result<Self> {
        let default = default.as_zval(true)?;

        self.properties
            .push((name.as_ref().to_string(), default, flags));
        Ok(self)
    }

    /// Adds a constant to the class. The type of the constant is defined by the type of the given
    /// default.
    ///
    /// Returns a result containing the class builder if the constant was successfully added.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the constant to add to the class.
    /// * `value` - The value of the constant.
    pub fn constant(mut self, name: impl AsRef<str>, value: impl IntoZval) -> Result<Self> {
        let value = value.as_zval(true)?;

        self.constants.push((name.as_ref().to_string(), value));
        Ok(self)
    }

    /// Sets the flags for the class.
    ///
    /// # Parameters
    ///
    /// * `flags` - Flags relating to the class. See [`ClassFlags`].
    pub fn flags(mut self, flags: ClassFlags) -> Self {
        self.ptr.ce_flags = flags.bits();
        self
    }

    /// Overrides the creation of the Zend object which will represent an instance
    /// of this class.
    ///
    /// # Parameters
    ///
    /// * `T` - The type which will override the Zend object. Must implement [`ZendObjectOverride`]
    /// which can be derived through the [`ZendObjectHandler`](ext_php_rs_derive::ZendObjectHandler)
    /// derive macro.
    pub fn object_override<T>(mut self) -> Self
    where
        T: ZendObjectOverride,
    {
        self.object_override = Some(T::create_object);
        self
    }

    /// Builds the class, returning a reference to the class entry.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] variant if the class could not be registered.
    pub fn build(mut self) -> Result<&'static mut ClassEntry> {
        self.ptr.name = ZendString::new_interned(self.name)?.release();

        self.methods.push(FunctionEntry::end());
        let func = Box::into_raw(self.methods.into_boxed_slice()) as *const FunctionEntry;
        self.ptr.info.internal.builtin_functions = func;

        let class = unsafe {
            zend_register_internal_class_ex(
                self.ptr,
                match self.extends {
                    Some(ptr) => (ptr as *const _) as *mut _,
                    None => ptr::null_mut(),
                },
            )
            .as_mut()
            .ok_or(Error::InvalidPointer)?
        };

        // SAFETY: We allocated memory for this pointer in `new`, so it is our job to free it when the builder has finished.
        unsafe { libc::free((self.ptr as *mut ClassEntry) as *mut libc::c_void) };

        for (name, mut default, flags) in self.properties {
            unsafe {
                zend_declare_property(
                    class,
                    c_str(&name)?,
                    name.len() as _,
                    &mut default,
                    flags.bits() as _,
                );
            }
        }

        for (name, value) in self.constants {
            let value = Box::into_raw(Box::new(value));
            unsafe { zend_declare_class_constant(class, c_str(&name)?, name.len() as u64, value) };
        }

        if let Some(object_override) = self.object_override {
            class.__bindgen_anon_2.create_object = Some(object_override);
        }

        Ok(class)
    }
}
