use std::{ffi::c_void, ptr};

use crate::{
    bindings::{
        zend_long, zend_string, zend_strpprintf, zend_type, GC_FLAGS_MASK, GC_FLAGS_SHIFT,
        GC_INFO_SHIFT, IS_MIXED, IS_STR_INTERNED, MAY_BE_ANY, MAY_BE_BOOL, _IS_BOOL,
        _ZEND_IS_VARIADIC_BIT, _ZEND_SEND_MODE_SHIFT, _ZEND_TYPE_NULLABLE_BIT,
    },
    functions::c_str,
};

use super::enums::DataType;

/// Internal Zend type.
pub type ZendType = zend_type;

impl ZendType {
    /// Builds an empty Zend type container.
    ///
    /// # Parameters
    ///
    /// * `pass_by_ref` - Whether the value should be passed by reference.
    /// * `is_variadic` - Whether this type represents a variadic argument.
    pub fn empty(pass_by_ref: bool, is_variadic: bool) -> Self {
        Self {
            ptr: ptr::null::<c_void>() as *mut c_void,
            type_mask: Self::arg_info_flags(pass_by_ref, is_variadic),
        }
    }

    pub fn empty_from_type(
        type_: DataType,
        pass_by_ref: bool,
        is_variadic: bool,
        allow_null: bool,
    ) -> Self {
        Self {
            ptr: ptr::null::<c_void>() as *mut c_void,
            type_mask: Self::type_init_code(type_, pass_by_ref, is_variadic, allow_null),
        }
    }

    /// Calculates the internal flags of the type.
    /// Translation of of the `_ZEND_ARG_INFO_FLAGS` macro from zend_API.h:110.
    ///
    /// # Parameters
    ///
    /// * `pass_by_ref` - Whether the value should be passed by reference.
    /// * `is_variadic` - Whether this type represents a variadic argument.
    pub(crate) fn arg_info_flags(pass_by_ref: bool, is_variadic: bool) -> u32 {
        ((pass_by_ref as u32) << _ZEND_SEND_MODE_SHIFT)
            | (if is_variadic {
                _ZEND_IS_VARIADIC_BIT
            } else {
                0
            })
    }

    /// Calculates the internal flags of the type.
    /// Translation of the `ZEND_TYPE_INIT_CODE` macro from zend_API.h:163.
    ///
    /// # Parameters
    ///
    /// * `type_` - The type to initialize the Zend type with.
    /// * `pass_by_ref` - Whether the value should be passed by reference.
    /// * `is_variadic` - Whether this type represents a variadic argument.
    /// * `allow_null` - Whether the value can be null.
    pub(crate) fn type_init_code(
        type_: DataType,
        pass_by_ref: bool,
        is_variadic: bool,
        allow_null: bool,
    ) -> u32 {
        let type_ = type_ as u32;

        (if type_ == _IS_BOOL {
            MAY_BE_BOOL
        } else {
            if type_ == IS_MIXED {
                MAY_BE_ANY
            } else {
                1 << type_
            }
        }) | (if allow_null {
            _ZEND_TYPE_NULLABLE_BIT
        } else {
            0
        }) | Self::arg_info_flags(pass_by_ref, is_variadic)
    }
}

/// Internal identifier used for a long.
/// The size depends on the system architecture. On 32-bit systems,
/// a ZendLong is 32-bits, while on a 64-bit system it is 64-bits.
pub type ZendLong = zend_long;

/// String type used in the Zend internals.
/// The actual size of the 'string' differs, as the
/// end of this struct is only 1 char long, but the length
/// inside the struct defines how many characters are in the string.
pub type ZendString = zend_string;

impl ZendString {
    /// Creates a new Zend string.
    ///
    /// Note that this returns a raw pointer, and will not be freed by
    /// Rust.
    ///
    /// # Parameters
    ///
    /// * `str_` - The string to create a Zend string from.
    pub fn new<S>(str_: S) -> *mut Self
    where
        S: AsRef<str>,
    {
        let str_ = str_.as_ref();
        unsafe { zend_strpprintf(str_.len() as u64, c_str(str_)) }
    }

    /// Translation of the `ZSTR_IS_INTERNED` macro.
    /// zend_string.h:76
    pub(crate) unsafe fn is_interned(&self) -> bool {
        (((self.gc.u.type_info >> GC_INFO_SHIFT) & (GC_FLAGS_MASK >> GC_FLAGS_SHIFT))
            & IS_STR_INTERNED)
            != 0
    }
}
