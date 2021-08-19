//! Types related to callables in PHP (anonymous functions, functions, etc).

use std::ops::Deref;

use super::zval::{IntoZval, Zval};
use crate::{
    bindings::_call_user_function_impl,
    errors::{Error, Result},
};

/// Acts as a wrapper around a callable [`Zval`]. Allows the owner to call the [`Zval`] as if it
/// was a PHP function through the [`try_call`](Callable::try_call) method.
#[derive(Debug)]
pub struct Callable<'a>(OwnedZval<'a>);

/// A container for a zval. Either contains a reference to a zval or an owned zval.
#[derive(Debug)]
enum OwnedZval<'a> {
    Reference(&'a Zval),
    Owned(Zval),
}

impl<'a> OwnedZval<'a> {
    fn as_ref(&self) -> &Zval {
        match self {
            OwnedZval::Reference(zv) => *zv,
            OwnedZval::Owned(zv) => zv,
        }
    }
}

impl<'a> Deref for OwnedZval<'a> {
    type Target = Zval;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> Callable<'a> {
    /// Attempts to create a new [`Callable`]. Should not need to be used directly, but through the
    /// [`Zval::callable`] method.
    ///
    /// # Parameters
    ///
    /// * `callable` - The underlying [`Zval`] that is callable.
    ///
    /// # Errors
    ///
    /// Returns an error if the [`Zval`] was not callable.
    pub fn new(callable: &'a Zval) -> Result<Self> {
        if callable.is_callable() {
            Ok(Self(OwnedZval::Reference(callable)))
        } else {
            Err(Error::Callable)
        }
    }

    /// Attempts to create a new [`Callable`] by taking ownership of a Zval. Returns a result
    /// containing the callable if the zval was callable.
    ///
    /// # Parameters
    ///
    /// * `callable` - TThe underlying [`Zval`] that is callable.
    pub fn new_owned(callable: Zval) -> Result<Self> {
        if callable.is_callable() {
            Ok(Self(OwnedZval::Owned(callable)))
        } else {
            Err(Error::Callable)
        }
    }

    /// Attempts to create a new [`Callable`] from a function name. Returns a result containing the
    /// callable if the function existed and was callable.
    ///
    /// # Parameters
    ///
    /// * `name` - Name of the callable function.
    pub fn try_from_name(name: &str) -> Result<Self> {
        let mut callable = Zval::new();
        callable.set_string(name, false)?;

        Self::new_owned(callable)
    }

    /// Attempts to call the callable with a list of arguments to pass to the function.
    /// Note that a thrown exception inside the callable is not detectable, therefore you should
    /// check if the return value is valid rather than unwrapping. Returns a result containing the
    /// return value of the function, or an error.
    ///
    /// You should not call this function directly, rather through the [`call_user_func`] macro.
    ///
    /// # Parameters
    ///
    /// * `params` - A list of parameters to call the function with.
    pub fn try_call(&self, params: Vec<&dyn IntoZval>) -> Result<Zval> {
        if !self.0.is_callable() {
            return Err(Error::Callable);
        }

        let mut retval = Zval::new();
        let len = params.len();
        let params = params
            .into_iter()
            .map(|val| val.as_zval(false))
            .collect::<Result<Vec<_>>>()?;
        let packed = params.into_boxed_slice();

        let result = unsafe {
            _call_user_function_impl(
                std::ptr::null_mut(),
                std::mem::transmute(self.0.as_ref()),
                &mut retval,
                len as _,
                packed.as_ptr() as *mut _,
                std::ptr::null_mut(),
            )
        };

        if result < 0 {
            Err(Error::Callable)
        } else {
            Ok(retval)
        }
    }
}
