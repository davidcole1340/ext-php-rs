//! Functions for interacting with the execution data passed to PHP functions\
//! introduced in Rust.

use crate::bindings::{zend_execute_data, ZEND_MM_ALIGNMENT, ZEND_MM_ALIGNMENT_MASK};

use super::{
    args::ArgParser,
    types::{
        object::{RegisteredClass, ZendClassObject, ZendObject},
        zval::Zval,
    },
};

/// Execution data passed when a function is called from Zend.
pub type ExecutionData = zend_execute_data;

impl ExecutionData {
    /// Returns an [`ArgParser`] pre-loaded with the arguments contained inside `self`.
    pub fn parser<'a>(&'a mut self) -> ArgParser<'a, '_> {
        self.parser_object().0
    }

    /// Returns an [`ArgParser`] pre-loaded with the arguments contained inside `self`.
    ///
    /// A reference to `$this` is also returned in an [`Option`], which resolves to [`None`]
    /// if this function is not called inside a method.
    pub fn parser_object<'a>(&'a mut self) -> (ArgParser<'a, '_>, Option<&'a mut ZendObject>) {
        // SAFETY: All fields of the `u2` union are the same type.
        let n_args = unsafe { self.This.u2.num_args };
        let mut args = vec![];

        for i in 0..n_args {
            // SAFETY: Function definition ensures arg lifetime doesn't exceed execution data lifetime.
            let arg = unsafe { self.zend_call_arg(i as usize) };
            args.push(arg);
        }

        let obj = self.This.object_mut();

        (ArgParser::new(args), obj)
    }

    /// Returns an [`ArgParser`] pre-loaded with the arguments contained inside `self`.
    ///
    /// A reference to `$this` is also returned in an [`Option`], which resolves to [`None`]
    /// if this function is not called inside a method.
    ///
    /// This function differs from [`parse_object`] in the fact that it returns a reference to
    /// a [`ZendClassObject`], which is an object that contains an arbitrary Rust type at the
    /// start of the object. The object will also resolve to [`None`] if the function is called
    /// inside a method that does not belong to an object with type `T`.
    pub fn parser_method<'a, T: RegisteredClass>(
        &'a mut self,
    ) -> (ArgParser<'a, '_>, Option<&'a mut ZendClassObject<T>>) {
        let (parser, obj) = self.parser_object();
        (
            parser,
            obj.and_then(|obj| ZendClassObject::from_zend_obj_mut(obj)),
        )
    }

    /// Attempts to retrieve a reference to the underlying class object of the Zend object.
    ///
    /// Returns a [`ZendClassObject`] if the execution data contained a valid object of type `T`,
    /// otherwise returns [`None`].
    pub fn get_object<T: RegisteredClass>(&mut self) -> Option<&mut ZendClassObject<T>> {
        // TODO(david): This should be a `&mut self` function but we need to fix arg parser first.
        ZendClassObject::from_zend_obj_mut(self.get_self()?)
    }

    /// Attempts to retrieve the 'this' object, which can be used in class methods
    /// to retrieve the underlying Zend object.
    pub fn get_self(&mut self) -> Option<&mut ZendObject> {
        // TODO(david): This should be a `&mut self` function but we need to fix arg parser first.
        self.This.object_mut()
    }

    /// Translation of macro `ZEND_CALL_ARG(call, n)`
    /// zend_compile.h:578
    ///
    /// The resultant zval reference has a lifetime equal to the lifetime of `self`.
    /// This isn't specified because when you attempt to get a reference to args and
    /// the `$this` object, Rust doesnt't let you. Since this is a private method it's
    /// up to the caller to ensure the lifetime isn't exceeded.
    #[doc(hidden)]
    unsafe fn zend_call_arg<'a>(&self, n: usize) -> Option<&'a mut Zval> {
        let ptr = self.zend_call_var_num(n as isize);
        ptr.as_mut()
    }

    /// Translation of macro `ZEND_CALL_VAR_NUM(call, n)`
    /// zend_compile.h: 575
    #[doc(hidden)]
    unsafe fn zend_call_var_num(&self, n: isize) -> *mut Zval {
        let ptr = self as *const Self as *mut Zval;
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
