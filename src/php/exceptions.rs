//! Contains all the base PHP throwables, including `Throwable` and `Exception`.

use std::ffi::CString;

use super::{class::ClassEntry, types::object::RegisteredClass};
use crate::{
    bindings::{
        zend_ce_argument_count_error, zend_ce_arithmetic_error, zend_ce_compile_error,
        zend_ce_division_by_zero_error, zend_ce_error_exception, zend_ce_exception,
        zend_ce_parse_error, zend_ce_throwable, zend_ce_type_error, zend_ce_unhandled_match_error,
        zend_ce_value_error, zend_throw_exception_ex,
    },
    errors::{Error, Result},
    php::flags::ClassFlags,
};

/// Represents a PHP exception which can be thrown using the `throw()` function. Primarily used to
/// return from a [`Result<T, PhpException>`] which can immediately be thrown by the `ext-php-rs`
/// macro API.
///
/// There are default [`From`] implementations for any type that implements [`ToString`], so these
/// can also be returned from these functions. You can also implement [`From<T>`] for your custom
/// error type.
#[derive(Debug)]
pub struct PhpException<'a> {
    message: String,
    code: i32,
    ex: &'a ClassEntry,
}

impl<'a> PhpException<'a> {
    /// Creates a new exception instance.
    ///
    /// # Parameters
    ///
    /// * `message` - Message to contain in the exception.
    /// * `code` - Integer code to go inside the exception.
    /// * `ex` - Exception type to throw.
    pub fn new(message: String, code: i32, ex: &'a ClassEntry) -> Self {
        Self { message, code, ex }
    }

    /// Creates a new default exception instance, using the default PHP `Exception` type as the
    /// exception type, with an integer code of zero.
    ///
    /// # Parameters
    ///
    /// * `message` - Message to contain in the exception.
    pub fn default(message: String) -> Self {
        Self::new(message, 0, ClassEntry::exception())
    }

    /// Creates an instance of an exception from a PHP class type and a message.
    ///
    /// # Parameters
    ///
    /// * `message` - Message to contain in the exception.
    pub fn from_class<T: RegisteredClass>(message: String) -> Self {
        Self::new(message, 0, T::get_metadata().ce())
    }

    /// Throws the exception, returning nothing inside a result if successful and an error
    /// otherwise.
    pub fn throw(self) -> Result<()> {
        throw_with_code(self.ex, self.code, &self.message)
    }
}

impl<'a> From<String> for PhpException<'a> {
    fn from(str: String) -> Self {
        Self::default(str)
    }
}

impl<'a> From<&str> for PhpException<'a> {
    fn from(str: &str) -> Self {
        Self::default(str.into())
    }
}

/// Throws an exception with a given message. See [`ClassEntry`] for some built-in exception
/// types.
///
/// Returns a result containing nothing if the exception was successfully thown.
///
/// # Parameters
///
/// * `ex` - The exception type to throw.
/// * `message` - The message to display when throwing the exception.
///
/// # Examples
///
/// ```no_run
/// use ext_php_rs::php::{class::ClassEntry, exceptions::throw};
///
/// throw(ClassEntry::compile_error(), "This is a CompileError.");
/// ```
pub fn throw(ex: &ClassEntry, message: &str) -> Result<()> {
    throw_with_code(ex, 0, message)
}

/// Throws an exception with a given message and status code. See [`ClassEntry`] for some built-in
/// exception types.
///
/// Returns a result containing nothing if the exception was successfully thown.
///
/// # Parameters
///
/// * `ex` - The exception type to throw.
/// * `code` - The status code to use when throwing the exception.
/// * `message` - The message to display when throwing the exception.
///
/// # Examples
///
/// ```no_run
/// use ext_php_rs::php::{class::ClassEntry, exceptions::throw_with_code};
///
/// throw_with_code(ClassEntry::compile_error(), 123, "This is a CompileError.");
/// ```
pub fn throw_with_code(ex: &ClassEntry, code: i32, message: &str) -> Result<()> {
    let flags = ex.flags();

    // Can't throw an interface or abstract class.
    if flags.contains(ClassFlags::Interface) || flags.contains(ClassFlags::Abstract) {
        return Err(Error::InvalidException(flags));
    }

    // SAFETY: We are given a reference to a `ClassEntry` therefore when we cast it to a pointer it
    // will be valid.
    unsafe {
        zend_throw_exception_ex(
            (ex as *const _) as *mut _,
            code as _,
            CString::new("%s")?.as_ptr(),
            CString::new(message)?.as_ptr(),
        )
    };
    Ok(())
}

// SAFETY: All default exceptions have been initialized by the time we should use these (in the module
// startup function). Note that they are not valid during the module init function, but rather than
// wrapping everything
#[allow(clippy::unwrap_used)]
impl ClassEntry {
    /// Returns the base `Throwable` class.
    pub fn throwable() -> &'static Self {
        unsafe { zend_ce_throwable.as_ref() }.unwrap()
    }

    /// Returns the base `Exception` class.
    pub fn exception() -> &'static Self {
        unsafe { zend_ce_exception.as_ref() }.unwrap()
    }

    /// Returns the base `ErrorException` class.
    pub fn error_exception() -> &'static Self {
        unsafe { zend_ce_error_exception.as_ref() }.unwrap()
    }

    /// Returns the base `CompileError` class.
    pub fn compile_error() -> &'static Self {
        unsafe { zend_ce_compile_error.as_ref() }.unwrap()
    }

    /// Returns the base `ParseError` class.
    pub fn parse_error() -> &'static Self {
        unsafe { zend_ce_parse_error.as_ref() }.unwrap()
    }

    /// Returns the base `TypeError` class.
    pub fn type_error() -> &'static Self {
        unsafe { zend_ce_type_error.as_ref() }.unwrap()
    }

    /// Returns the base `ArgumentCountError` class.
    pub fn argument_count_error() -> &'static Self {
        unsafe { zend_ce_argument_count_error.as_ref() }.unwrap()
    }

    /// Returns the base `ValueError` class.
    pub fn value_error() -> &'static Self {
        unsafe { zend_ce_value_error.as_ref() }.unwrap()
    }

    /// Returns the base `ArithmeticError` class.
    pub fn arithmetic_error() -> &'static Self {
        unsafe { zend_ce_arithmetic_error.as_ref() }.unwrap()
    }

    /// Returns the base `DivisionByZeroError` class.
    pub fn division_by_zero_error() -> &'static Self {
        unsafe { zend_ce_division_by_zero_error.as_ref() }.unwrap()
    }

    /// Returns the base `UnhandledMatchError` class.
    pub fn unhandled_match_error() -> &'static Self {
        unsafe { zend_ce_unhandled_match_error.as_ref() }.unwrap()
    }
}
