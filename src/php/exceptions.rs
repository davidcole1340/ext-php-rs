//! Contains all the base PHP throwables, including `Throwable` and `Exception`.

use super::class::ClassEntry;
use crate::{
    bindings::{
        zend_ce_argument_count_error, zend_ce_arithmetic_error, zend_ce_compile_error,
        zend_ce_division_by_zero_error, zend_ce_error_exception, zend_ce_exception,
        zend_ce_parse_error, zend_ce_throwable, zend_ce_type_error, zend_ce_unhandled_match_error,
        zend_ce_value_error, zend_throw_exception_ex,
    },
    functions::c_str,
};

/// Throws an exception with a given message. See [`ClassEntry`] for some built-in exception
/// types.
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
pub fn throw(ex: &ClassEntry, message: &str) {
    throw_with_code(ex, 0, message);
}

/// Throws an exception with a given message and status code. See [`ClassEntry`] for some built-in
/// exception types.
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
pub fn throw_with_code(ex: &ClassEntry, code: i32, message: &str) {
    // SAFETY: We are given a reference to a `ClassEntry` therefore when we cast it to a pointer it
    // will be valid.
    unsafe {
        zend_throw_exception_ex(
            (ex as *const _) as *mut _,
            code as _,
            c_str("%s"),
            c_str(message),
        )
    };
}

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
