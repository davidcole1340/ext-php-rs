//! Builder and objects for creating classes in the PHP world.

use crate::{
    errors::{Error, Result},
    php::types::object::{ZendClassObject, ZendObject},
};
use std::{alloc::Layout, convert::TryInto, ffi::CString, fmt::Debug};

use crate::bindings::{
    zend_class_entry, zend_declare_class_constant, zend_declare_property,
    zend_do_implement_interface, zend_register_internal_class_ex,
};

use super::{
    flags::{ClassFlags, MethodFlags, PropertyFlags},
    function::FunctionEntry,
    globals::ExecutorGlobals,
    types::{
        object::RegisteredClass,
        string::ZendString,
        zval::{IntoZval, Zval},
    },
};

/// A Zend class entry. Alias.
pub type ClassEntry = zend_class_entry;

impl PartialEq for ClassEntry {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl ClassEntry {
    /// Attempts to find a reference to a class in the global class table.
    ///
    /// Returns a reference to the class if found, or [`None`] if the class could
    /// not be found or the class table has not been initialized.
    pub fn try_find(name: &str) -> Option<&'static Self> {
        ExecutorGlobals::get().class_table()?;
        let name = ZendString::new(name, false).ok()?;

        unsafe {
            crate::bindings::zend_lookup_class_ex(name.borrow_ptr(), std::ptr::null_mut(), 0)
                .as_ref()
        }
    }

    /// Returns the class flags.
    pub fn flags(&self) -> ClassFlags {
        ClassFlags::from_bits_truncate(self.ce_flags)
    }

    /// Returns `true` if the class entry is an interface, and `false` otherwise.
    pub fn is_interface(&self) -> bool {
        self.flags().contains(ClassFlags::Interface)
    }

    /// Checks if the class is an instance of another class or interface.
    ///
    /// # Parameters
    ///
    /// * `ce` - The inherited class entry to check.
    pub fn instance_of(&self, ce: &ClassEntry) -> bool {
        if self == ce {
            return true;
        }

        if ce.flags().contains(ClassFlags::Interface) {
            let interfaces = match self.interfaces() {
                Some(interfaces) => interfaces,
                None => return false,
            };

            for i in interfaces {
                if ce == i {
                    return true;
                }
            }
        } else {
            loop {
                let parent = match self.parent() {
                    Some(parent) => parent,
                    None => return false,
                };

                if parent == ce {
                    return true;
                }
            }
        }

        false
    }

    /// Returns an iterator of all the interfaces that the class implements. Returns [`None`] if
    /// the interfaces have not been resolved on the class.
    pub fn interfaces(&self) -> Option<impl Iterator<Item = &ClassEntry>> {
        self.flags()
            .contains(ClassFlags::ResolvedInterfaces)
            .then(|| unsafe {
                (0..self.num_interfaces)
                    .into_iter()
                    .map(move |i| *self.__bindgen_anon_3.interfaces.offset(i as _))
                    .filter_map(|ptr| ptr.as_ref())
            })
    }

    /// Returns the parent of the class.
    ///
    /// If the parent of the class has not been resolved, it attempts to find the parent by name.
    /// Returns [`None`] if the parent was not resolved and the parent was not able to be found
    /// by name.
    pub fn parent(&self) -> Option<&Self> {
        if self.flags().contains(ClassFlags::ResolvedParent) {
            unsafe { self.__bindgen_anon_1.parent.as_ref() }
        } else {
            let name =
                unsafe { ZendString::from_ptr(self.__bindgen_anon_1.parent_name, false) }.ok()?;
            Self::try_find(name.as_str()?)
        }
    }
}

