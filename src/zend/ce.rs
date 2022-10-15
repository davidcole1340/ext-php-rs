//! Stock class entries registered with PHP, primarily exceptions.

#![allow(clippy::unwrap_used)]

use crate::ffi::{
    zend_ce_aggregate, zend_ce_argument_count_error, zend_ce_arithmetic_error, zend_ce_arrayaccess,
    zend_ce_compile_error, zend_ce_countable, zend_ce_division_by_zero_error,
    zend_ce_error_exception, zend_ce_exception, zend_ce_iterator, zend_ce_parse_error,
    zend_ce_serializable, zend_ce_stringable, zend_ce_throwable, zend_ce_traversable,
    zend_ce_type_error, zend_ce_unhandled_match_error, zend_ce_value_error,
    zend_standard_class_def,
};

use super::ClassEntry;

/// Returns the base `stdClass` class.
pub fn stdclass() -> &'static ClassEntry {
    unsafe { zend_standard_class_def.as_ref() }.unwrap()
}

/// Returns the base `Throwable` class.
pub fn throwable() -> &'static ClassEntry {
    unsafe { zend_ce_throwable.as_ref() }.unwrap()
}

/// Returns the base `Exception` class.
pub fn exception() -> &'static ClassEntry {
    unsafe { zend_ce_exception.as_ref() }.unwrap()
}

/// Returns the base `ErrorException` class.
pub fn error_exception() -> &'static ClassEntry {
    unsafe { zend_ce_error_exception.as_ref() }.unwrap()
}

/// Returns the base `CompileError` class.
pub fn compile_error() -> &'static ClassEntry {
    unsafe { zend_ce_compile_error.as_ref() }.unwrap()
}

/// Returns the base `ParseError` class.
pub fn parse_error() -> &'static ClassEntry {
    unsafe { zend_ce_parse_error.as_ref() }.unwrap()
}

/// Returns the base `TypeError` class.
pub fn type_error() -> &'static ClassEntry {
    unsafe { zend_ce_type_error.as_ref() }.unwrap()
}

/// Returns the base `ArgumentCountError` class.
pub fn argument_count_error() -> &'static ClassEntry {
    unsafe { zend_ce_argument_count_error.as_ref() }.unwrap()
}

/// Returns the base `ValueError` class.
pub fn value_error() -> &'static ClassEntry {
    unsafe { zend_ce_value_error.as_ref() }.unwrap()
}

/// Returns the base `ArithmeticError` class.
pub fn arithmetic_error() -> &'static ClassEntry {
    unsafe { zend_ce_arithmetic_error.as_ref() }.unwrap()
}

/// Returns the base `DivisionByZeroError` class.
pub fn division_by_zero_error() -> &'static ClassEntry {
    unsafe { zend_ce_division_by_zero_error.as_ref() }.unwrap()
}

/// Returns the base `UnhandledMatchError` class.
pub fn unhandled_match_error() -> &'static ClassEntry {
    unsafe { zend_ce_unhandled_match_error.as_ref() }.unwrap()
}

/// Returns the [`Traversable`](https://www.php.net/manual/en/class.traversable.php) interface.
pub fn traversable() -> &'static ClassEntry {
    unsafe { zend_ce_traversable.as_ref() }.unwrap()
}

/// Returns the [`IteratorAggregate`](https://www.php.net/manual/en/class.iteratoraggregate.php) interface.
pub fn aggregate() -> &'static ClassEntry {
    unsafe { zend_ce_aggregate.as_ref() }.unwrap()
}

/// Returns the [`Iterator`](https://www.php.net/manual/en/class.iterator.php) interface.
pub fn iterator() -> &'static ClassEntry {
    unsafe { zend_ce_iterator.as_ref() }.unwrap()
}

/// Returns the [`ArrayAccess`](https://www.php.net/manual/en/class.arrayaccess.php) interface.
pub fn arrayaccess() -> &'static ClassEntry {
    unsafe { zend_ce_arrayaccess.as_ref() }.unwrap()
}

/// Returns the [`Serializable`](https://www.php.net/manual/en/class.serializable.php) interface.
pub fn serializable() -> &'static ClassEntry {
    unsafe { zend_ce_serializable.as_ref() }.unwrap()
}

/// Returns the [`Countable`](https://www.php.net/manual/en/class.countable.php) interface.
pub fn countable() -> &'static ClassEntry {
    unsafe { zend_ce_countable.as_ref() }.unwrap()
}

/// Returns the [`Stringable`](https://www.php.net/manual/en/class.stringable.php) interface.
pub fn stringable() -> &'static ClassEntry {
    unsafe { zend_ce_stringable.as_ref() }.unwrap()
}
