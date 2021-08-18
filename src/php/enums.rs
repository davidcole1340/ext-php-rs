//! Wrapper for enums introduced in C.

use std::{convert::TryFrom, fmt::Display};

use crate::{
    bindings::{
        IS_ARRAY, IS_CALLABLE, IS_CONSTANT_AST, IS_DOUBLE, IS_FALSE, IS_LONG, IS_NULL, IS_OBJECT,
        IS_REFERENCE, IS_RESOURCE, IS_STRING, IS_TRUE, IS_UNDEF, IS_VOID, _IS_BOOL,
    },
    errors::{Error, Result},
    php::flags::ZvalTypeFlags,
};

/// Valid data types for PHP.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

    /// Only used when creating arguments.
    Bool = _IS_BOOL,
}

// TODO: Ideally want something like this
// pub struct Type {
//     data_type: DataType,
//     is_refcounted: bool,
//     is_collectable: bool,
//     is_immutable: bool,
//     is_persistent: bool,
// }
//
// impl From<u32> for Type { ... }

impl TryFrom<ZvalTypeFlags> for DataType {
    type Error = Error;

    fn try_from(value: ZvalTypeFlags) -> Result<Self> {
        macro_rules! contains {
            ($t: ident) => {
                if value.contains(ZvalTypeFlags::$t) {
                    return Ok(DataType::$t);
                }
            };
        }

        contains!(Undef);
        contains!(Null);
        contains!(False);
        contains!(True);
        contains!(False);
        contains!(Long);
        contains!(Double);
        contains!(String);
        contains!(Array);
        contains!(Object);
        contains!(Resource);
        contains!(Callable);
        contains!(ConstantExpression);
        contains!(Void);

        Err(Error::UnknownDatatype(0))
    }
}

impl TryFrom<u32> for DataType {
    type Error = Error;

    #[allow(clippy::bad_bit_mask)]
    fn try_from(value: u32) -> Result<Self> {
        macro_rules! contains {
            ($c: ident, $t: ident) => {
                if (value & $c) == $c {
                    return Ok(DataType::$t);
                }
            };
        }

        contains!(IS_VOID, Void);
        contains!(IS_CALLABLE, Callable);
        contains!(IS_CONSTANT_AST, ConstantExpression);
        contains!(IS_CONSTANT_AST, ConstantExpression);
        contains!(IS_CONSTANT_AST, ConstantExpression);
        contains!(IS_REFERENCE, Reference);
        contains!(IS_RESOURCE, Resource);
        contains!(IS_OBJECT, Object);
        contains!(IS_ARRAY, Array);
        contains!(IS_STRING, String);
        contains!(IS_DOUBLE, Double);
        contains!(IS_LONG, Long);
        contains!(IS_TRUE, True);
        contains!(IS_FALSE, False);
        contains!(IS_NULL, Null);
        contains!(IS_UNDEF, Undef);

        Err(Error::UnknownDatatype(value))
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
            DataType::Bool => write!(f, "Bool"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DataType;
    use crate::bindings::{
        IS_ARRAY, IS_ARRAY_EX, IS_CALLABLE, IS_CONSTANT_AST, IS_CONSTANT_AST_EX, IS_DOUBLE,
        IS_FALSE, IS_INTERNED_STRING_EX, IS_LONG, IS_NULL, IS_OBJECT, IS_OBJECT_EX, IS_REFERENCE,
        IS_REFERENCE_EX, IS_RESOURCE, IS_RESOURCE_EX, IS_STRING, IS_STRING_EX, IS_TRUE, IS_UNDEF,
        IS_VOID,
    };
    use std::convert::TryFrom;

    #[test]
    fn test_datatype() {
        macro_rules! test {
            ($c: ident, $t: ident) => {
                assert_eq!(DataType::try_from($c), Ok(DataType::$t));
            };
        }

        test!(IS_UNDEF, Undef);
        test!(IS_NULL, Null);
        test!(IS_FALSE, False);
        test!(IS_TRUE, True);
        test!(IS_LONG, Long);
        test!(IS_DOUBLE, Double);
        test!(IS_STRING, String);
        test!(IS_ARRAY, Array);
        test!(IS_OBJECT, Object);
        test!(IS_RESOURCE, Resource);
        test!(IS_REFERENCE, Reference);
        test!(IS_CONSTANT_AST, ConstantExpression);
        test!(IS_CALLABLE, Callable);
        test!(IS_VOID, Void);

        test!(IS_INTERNED_STRING_EX, String);
        test!(IS_STRING_EX, String);
        test!(IS_ARRAY_EX, Array);
        test!(IS_OBJECT_EX, Object);
        test!(IS_RESOURCE_EX, Resource);
        test!(IS_REFERENCE_EX, Reference);
        test!(IS_CONSTANT_AST_EX, ConstantExpression);
    }
}
