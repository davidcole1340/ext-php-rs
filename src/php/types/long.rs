//! Represents an integer introduced in PHP. Note that the size of this integer differs.
//! On a 32-bit system, a ZendLong is 32-bits, while on a 64-bit system it is 64-bits.

use crate::bindings::zend_long;

/// Internal identifier used for a long.
/// The size depends on the system architecture. On 32-bit systems,
/// a ZendLong is 32-bits, while on a 64-bit system it is 64-bits.
pub type ZendLong = zend_long;
