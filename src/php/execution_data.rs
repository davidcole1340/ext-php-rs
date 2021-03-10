use std::{convert::TryFrom, mem};

use crate::bindings::{zend_execute_data, ZEND_MM_ALIGNMENT, ZEND_MM_ALIGNMENT_MASK};

use super::zval::Zval;

/// Execution data passed when a function is called from Zend.
pub type ExecutionData = zend_execute_data;

impl ExecutionData {
    /// Retrieves an argument from the execution data at a given offset.
    /// Offsets start at zero. Make sure to never attempt to retrieve an
    /// argument that may not exist (greater offset than arg_len - 1).
    ///
    /// Marked as unsafe.
    ///
    /// # Parameters
    ///
    /// * `offset` - The offset of the argument to read, where the first
    /// argument is at offset 0.
    ///
    /// # Generics
    ///
    /// * `T` - The type to attempt to retrieve the argument as.
    ///
    /// # Returns
    ///
    /// * `Some()` - The argument was successfully read and parsed.
    /// * `None` - The argument was not present or the type of the
    /// argument was wrong.
    pub(crate) unsafe fn get_arg<T>(&self, offset: usize) -> Option<T>
    where
        T: TryFrom<&'static Zval>,
    {
        match self.zend_call_arg(offset) {
            Some(zval) => match T::try_from(zval) {
                Ok(res) => Some(res),
                Err(_) => None,
            },
            None => None,
        }
    }

    /// Translation of macro `ZEND_CALL_ARG(call, n)`
    /// zend_compile.h:578
    #[doc(hidden)]
    unsafe fn zend_call_arg(&self, n: usize) -> Option<&'static Zval> {
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
        let size = mem::size_of::<T>();
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
