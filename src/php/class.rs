use std::mem;

use crate::bindings::{
    zend_class_entry, zend_declare_class_constant_ex, zend_declare_property_ex,
    zend_register_internal_class,
};

use super::{
    flags::{ClassFlags, ConstantFlags, MethodFlags, PropertyFlags},
    function::FunctionEntry,
    types::{string::ZendString, zval::Zval},
};

/// A Zend class entry. Alias.
pub type ClassEntry = zend_class_entry;

/// Builds a class to be exported as a PHP class.
pub struct ClassBuilder<'a> {
    ptr: &'a mut ClassEntry,
    functions: Vec<FunctionEntry>,
    properties: Vec<(&'a str, &'a str, Zval, PropertyFlags)>,
    constants: Vec<(&'a str, &'a str, Zval, ConstantFlags)>,
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
        let ptr = unsafe { libc::malloc(mem::size_of::<ClassEntry>()) } as *mut ClassEntry;
        let self_ = Self {
            ptr: unsafe { ptr.as_mut() }.unwrap(),
            functions: vec![],
            properties: vec![],
            constants: vec![],
        };
        self_.ptr.name = ZendString::new_interned(name, true);
        self_
    }

    /// Adds a method to the class.
    ///
    /// # Parameters
    ///
    /// * `func` - The function entry to add to the class.
    /// * `flags` - Flags relating to the function. See [`MethodFlags`].
    pub fn function(mut self, mut func: FunctionEntry, flags: MethodFlags) -> Self {
        func.flags = flags.bits();
        self.functions.push(func);
        self
    }

    /// Adds a property to the class.
    /// The type of the property is defined by the type of the given default.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the property to add to the class.
    /// * `doc` - Documentation comment for the property.
    /// * `default` - The default value of the property.
    /// * `flags` - Flags relating to the property. See [`PropertyFlags`].
    pub fn property<T>(
        mut self,
        name: &'a str,
        doc: &'a str,
        default: T,
        flags: PropertyFlags,
    ) -> Self
    where
        T: Into<Zval>,
    {
        self.properties.push((name, doc, default.into(), flags));
        self
    }

    /// Adds a constant to the class.
    /// The type of the constant is defined by the type of the given default.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the constant to add to the class.
    /// * `doc` - Documentation comment for the constant.
    /// * `value` - The value of the constant.
    /// * `flags` - Flags relating to the constant. See [`ConstantFlags`].
    pub fn constant<T>(
        mut self,
        name: &'a str,
        doc: &'a str,
        value: T,
        flags: ConstantFlags,
    ) -> Self
    where
        T: Into<Zval>,
    {
        self.constants.push((name, doc, value.into(), flags));
        self
    }

    /// Sets the flags for the class.
    ///
    /// # Parameters
    ///
    /// * `flags` - Flags relating to the class. See [`ClassFlags`].
    pub fn flags(self, flags: ClassFlags) -> Self {
        self.ptr.ce_flags = flags.bits();
        self
    }

    /// Builds the class, returning a pointer to the class entry.
    pub fn build(mut self) -> *mut ClassEntry {
        self.functions.push(FunctionEntry::end());
        let func = Box::into_raw(self.functions.into_boxed_slice()) as *const FunctionEntry;
        self.ptr.info.internal.builtin_functions = func;

        let class = unsafe { zend_register_internal_class(self.ptr) };
        unsafe { libc::free((self.ptr as *mut ClassEntry) as *mut libc::c_void) };

        for (name, doc, default, flags) in self.properties {
            let name = ZendString::new_interned(name, true);
            let doc = ZendString::new_interned(doc, true);
            let default = Box::into_raw(Box::new(default));
            unsafe { zend_declare_property_ex(class, name, default, flags.bits() as i32, doc) };
        }

        for (name, doc, value, flags) in self.constants {
            let name = ZendString::new_interned(name, true);
            let doc = ZendString::new_interned(doc, true);
            let value = Box::into_raw(Box::new(value));
            unsafe { zend_declare_class_constant_ex(class, name, value, flags.bits() as i32, doc) };
        }

        class
    }
}
