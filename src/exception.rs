//! Types and functions used for throwing exceptions from Rust to PHP.

use std::ffi::CString;

use crate::{
    class::RegisteredClass,
    error::{Error, Result},
    ffi::zend_throw_exception_ex,
    flags::ClassFlags,
    zend::{ce, ClassEntry},
};

/// Result type with the error variant as a [`PhpException`].
pub type PhpResult<T = ()> = std::result::Result<T, PhpException>;

/// Represents a PHP exception which can be thrown using the `throw()` function.
/// Primarily used to return from a [`Result<T, PhpException>`] which can
/// immediately be thrown by the `ext-php-rs` macro API.
///
/// There are default [`From`] implementations for any type that implements
/// [`ToString`], so these can also be returned from these functions. You can
/// also implement [`From<T>`] for your custom error type.
#[derive(Debug)]
pub struct PhpException {
    message: String,
    code: i32,
    ex: &'static ClassEntry,
}

impl PhpException {
    /// Creates a new exception instance.
    ///
    /// # Parameters
    ///
    /// * `message` - Message to contain in the exception.
    /// * `code` - Integer code to go inside the exception.
    /// * `ex` - Exception type to throw.
    pub fn new(message: String, code: i32, ex: &'static ClassEntry) -> Self {
        Self { message, code, ex }
    }

    /// Creates a new default exception instance, using the default PHP
    /// `Exception` type as the exception type, with an integer code of
    /// zero.
    ///
    /// # Parameters
    ///
    /// * `message` - Message to contain in the exception.
    pub fn default(message: String) -> Self {
        Self::new(message, 0, ce::exception())
    }

    /// Creates an instance of an exception from a PHP class type and a message.
    ///
    /// # Parameters
    ///
    /// * `message` - Message to contain in the exception.
    pub fn from_class<T: RegisteredClass>(message: String) -> Self {
        Self::new(message, 0, T::get_metadata().ce())
    }

    /// Throws the exception, returning nothing inside a result if successful
    /// and an error otherwise.
    pub fn throw(self) -> Result<()> {
        throw_with_code(self.ex, self.code, &self.message)
    }
}

impl From<String> for PhpException {
    fn from(str: String) -> Self {
        Self::default(str)
    }
}

impl From<&str> for PhpException {
    fn from(str: &str) -> Self {
        Self::default(str.into())
    }
}

#[cfg(feature = "anyhow")]
impl From<anyhow::Error> for PhpException {
    fn from(err: anyhow::Error) -> Self {
        Self::new(format!("{:#}", err), 0, crate::zend::ce::exception())
    }
}

/// Throws an exception with a given message. See [`ClassEntry`] for some
/// built-in exception types.
///
/// Returns a result containing nothing if the exception was successfully
/// thrown.
///
/// # Parameters
///
/// * `ex` - The exception type to throw.
/// * `message` - The message to display when throwing the exception.
///
/// # Examples
///
/// ```no_run
/// use ext_php_rs::{zend::{ce, ClassEntry}, exception::throw};
///
/// throw(ce::compile_error(), "This is a CompileError.");
/// ```
pub fn throw(ex: &ClassEntry, message: &str) -> Result<()> {
    throw_with_code(ex, 0, message)
}

/// Throws an exception with a given message and status code. See [`ClassEntry`]
/// for some built-in exception types.
///
/// Returns a result containing nothing if the exception was successfully
/// thrown.
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
/// use ext_php_rs::{zend::{ce, ClassEntry}, exception::throw_with_code};
///
/// throw_with_code(ce::compile_error(), 123, "This is a CompileError.");
/// ```
pub fn throw_with_code(ex: &ClassEntry, code: i32, message: &str) -> Result<()> {
    let flags = ex.flags();

    // Can't throw an interface or abstract class.
    if flags.contains(ClassFlags::Interface) || flags.contains(ClassFlags::Abstract) {
        return Err(Error::InvalidException(flags));
    }

    // SAFETY: We are given a reference to a `ClassEntry` therefore when we cast it
    // to a pointer it will be valid.
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
