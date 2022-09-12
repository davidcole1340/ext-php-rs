//! Types related to callables in PHP (anonymous functions, functions, etc).

use std::{convert::TryFrom, ops::Deref};

use crate::{
    convert::{FromZval, IntoZvalDyn},
    error::{Error, Result},
    ffi::_call_user_function_impl,
    flags::DataType,
    zend::ExecutorGlobals,
};

use super::Zval;

/// Acts as a wrapper around a callable [`Zval`]. Allows the owner to call the
/// [`Zval`] as if it was a PHP function through the [`try_call`] method.
///
/// [`try_call`]: #method.try_call
#[derive(Debug)]
pub struct ZendCallable<'a>(OwnedZval<'a>);

impl<'a> ZendCallable<'a> {
    /// Attempts to create a new [`ZendCallable`] from a zval.
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

    /// Attempts to create a new [`ZendCallable`] by taking ownership of a Zval.
    /// Returns a result containing the callable if the zval was callable.
    ///
    /// # Parameters
    ///
    /// * `callable` - The underlying [`Zval`] that is callable.
    pub fn new_owned(callable: Zval) -> Result<Self> {
        if callable.is_callable() {
            Ok(Self(OwnedZval::Owned(callable)))
        } else {
            Err(Error::Callable)
        }
    }

    /// Attempts to create a new [`ZendCallable`] from a function name. Returns
    /// a result containing the callable if the function existed and was
    /// callable.
    ///
    /// # Parameters
    ///
    /// * `name` - Name of the callable function.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ext_php_rs::types::ZendCallable;
    ///
    /// let strpos = ZendCallable::try_from_name("strpos").unwrap();
    /// let result = strpos.try_call(vec![&"hello", &"e"]).unwrap();
    /// assert_eq!(result.long(), Some(1));
    /// ```
    pub fn try_from_name(name: &str) -> Result<Self> {
        let mut callable = Zval::new();
        callable.set_string(name, false)?;

        Self::new_owned(callable)
    }

    /// Attempts to call the callable with a list of arguments to pass to the
    /// function.
    ///
    /// You should not call this function directly, rather through the
    /// [`call_user_func`] macro.
    ///
    /// # Parameters
    ///
    /// * `params` - A list of parameters to call the function with.
    ///
    /// # Returns
    ///
    /// Returns the result wrapped in [`Ok`] upon success. If calling the
    /// callable fails, or an exception is thrown, an [`Err`] is returned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ext_php_rs::types::ZendCallable;
    ///
    /// let strpos = ZendCallable::try_from_name("strpos").unwrap();
    /// let result = strpos.try_call(vec![&"hello", &"e"]).unwrap();
    /// assert_eq!(result.long(), Some(1));
    /// ```
    pub fn try_call(&self, params: Vec<&dyn IntoZvalDyn>) -> Result<Zval> {
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
                self.0.as_ref() as *const crate::ffi::_zval_struct as *mut crate::ffi::_zval_struct,
                &mut retval,
                len as _,
                packed.as_ptr() as *mut _,
                std::ptr::null_mut(),
            )
        };

        if result < 0 {
            Err(Error::Callable)
        } else if let Some(e) = ExecutorGlobals::take_exception() {
            Err(Error::Exception(e))
        } else {
            Ok(retval)
        }
    }
}

impl<'a> FromZval<'a> for ZendCallable<'a> {
    const TYPE: DataType = DataType::Callable;

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        ZendCallable::new(zval).ok()
    }
}

impl<'a> TryFrom<Zval> for ZendCallable<'a> {
    type Error = Error;

    fn try_from(value: Zval) -> Result<Self> {
        ZendCallable::new_owned(value)
    }
}

/// A container for a zval. Either contains a reference to a zval or an owned
/// zval.
#[derive(Debug)]
enum OwnedZval<'a> {
    Reference(&'a Zval),
    Owned(Zval),
}

impl<'a> OwnedZval<'a> {
    fn as_ref(&self) -> &Zval {
        match self {
            OwnedZval::Reference(zv) => zv,
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
