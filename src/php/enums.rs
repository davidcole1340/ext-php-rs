//! Wrapper for enums introduced in C.

use std::{convert::TryFrom, fmt::Display};

use crate::{
    bindings::{
        IS_ARRAY, IS_CALLABLE, IS_CONSTANT_AST, IS_DOUBLE, IS_FALSE, IS_LONG, IS_NULL, IS_OBJECT,
        IS_REFERENCE, IS_RESOURCE, IS_STRING, IS_TRUE, IS_UNDEF, IS_VOID,
    },
    errors::{Error, Result},
};

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

impl TryFrom<u8> for DataType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value as u32 {
            IS_UNDEF => Ok(DataType::Undef),
            IS_NULL => Ok(DataType::Null),
            IS_FALSE => Ok(DataType::False),
            IS_TRUE => Ok(DataType::True),
            IS_LONG => Ok(DataType::Long),
            IS_DOUBLE => Ok(DataType::Double),
            IS_STRING => Ok(DataType::String),
            IS_ARRAY => Ok(DataType::Array),
            IS_OBJECT => Ok(DataType::Object),
            IS_RESOURCE => Ok(DataType::Resource),
            IS_REFERENCE => Ok(DataType::Reference),
            IS_CALLABLE => Ok(DataType::Callable),
            IS_CONSTANT_AST => Ok(DataType::ConstantExpression),
            IS_VOID => Ok(DataType::Void),

            _ => Err(Error::UnknownDatatype(value)),
        }
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Undef => write!(f, "Undefined"),
            DataType::Null => write!(f, "Null"),
            DataType::False => write!(f, "False"),
            DataType::True => write!(f, "True"),
            DataType::Long => write!(f, "Long"),
            DataType::Double => write!(f, "Double"),
            DataType::String => write!(f, "String"),
            DataType::Array => write!(f, "Array"),
            DataType::Object => write!(f, "Object"),
            DataType::Resource => write!(f, "Resource"),
            DataType::Reference => write!(f, "Reference"),
            DataType::Callable => write!(f, "Callable"),
            DataType::ConstantExpression => write!(f, "Constant Expression"),
            DataType::Void => write!(f, "Void"),
        }
    }
}
