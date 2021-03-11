use crate::bindings::{
    IS_ARRAY, IS_CONSTANT_AST, IS_DOUBLE, IS_FALSE, IS_LONG, IS_NULL, IS_OBJECT, IS_REFERENCE,
    IS_RESOURCE, IS_STRING, IS_TRUE, IS_UNDEF, IS_VOID,
};

use super::types::ZendLong;

/// Valid data types for PHP.
#[derive(Clone, Copy)]
pub enum DataType {
    Undef = IS_UNDEF as isize,

    Null = IS_NULL as isize,
    False = IS_FALSE as isize,
    True = IS_TRUE as isize,
    Long = IS_LONG as isize,
    Double = IS_DOUBLE as isize,
    String = IS_STRING as isize,
    Array = IS_ARRAY as isize,
    Object = IS_OBJECT as isize,
    Resource = IS_RESOURCE as isize,
    Reference = IS_REFERENCE as isize,

    ConstantExpression = IS_CONSTANT_AST as isize,
    Void = IS_VOID as isize,
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
