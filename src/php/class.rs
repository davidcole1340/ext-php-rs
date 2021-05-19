//! Builder and objects for creating classes in the PHP world.

use std::{mem, ptr};

use crate::{
    bindings::{
        ext_php_rs_zend_string_release, zend_class_entry, zend_declare_class_constant,
        zend_declare_property, zend_register_internal_class_ex,
    },
    functions::c_str,
};

use super::{
    flags::{ClassFlags, MethodFlags, PropertyFlags},
    function::FunctionEntry,
    types::{
        object::{ZendObject, ZendObjectOverride},
        string::ZendString,
        zval::Zval,
    },
};

/// A Zend class entry. Alias.
pub type ClassEntry = zend_class_entry;

/// Builds a class to be exported as a PHP class.
pub struct ClassBuilder<'a> {
    ptr: &'a mut ClassEntry,
    extends: *mut ClassEntry,
    methods: Vec<FunctionEntry>,
    object_override: Option<unsafe extern "C" fn(class_type: *mut ClassEntry) -> *mut ZendObject>,
    properties: Vec<(&'a str, Zval, PropertyFlags)>,
    constants: Vec<(&'a str, Zval)>,
}

impl<'a> ClassBuilder<'a> {
    /// Creates a new class builder, used to build classes
    /// to be exported to PHP.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the class.
    pub fn new<N>(name: N) -> Self
    where
        N: AsRef<str>,
    {
        let ptr = unsafe {
            (libc::calloc(1, mem::size_of::<ClassEntry>()) as *mut ClassEntry)
                .as_mut()
                .unwrap()
        };
        ptr.name = ZendString::new_interned(name);

        Self {
            ptr,
            extends: ptr::null_mut(),
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
    pub fn extends(mut self, parent: *mut ClassEntry) -> Self {
        self.extends = parent;
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
    /// # Parameters
    ///
    /// * `name` - The name of the property to add to the class.
    /// * `default` - The default value of the property.
    /// * `flags` - Flags relating to the property. See [`PropertyFlags`].
    #[allow(unused_mut, unused_variables)]
    pub fn property<T>(mut self, name: &'a str, default: T, flags: PropertyFlags) -> Self
    where
        T: Into<Zval>,
    {
        let mut default = default.into();

        if default.is_string() {
            let val = default.string().unwrap();
            unsafe { ext_php_rs_zend_string_release(default.value.str_) };
            default.set_persistent_string(val);
        }

        self.properties.push((name, default, flags));
        self
    }

    /// Adds a constant to the class.
    /// The type of the constant is defined by the type of the given default.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the constant to add to the class.
    /// * `value` - The value of the constant.
    pub fn constant<T>(mut self, name: &'a str, value: T) -> Self
    where
        T: Into<Zval>,
    {
        let mut value = value.into();

        if value.is_string() {
            let val = value.string().unwrap();
            unsafe { ext_php_rs_zend_string_release(value.value.str_) };
            value.set_persistent_string(val);
        }

        self.constants.push((name, value));
        self
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

    /// Builds the class, returning a pointer to the class entry.
    pub fn build(mut self) -> *mut ClassEntry {
        self.methods.push(FunctionEntry::end());
        let func = Box::into_raw(self.methods.into_boxed_slice()) as *const FunctionEntry;
        self.ptr.info.internal.builtin_functions = func;

        let class = unsafe {
            zend_register_internal_class_ex(self.ptr, self.extends)
                .as_mut()
                .unwrap()
        };

        unsafe { libc::free((self.ptr as *mut ClassEntry) as *mut libc::c_void) };

        for (name, mut default, flags) in self.properties {
            unsafe {
                zend_declare_property(
                    class,
                    c_str(name),
                    name.len() as _,
                    &mut default,
                    flags.bits() as _,
                );
            }
        }

        for (name, value) in self.constants {
            let value = Box::into_raw(Box::new(value));
            unsafe { zend_declare_class_constant(class, c_str(name), name.len() as u64, value) };
        }

        if let Some(object_override) = self.object_override {
            class.__bindgen_anon_2.create_object = Some(object_override);
        }

        class
    }
}
