//! Bitflags used in PHP and the Zend engine.

use bitflags::bitflags;

use crate::bindings::{
    CONST_CS, CONST_DEPRECATED, CONST_NO_FILE_CACHE, CONST_PERSISTENT, IS_ARRAY, IS_CALLABLE,
    IS_CONSTANT_AST, IS_DOUBLE, IS_FALSE, IS_LONG, IS_NULL, IS_OBJECT, IS_PTR, IS_REFERENCE,
    IS_RESOURCE, IS_STRING, IS_TRUE, IS_TYPE_COLLECTABLE, IS_TYPE_REFCOUNTED, IS_UNDEF, IS_VOID,
    ZEND_ACC_ABSTRACT, ZEND_ACC_ANON_CLASS, ZEND_ACC_CALL_VIA_TRAMPOLINE, ZEND_ACC_CHANGED,
    ZEND_ACC_CLOSURE, ZEND_ACC_CONSTANTS_UPDATED, ZEND_ACC_CTOR, ZEND_ACC_DEPRECATED,
    ZEND_ACC_DONE_PASS_TWO, ZEND_ACC_EARLY_BINDING, ZEND_ACC_FAKE_CLOSURE, ZEND_ACC_FINAL,
    ZEND_ACC_GENERATOR, ZEND_ACC_HAS_FINALLY_BLOCK, ZEND_ACC_HAS_RETURN_TYPE,
    ZEND_ACC_HAS_TYPE_HINTS, ZEND_ACC_HAS_UNLINKED_USES, ZEND_ACC_HEAP_RT_CACHE,
    ZEND_ACC_IMMUTABLE, ZEND_ACC_IMPLICIT_ABSTRACT_CLASS, ZEND_ACC_INTERFACE, ZEND_ACC_LINKED,
    ZEND_ACC_NEARLY_LINKED, ZEND_ACC_NEVER_CACHE, ZEND_ACC_NO_DYNAMIC_PROPERTIES,
    ZEND_ACC_PRELOADED, ZEND_ACC_PRIVATE, ZEND_ACC_PROMOTED, ZEND_ACC_PROPERTY_TYPES_RESOLVED,
    ZEND_ACC_PROTECTED, ZEND_ACC_PUBLIC, ZEND_ACC_RESOLVED_INTERFACES, ZEND_ACC_RESOLVED_PARENT,
    ZEND_ACC_RETURN_REFERENCE, ZEND_ACC_REUSE_GET_ITERATOR, ZEND_ACC_STATIC, ZEND_ACC_STRICT_TYPES,
    ZEND_ACC_TOP_LEVEL, ZEND_ACC_TRAIT, ZEND_ACC_TRAIT_CLONE, ZEND_ACC_UNRESOLVED_VARIANCE,
    ZEND_ACC_USES_THIS, ZEND_ACC_USE_GUARDS, ZEND_ACC_VARIADIC, ZEND_HAS_STATIC_IN_METHODS,
    Z_TYPE_FLAGS_SHIFT,
};

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
        const PropertyTypesResolved = ZEND_ACC_PROPERTY_TYPES_RESOLVED;
        const ReuseGetIterator = ZEND_ACC_REUSE_GET_ITERATOR;
        const ResolvedParent = ZEND_ACC_RESOLVED_PARENT;
        const ResolvedInterfaces = ZEND_ACC_RESOLVED_INTERFACES;
        const UnresolvedVariance = ZEND_ACC_UNRESOLVED_VARIANCE;
        const NearlyLinked = ZEND_ACC_NEARLY_LINKED;
        const HasUnlinkedUses = ZEND_ACC_HAS_UNLINKED_USES;
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
