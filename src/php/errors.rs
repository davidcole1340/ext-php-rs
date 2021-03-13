//! Contains all the base PHP extensions.

use super::class::ClassEntry;
use crate::bindings::{
    zend_ce_argument_count_error, zend_ce_arithmetic_error, zend_ce_compile_error,
    zend_ce_division_by_zero_error, zend_ce_error_exception, zend_ce_exception,
    zend_ce_parse_error, zend_ce_throwable, zend_ce_type_error, zend_ce_unhandled_match_error,
    zend_ce_value_error,
};

impl ClassEntry {
    /// Returns the base `Throwable` class.
    pub fn throwable<'a>() -> Option<&'a Self> {
        unsafe { zend_ce_throwable.as_ref() }
    }

    /// Returns the base `Exception` class.
    pub fn exception<'a>() -> Option<&'a Self> {
        unsafe { zend_ce_exception.as_ref() }
    }

    /// Returns the base `ErrorException` class.
    pub fn error_exception<'a>() -> Option<&'a Self> {
        unsafe { zend_ce_error_exception.as_ref() }
    }

    /// Returns the base `CompileError` class.
    pub fn compile_error<'a>() -> Option<&'a Self> {
        unsafe { zend_ce_compile_error.as_ref() }
    }

    /// Returns the base `ParseError` class.
    pub fn parse_error<'a>() -> Option<&'a Self> {
        unsafe { zend_ce_parse_error.as_ref() }
    }

    /// Returns the base `TypeError` class.
    pub fn type_error<'a>() -> Option<&'a Self> {
        unsafe { zend_ce_type_error.as_ref() }
    }

    /// Returns the base `ArgumentCountError` class.
    pub fn argument_count_error<'a>() -> Option<&'a Self> {
        unsafe { zend_ce_argument_count_error.as_ref() }
    }

    /// Returns the base `ValueError` class.
    pub fn value_error<'a>() -> Option<&'a Self> {
        unsafe { zend_ce_value_error.as_ref() }
    }

    /// Returns the base `ArithmeticError` class.
    pub fn arithmetic_error<'a>() -> Option<&'a Self> {
        unsafe { zend_ce_arithmetic_error.as_ref() }
    }

    /// Returns the base `DivisionByZeroError` class.
    pub fn division_by_zero_error<'a>() -> Option<&'a Self> {
        unsafe { zend_ce_division_by_zero_error.as_ref() }
    }

    /// Returns the base `UnhandledMatchError` class.
    pub fn unhandled_match_error<'a>() -> Option<&'a Self> {
        unsafe { zend_ce_unhandled_match_error.as_ref() }
    }
}
