//! Builder and objects for creating classes in the PHP world.

use crate::ffi::instanceof_function_slow;
use crate::types::{ZendIterator, Zval};
use crate::{
    boxed::ZBox,
    ffi::zend_class_entry,
    flags::ClassFlags,
    types::{ZendObject, ZendStr},
    zend::ExecutorGlobals,
};
use std::{convert::TryInto, fmt::Debug, ops::DerefMut};

/// A PHP class entry.
///
/// Represents a class registered with the PHP interpreter.
pub type ClassEntry = zend_class_entry;

impl ClassEntry {
    /// Attempts to find a reference to a class in the global class table.
    ///
    /// Returns a reference to the class if found, or [`None`] if the class
    /// could not be found or the class table has not been initialized.
    pub fn try_find(name: &str) -> Option<&'static Self> {
        ExecutorGlobals::get().class_table()?;
        let mut name = ZendStr::new(name, false);

        unsafe {
            crate::ffi::zend_lookup_class_ex(name.deref_mut(), std::ptr::null_mut(), 0).as_ref()
        }
    }

    /// Creates a new [`ZendObject`], returned inside an [`ZBox<ZendObject>`]
    /// wrapper.
    ///
    /// # Panics
    ///
    /// Panics when allocating memory for the new object fails.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(&self) -> ZBox<ZendObject> {
        ZendObject::new(self)
    }

    /// Returns the class flags.
    pub fn flags(&self) -> ClassFlags {
        ClassFlags::from_bits_truncate(self.ce_flags)
    }

    /// Returns `true` if the class entry is an interface, and `false`
    /// otherwise.
    pub fn is_interface(&self) -> bool {
        self.flags().contains(ClassFlags::Interface)
    }

    /// Checks if the class is an instance of another class or interface.
    ///
    /// # Parameters
    ///
    /// * `other` - The inherited class entry to check.
    pub fn instance_of(&self, other: &ClassEntry) -> bool {
        if self == other {
            return true;
        }

        unsafe { instanceof_function_slow(self as _, other as _) }
    }

    /// Returns an iterator of all the interfaces that the class implements.
    ///
    /// Returns [`None`] if the interfaces have not been resolved on the
    /// class.
    pub fn interfaces(&self) -> Option<impl Iterator<Item = &ClassEntry>> {
        self.flags()
            .contains(ClassFlags::ResolvedInterfaces)
            .then(|| unsafe {
                (0..self.num_interfaces)
                    .map(move |i| *self.__bindgen_anon_3.interfaces.offset(i as _))
                    .filter_map(|ptr| ptr.as_ref())
            })
    }

    /// Returns the parent of the class.
    ///
    /// If the parent of the class has not been resolved, it attempts to find
    /// the parent by name. Returns [`None`] if the parent was not resolved
    /// and the parent was not able to be found by name.
    pub fn parent(&self) -> Option<&Self> {
        if self.flags().contains(ClassFlags::ResolvedParent) {
            unsafe { self.__bindgen_anon_1.parent.as_ref() }
        } else {
            let name = unsafe { self.__bindgen_anon_1.parent_name.as_ref()? };
            Self::try_find(name.as_str().ok()?)
        }
    }

    /// Returns the iterator for the class for a specific instance
    ///
    /// Returns [`None`] if there is no associated iterator for the class.
    pub fn get_iterator<'a>(&self, zval: &'a Zval, by_ref: bool) -> Option<&'a mut ZendIterator> {
        let ptr: *const Self = self;
        let zval_ptr: *const Zval = zval;

        let iterator = unsafe {
            (*ptr).get_iterator?(
                ptr as *mut ClassEntry,
                zval_ptr as *mut Zval,
                if by_ref { 1 } else { 0 },
            )
        };

        unsafe { iterator.as_mut() }
    }

    /// Gets the name of the class.
    pub fn name(&self) -> Option<&str> {
        unsafe { self.name.as_ref().and_then(|s| s.as_str().ok()) }
    }
}

impl PartialEq for ClassEntry {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Debug for ClassEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name: String = unsafe { self.name.as_ref() }
            .and_then(|s| s.try_into().ok())
            .ok_or(std::fmt::Error)?;

        f.debug_struct("ClassEntry")
            .field("name", &name)
            .field("flags", &self.flags())
            .field("is_interface", &self.is_interface())
            .field(
                "interfaces",
                &self.interfaces().map(|iter| iter.collect::<Vec<_>>()),
            )
            .field("parent", &self.parent())
            .finish()
    }
}
