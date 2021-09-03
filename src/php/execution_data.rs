//! Functions for interacting with the execution data passed to PHP functions\
//! introduced in Rust.

use std::mem;

use crate::{
    bindings::{zend_execute_data, ZEND_MM_ALIGNMENT, ZEND_MM_ALIGNMENT_MASK},
    errors::{Error, Result},
};

use super::types::{
    object::{ClassObject, RegisteredClass, ZendClassObject, ZendObject},
    zval::Zval,
};

/// Execution data passed when a function is called from Zend.
pub type ExecutionData = zend_execute_data;

impl ExecutionData {
    /// Attempts to retrieve a reference to the underlying class object of the Zend object.
    ///
    /// Returns a [`ClassObject`] if the execution data contained a valid object, otherwise
    /// returns [`None`].
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the function is called on an instance of [`ExecutionData`]
    /// that:
    ///
    /// 1. Contains an object.
    /// 2. The object was originally derived from `T`.
    pub unsafe fn get_object<T: RegisteredClass>(&self) -> Option<ClassObject<T>> {
        let ptr = self.This.object()? as *const ZendObject as *mut u8;
        let offset = mem::size_of::<T>();
        let ptr = ptr.offset(0 - offset as isize) as *mut ZendClassObject<T>;
        Some(ClassObject::from_zend_class_object(&mut *ptr, false))
    }

    /// Attempts to retrieve the 'this' object, which can be used in class methods
    /// to retrieve the underlying Zend object.
    pub fn get_self(&self) -> Result<&mut ZendObject> {
        unsafe { self.This.value.obj.as_mut() }.ok_or(Error::InvalidScope)
    }

    /// Translation of macro `ZEND_CALL_ARG(call, n)`
    /// zend_compile.h:578
    #[doc(hidden)]
    pub(crate) unsafe fn zend_call_arg(&self, n: usize) -> Option<&'static Zval> {
        let ptr = self.zend_call_var_num(n as isize);
        ptr.as_ref()
    }

    /// Translation of macro `ZEND_CALL_VAR_NUM(call, n)`
    /// zend_compile.h: 575
    #[doc(hidden)]
    unsafe fn zend_call_var_num(&self, n: isize) -> *const Zval {
        let ptr = self as *const Self as *const Zval;
        ptr.offset(Self::zend_call_frame_slot() + n as isize)
    }

    /// Translation of macro `ZEND_CALL_FRAME_SLOT`
    /// zend_compile:573
    #[doc(hidden)]
    fn zend_call_frame_slot() -> isize {
        (Self::zend_mm_aligned_size::<Self>() + Self::zend_mm_aligned_size::<Zval>() - 1)
            / Self::zend_mm_aligned_size::<Zval>()
    }

    /// Translation of macro `ZEND_MM_ALIGNED_SIZE(size)`
    /// zend_alloc.h:41
    #[doc(hidden)]
    fn zend_mm_aligned_size<T>() -> isize {
        let size = std::mem::size_of::<T>();
        ((size as isize) + ZEND_MM_ALIGNMENT as isize - 1) & ZEND_MM_ALIGNMENT_MASK as isize
    }
}

#[cfg(test)]
mod tests {
    use super::ExecutionData;

    #[test]
    fn test_zend_call_frame_slot() {
        // PHP 8.0.2 (cli) (built: Feb 21 2021 11:51:33) ( NTS )
        // Copyright (c) The PHP Group
        // Zend Engine v4.0.2, Copyright (c) Zend Technologies
        assert_eq!(ExecutionData::zend_call_frame_slot(), 5);
    }
}
