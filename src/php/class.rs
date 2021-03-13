use std::mem;

use crate::bindings::{zend_class_entry, zend_register_internal_class};

use super::{
    flags::{ClassFlags, MethodFlags},
    function::FunctionEntry,
    types::string::ZendString,
};

/// A Zend class entry. Alias.
pub type ClassEntry = zend_class_entry;

/// Builds a class to be exported as a PHP class.
pub struct ClassBuilder<'a> {
    ptr: &'a mut ClassEntry,
    functions: Vec<FunctionEntry>,
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
        class
    }
}
