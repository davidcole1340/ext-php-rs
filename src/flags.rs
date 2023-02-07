//! Flags and enums used in PHP and the Zend engine.

use bitflags::bitflags;

#[cfg(not(php82))]
use crate::ffi::ZEND_ACC_REUSE_GET_ITERATOR;
use crate::ffi::{
    CONST_CS, CONST_DEPRECATED, CONST_NO_FILE_CACHE, CONST_PERSISTENT, IS_ARRAY, IS_CALLABLE,
    IS_CONSTANT_AST, IS_DOUBLE, IS_FALSE, IS_LONG, IS_MIXED, IS_NULL, IS_OBJECT, IS_PTR,
    IS_REFERENCE, IS_RESOURCE, IS_STRING, IS_TRUE, IS_TYPE_COLLECTABLE, IS_TYPE_REFCOUNTED,
    IS_UNDEF, IS_VOID, ZEND_ACC_ABSTRACT, ZEND_ACC_ANON_CLASS, ZEND_ACC_CALL_VIA_TRAMPOLINE,
    ZEND_ACC_CHANGED, ZEND_ACC_CLOSURE, ZEND_ACC_CONSTANTS_UPDATED, ZEND_ACC_CTOR,
    ZEND_ACC_DEPRECATED, ZEND_ACC_DONE_PASS_TWO, ZEND_ACC_EARLY_BINDING, ZEND_ACC_FAKE_CLOSURE,
    ZEND_ACC_FINAL, ZEND_ACC_GENERATOR, ZEND_ACC_HAS_FINALLY_BLOCK, ZEND_ACC_HAS_RETURN_TYPE,
    ZEND_ACC_HAS_TYPE_HINTS, ZEND_ACC_HEAP_RT_CACHE, ZEND_ACC_IMMUTABLE,
    ZEND_ACC_IMPLICIT_ABSTRACT_CLASS, ZEND_ACC_INTERFACE, ZEND_ACC_LINKED, ZEND_ACC_NEARLY_LINKED,
    ZEND_ACC_NEVER_CACHE, ZEND_ACC_NO_DYNAMIC_PROPERTIES, ZEND_ACC_PRELOADED, ZEND_ACC_PRIVATE,
    ZEND_ACC_PROMOTED, ZEND_ACC_PROTECTED, ZEND_ACC_PUBLIC, ZEND_ACC_RESOLVED_INTERFACES,
    ZEND_ACC_RESOLVED_PARENT, ZEND_ACC_RETURN_REFERENCE, ZEND_ACC_STATIC, ZEND_ACC_STRICT_TYPES,
    ZEND_ACC_TOP_LEVEL, ZEND_ACC_TRAIT, ZEND_ACC_TRAIT_CLONE, ZEND_ACC_UNRESOLVED_VARIANCE,
    ZEND_ACC_USES_THIS, ZEND_ACC_USE_GUARDS, ZEND_ACC_VARIADIC, ZEND_HAS_STATIC_IN_METHODS,
    Z_TYPE_FLAGS_SHIFT, _IS_BOOL,
};

use std::{convert::TryFrom, fmt::Display};

use crate::error::{Error, Result};

bitflags! {
    /// Flags used for setting the type of Zval.
    pub struct ZvalTypeFlags: u32 {
        const Undef = IS_UNDEF;
        const Null = IS_NULL;
        const False = IS_FALSE;
        const True = IS_TRUE;
        const Long = IS_LONG;
        const Double = IS_DOUBLE;
        const String = IS_STRING;
        const Array = IS_ARRAY;
        const Object = IS_OBJECT;
        const Resource = IS_RESOURCE;
        const Reference = IS_REFERENCE;
        const Callable = IS_CALLABLE;
        const ConstantExpression = IS_CONSTANT_AST;
        const Void = IS_VOID;
        const Ptr = IS_PTR;

        const InternedStringEx = Self::String.bits;
        const StringEx = Self::String.bits | Self::RefCounted.bits;
        const ArrayEx = Self::Array.bits | Self::RefCounted.bits | Self::Collectable.bits;
        const ObjectEx = Self::Object.bits | Self::RefCounted.bits | Self::Collectable.bits;
        const ResourceEx = Self::Resource.bits | Self::RefCounted.bits;
        const ReferenceEx = Self::Reference.bits | Self::RefCounted.bits;
        const ConstantAstEx = Self::ConstantExpression.bits | Self::RefCounted.bits;

        const RefCounted = (IS_TYPE_REFCOUNTED << Z_TYPE_FLAGS_SHIFT);
        const Collectable = (IS_TYPE_COLLECTABLE << Z_TYPE_FLAGS_SHIFT);
    }
}

