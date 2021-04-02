//! Wrapper for enums introduced in C.

use crate::bindings::{
    IS_ARRAY, IS_CALLABLE, IS_CONSTANT_AST, IS_DOUBLE, IS_FALSE, IS_LONG, IS_NULL, IS_OBJECT,
    IS_REFERENCE, IS_RESOURCE, IS_STRING, IS_TRUE, IS_UNDEF, IS_VOID,
};

use super::types::long::ZendLong;

/// Valid data types for PHP.
#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum DataType {
    Undef = IS_UNDEF,

    Null = IS_NULL,
    False = IS_FALSE,
    True = IS_TRUE,
    Long = IS_LONG,
    Double = IS_DOUBLE,
    String = IS_STRING,
    Array = IS_ARRAY,
    Object = IS_OBJECT,
    Resource = IS_RESOURCE,
    Reference = IS_REFERENCE,
    Callable = IS_CALLABLE,

    ConstantExpression = IS_CONSTANT_AST,
    Void = IS_VOID,
}

impl From<ZendLong> for DataType {
    fn from(_: ZendLong) -> Self {
        Self::Long
    }
}

impl From<bool> for DataType {
    fn from(x: bool) -> Self {
        if x {
            Self::True
        } else {
            Self::False
        }
    }
}

impl From<f64> for DataType {
    fn from(_: f64) -> Self {
        Self::Double
    }
}

impl From<String> for DataType {
    fn from(_: String) -> Self {
        Self::String
    }
}