/// Builds a class to be exported as a PHP class.
pub struct ClassBuilder<'a> {
    name: String,
    ptr: &'a mut ClassEntry,
    extends: Option<&'static ClassEntry>,
    interfaces: Vec<&'static ClassEntry>,
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
    pub fn new<T: Into<String>>(name: T) -> Self {
        // SAFETY: Allocating temporary class entry. Will return a null-ptr if allocation fails,
        // which will cause the program to panic (standard in Rust). Unwrapping is OK - the ptr
        // will either be valid or null.
        let ptr = unsafe {
            (std::alloc::alloc_zeroed(Layout::new::<ClassEntry>()) as *mut ClassEntry)
                .as_mut()
                .unwrap()
        };

        Self {
            name: name.into(),
            ptr,
            extends: None,
            interfaces: vec![],
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

    /// Implements an interface on the class.
    ///
    /// # Parameters
    ///
    /// * `interface` - Interface to implement on the class.
    ///
    /// # Panics
    ///
    /// Panics when the given class entry `interface` is not an interface.
    pub fn implements(mut self, interface: &'static ClassEntry) -> Self {
        if !interface.is_interface() {
            panic!("Given class entry was not an interface.");
        }

        self.interfaces.push(interface);
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
    ///
    /// # Panics
    ///
    /// Function will panic if the given `default` cannot be converted into a [`Zval`].
    pub fn property<T: Into<String>>(
        mut self,
        name: T,
        default: impl IntoZval,
        flags: PropertyFlags,
    ) -> Self {
        let default = match default.as_zval(true) {
            Ok(default) => default,
            Err(_) => panic!("Invalid default value for property `{}`.", name.into()),
        };

        self.properties.push((name.into(), default, flags));
        self
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
    pub fn constant<T: Into<String>>(mut self, name: T, value: impl IntoZval) -> Result<Self> {
        let value = value.as_zval(true)?;

        self.constants.push((name.into(), value));
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
    /// * `T` - The type which will override the Zend object. Must implement [`RegisteredClass`]
    /// which can be derived using the [`php_class`](crate::php_class) attribute macro.
    pub fn object_override<T: RegisteredClass>(mut self) -> Self {
        unsafe extern "C" fn create_object<T: RegisteredClass>(
            _: *mut ClassEntry,
        ) -> *mut ZendObject {
            let ptr = ZendClassObject::<T>::new_ptr(None);
            &mut (*ptr).std
        }

        self.object_override = Some(create_object::<T>);
        self
    }

    /// Builds the class, returning a reference to the class entry.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] variant if the class could not be registered.
    pub fn build(mut self) -> Result<&'static mut ClassEntry> {
        self.ptr.name = ZendString::new_interned(&self.name)?.release();

        self.methods.push(FunctionEntry::end());
        let func = Box::into_raw(self.methods.into_boxed_slice()) as *const FunctionEntry;
        self.ptr.info.internal.builtin_functions = func;

        let class = unsafe {
            zend_register_internal_class_ex(
                self.ptr,
                match self.extends {
                    Some(ptr) => (ptr as *const _) as *mut _,
                    None => std::ptr::null_mut(),
                },
            )
            .as_mut()
            .ok_or(Error::InvalidPointer)?
        };

        // SAFETY: We allocated memory for this pointer in `new`, so it is our job to free it when the builder has finished.
        unsafe {
            std::alloc::dealloc((self.ptr as *mut _) as *mut u8, Layout::new::<ClassEntry>())
        };

        for iface in self.interfaces {
            unsafe { zend_do_implement_interface(class, std::mem::transmute(iface)) };
        }

        for (name, mut default, flags) in self.properties {
            unsafe {
                zend_declare_property(
                    class,
                    CString::new(name.as_str())?.as_ptr(),
                    name.len() as _,
                    &mut default,
                    flags.bits() as _,
                );
            }
        }

        for (name, value) in self.constants {
            let value = Box::into_raw(Box::new(value));
            unsafe {
                zend_declare_class_constant(
                    class,
                    CString::new(name.as_str())?.as_ptr(),
                    name.len() as u64,
                    value,
                )
            };
        }

        if let Some(object_override) = self.object_override {
            class.__bindgen_anon_2.create_object = Some(object_override);
        }

        Ok(class)
    }
}

impl Debug for ClassEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name: String = unsafe { ZendString::from_ptr(self.name, false) }
            .and_then(|str| str.try_into())
            .map_err(|_| std::fmt::Error)?;

        f.debug_struct("ClassEntry")
            .field("name", &name)
            .field("flags", &self.flags())
            .finish()
    }
}
