//! Wrapper for enums introduced in C.

use std::{convert::TryFrom, fmt::Display};

use crate::{
    bindings::{
        IS_ARRAY, IS_CALLABLE, IS_CONSTANT_AST, IS_DOUBLE, IS_FALSE, IS_LONG, IS_MIXED, IS_NULL,
        IS_OBJECT, IS_PTR, IS_REFERENCE, IS_RESOURCE, IS_STRING, IS_TRUE, IS_UNDEF, IS_VOID,
        _IS_BOOL,
    },
    errors::{Error, Result},
    php::flags::ZvalTypeFlags,
};

/// Valid data types for PHP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataType {
    Undef,
    Null,
    False,
    True,
    Long,
    Double,
    String,
    Array,
    Object(Option<&'static str>),
    Resource,
    Reference,
    Callable,
    ConstantExpression,
    Void,
    Mixed,
    Bool,
    Ptr,
}

impl Default for DataType {
    fn default() -> Self {
        Self::Void
    }
}

impl DataType {
    /// Returns the integer representation of the data type.
    pub const fn as_u32(&self) -> u32 {
        match self {
            DataType::Undef => IS_UNDEF,
            DataType::Null => IS_NULL,
            DataType::False => IS_FALSE,
            DataType::True => IS_TRUE,
            DataType::Long => IS_LONG,
            DataType::Double => IS_DOUBLE,
            DataType::String => IS_STRING,
            DataType::Array => IS_ARRAY,
            DataType::Object(_) => IS_OBJECT,
            DataType::Resource => IS_RESOURCE,
            DataType::Reference => IS_RESOURCE,
            DataType::Callable => IS_CALLABLE,
            DataType::ConstantExpression => IS_CONSTANT_AST,
            DataType::Void => IS_VOID,
            DataType::Mixed => IS_MIXED,
            DataType::Bool => _IS_BOOL,
            DataType::Ptr => IS_PTR,
        }
    }
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
        contains!(Resource);
        contains!(Callable);
        contains!(ConstantExpression);
        contains!(Void);

        if value.contains(ZvalTypeFlags::Object) {
            return Ok(DataType::Object(None));
        }

        Err(Error::UnknownDatatype(0))
    }
}

impl From<u32> for DataType {
    #[allow(clippy::bad_bit_mask)]
    fn from(value: u32) -> Self {
        macro_rules! contains {
            ($c: ident, $t: ident) => {
                if (value & $c) == $c {
                    return DataType::$t;
                }
            };
        }

        contains!(IS_VOID, Void);
        contains!(IS_CALLABLE, Callable);
        contains!(IS_CONSTANT_AST, ConstantExpression);
        contains!(IS_REFERENCE, Reference);
        contains!(IS_RESOURCE, Resource);
        contains!(IS_ARRAY, Array);
        contains!(IS_STRING, String);
        contains!(IS_DOUBLE, Double);
        contains!(IS_LONG, Long);
        contains!(IS_TRUE, True);
        contains!(IS_FALSE, False);
        contains!(IS_NULL, Null);
        contains!(IS_PTR, Ptr);

        if (value & IS_OBJECT) == IS_OBJECT {
            return DataType::Object(None);
        }

        contains!(IS_UNDEF, Undef);

        DataType::Mixed
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
            DataType::Object(obj) => write!(f, "{}", obj.as_deref().unwrap_or("Object")),
            DataType::Resource => write!(f, "Resource"),
            DataType::Reference => write!(f, "Reference"),
            DataType::Callable => write!(f, "Callable"),
            DataType::ConstantExpression => write!(f, "Constant Expression"),
            DataType::Void => write!(f, "Void"),
            DataType::Bool => write!(f, "Bool"),
            DataType::Mixed => write!(f, "Mixed"),
            DataType::Ptr => write!(f, "Pointer"),
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
        assert_eq!(DataType::try_from(IS_OBJECT), Ok(DataType::Object(None)));
        test!(IS_RESOURCE, Resource);
        test!(IS_REFERENCE, Reference);
        test!(IS_CONSTANT_AST, ConstantExpression);
        test!(IS_CALLABLE, Callable);
        test!(IS_VOID, Void);

        test!(IS_INTERNED_STRING_EX, String);
        test!(IS_STRING_EX, String);
        test!(IS_ARRAY_EX, Array);
        assert_eq!(DataType::try_from(IS_OBJECT_EX), Ok(DataType::Object(None)));
        test!(IS_RESOURCE_EX, Resource);
        test!(IS_REFERENCE_EX, Reference);
        test!(IS_CONSTANT_AST_EX, ConstantExpression);
    }
}