bitflags! {
    /// Flags for building classes.
    pub struct ClassFlags: u32 {
        const Final = ZEND_ACC_FINAL;
        const Abstract = ZEND_ACC_ABSTRACT;
        const Immutable = ZEND_ACC_IMMUTABLE;
        const HasTypeHints = ZEND_ACC_HAS_TYPE_HINTS;
        const TopLevel = ZEND_ACC_TOP_LEVEL;
        const Preloaded = ZEND_ACC_PRELOADED;

        const Interface = ZEND_ACC_INTERFACE;
        const Trait = ZEND_ACC_TRAIT;
        const AnonymousClass = ZEND_ACC_ANON_CLASS;
        const Linked = ZEND_ACC_LINKED;
        const ImplicitAbstractClass = ZEND_ACC_IMPLICIT_ABSTRACT_CLASS;
        const UseGuards = ZEND_ACC_USE_GUARDS;
        const ConstantsUpdated = ZEND_ACC_CONSTANTS_UPDATED;
        const NoDynamicProperties = ZEND_ACC_NO_DYNAMIC_PROPERTIES;
        const HasStaticInMethods = ZEND_HAS_STATIC_IN_METHODS;
        #[cfg(not(php82))]
        const ReuseGetIterator = ZEND_ACC_REUSE_GET_ITERATOR;
        const ResolvedParent = ZEND_ACC_RESOLVED_PARENT;
        const ResolvedInterfaces = ZEND_ACC_RESOLVED_INTERFACES;
        const UnresolvedVariance = ZEND_ACC_UNRESOLVED_VARIANCE;
        const NearlyLinked = ZEND_ACC_NEARLY_LINKED;

        #[cfg(any(php81,php82))]
        const NotSerializable = crate::ffi::ZEND_ACC_NOT_SERIALIZABLE;
    }
}

bitflags! {
    /// Flags for building methods.
    pub struct MethodFlags: u32 {
        const Public = ZEND_ACC_PUBLIC;
        const Protected = ZEND_ACC_PROTECTED;
        const Private = ZEND_ACC_PRIVATE;
        const Changed = ZEND_ACC_CHANGED;
        const Static = ZEND_ACC_STATIC;
        const Final = ZEND_ACC_FINAL;
        const Abstract = ZEND_ACC_ABSTRACT;
        const Immutable = ZEND_ACC_IMMUTABLE;
        const HasTypeHints = ZEND_ACC_HAS_TYPE_HINTS;
        const TopLevel = ZEND_ACC_TOP_LEVEL;
        const Preloaded = ZEND_ACC_PRELOADED;

        const Deprecated = ZEND_ACC_DEPRECATED;
        const ReturnReference = ZEND_ACC_RETURN_REFERENCE;
        const HasReturnType = ZEND_ACC_HAS_RETURN_TYPE;
        const Variadic = ZEND_ACC_VARIADIC;
        const HasFinallyBlock = ZEND_ACC_HAS_FINALLY_BLOCK;
        const EarlyBinding = ZEND_ACC_EARLY_BINDING;
        const UsesThis = ZEND_ACC_USES_THIS;
        const CallViaTrampoline = ZEND_ACC_CALL_VIA_TRAMPOLINE;
        const NeverCache = ZEND_ACC_NEVER_CACHE;
        const TraitClone = ZEND_ACC_TRAIT_CLONE;
        const IsConstructor = ZEND_ACC_CTOR;
        const Closure = ZEND_ACC_CLOSURE;
        const FakeClosure = ZEND_ACC_FAKE_CLOSURE;
        const Generator = ZEND_ACC_GENERATOR;
        const DonePassTwo = ZEND_ACC_DONE_PASS_TWO;
        const HeapRTCache = ZEND_ACC_HEAP_RT_CACHE;
        const StrictTypes = ZEND_ACC_STRICT_TYPES;
    }
}

bitflags! {
    /// Flags for building properties.
    pub struct PropertyFlags: u32 {
        const Public = ZEND_ACC_PUBLIC;
        const Protected = ZEND_ACC_PROTECTED;
        const Private = ZEND_ACC_PRIVATE;
        const Changed = ZEND_ACC_CHANGED;
        const Static = ZEND_ACC_STATIC;
        const Promoted = ZEND_ACC_PROMOTED;
    }
}

bitflags! {
    /// Flags for building constants.
    pub struct ConstantFlags: u32 {
        const Public = ZEND_ACC_PUBLIC;
        const Protected = ZEND_ACC_PROTECTED;
        const Private = ZEND_ACC_PRIVATE;
        const Promoted = ZEND_ACC_PROMOTED;
    }
}

bitflags! {
    /// Flags for building module global constants.
    pub struct GlobalConstantFlags: u32 {
        const CaseSensitive = CONST_CS;
        const Persistent = CONST_PERSISTENT;
        const NoFileCache = CONST_NO_FILE_CACHE;
        const Deprecated = CONST_DEPRECATED;
    }
}

bitflags! {
    /// Represents the result of a function.
    pub struct ZendResult: i32 {
        const Success = 0;
        const Failure = -1;
    }
}

/// Valid data types for PHP.
#[repr(C, u8)]
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
        contains!(IS_PTR, Ptr);
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
    use crate::ffi::{
        IS_ARRAY, IS_ARRAY_EX, IS_CALLABLE, IS_CONSTANT_AST, IS_CONSTANT_AST_EX, IS_DOUBLE,
        IS_FALSE, IS_INTERNED_STRING_EX, IS_LONG, IS_NULL, IS_OBJECT, IS_OBJECT_EX, IS_PTR,
        IS_REFERENCE, IS_REFERENCE_EX, IS_RESOURCE, IS_RESOURCE_EX, IS_STRING, IS_STRING_EX,
        IS_TRUE, IS_UNDEF, IS_VOID,
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
        test!(IS_PTR, Ptr);

        test!(IS_INTERNED_STRING_EX, String);
        test!(IS_STRING_EX, String);
        test!(IS_ARRAY_EX, Array);
        assert_eq!(DataType::try_from(IS_OBJECT_EX), Ok(DataType::Object(None)));
        test!(IS_RESOURCE_EX, Resource);
        test!(IS_REFERENCE_EX, Reference);
        test!(IS_CONSTANT_AST_EX, ConstantExpression);
    }
}
