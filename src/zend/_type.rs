use std::{ffi::c_void, ptr};

use crate::{
    ffi::{
        zend_type, IS_MIXED, MAY_BE_ANY, MAY_BE_BOOL, _IS_BOOL, _ZEND_IS_VARIADIC_BIT,
        _ZEND_SEND_MODE_SHIFT, _ZEND_TYPE_NAME_BIT, _ZEND_TYPE_NULLABLE_BIT,
    },
    flags::DataType,
    types::ZendStr,
};

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

    /// Attempts to create a zend type for a given datatype. Returns an option
    /// containing the type.
    ///
    /// Returns [`None`] if the data type was a class object where the class
    /// name could not be converted into a C string (i.e. contained
    /// NUL-bytes).
    ///
    /// # Parameters
    ///
    /// * `type_` - Data type to create zend type for.
    /// * `pass_by_ref` - Whether the type should be passed by reference.
    /// * `is_variadic` - Whether the type is for a variadic argument.
    /// * `allow_null` - Whether the type should allow null to be passed in
    ///   place.
    pub fn empty_from_type(
        type_: DataType,
        pass_by_ref: bool,
        is_variadic: bool,
        allow_null: bool,
    ) -> Option<Self> {
        match type_ {
            DataType::Object(Some(class)) => {
                Self::empty_from_class_type(class, pass_by_ref, is_variadic, allow_null)
            }
            type_ => Some(Self::empty_from_primitive_type(
                type_,
                pass_by_ref,
                is_variadic,
                allow_null,
            )),
        }
    }

    /// Attempts to create a zend type for a class object type. Returns an
    /// option containing the type if successful.
    ///
    /// Returns [`None`] if the data type was a class object where the class
    /// name could not be converted into a C string (i.e. contained
    /// NUL-bytes).
    ///
    /// # Parameters
    ///
    /// * `class_name` - Name of the class parameter.
    /// * `pass_by_ref` - Whether the type should be passed by reference.
    /// * `is_variadic` - Whether the type is for a variadic argument.
    /// * `allow_null` - Whether the type should allow null to be passed in
    ///   place.
    fn empty_from_class_type(
        class_name: &str,
        pass_by_ref: bool,
        is_variadic: bool,
        allow_null: bool,
    ) -> Option<Self> {
        Some(Self {
            ptr: ZendStr::new(class_name, true).into_raw().as_ptr() as *mut c_void,
            type_mask: _ZEND_TYPE_NAME_BIT
                | (if allow_null {
                    _ZEND_TYPE_NULLABLE_BIT
                } else {
                    0
                })
                | Self::arg_info_flags(pass_by_ref, is_variadic),
        })
    }

    /// Attempts to create a zend type for a primitive PHP type.
    ///
    /// # Parameters
    ///
    /// * `type_` - Data type to create zend type for.
    /// * `pass_by_ref` - Whether the type should be passed by reference.
    /// * `is_variadic` - Whether the type is for a variadic argument.
    /// * `allow_null` - Whether the type should allow null to be passed in
    ///   place.
    ///
    /// # Panics
    ///
    /// Panics if the given `type_` is for a class object type.
    fn empty_from_primitive_type(
        type_: DataType,
        pass_by_ref: bool,
        is_variadic: bool,
        allow_null: bool,
    ) -> Self {
        assert!(!matches!(type_, DataType::Object(Some(_))));
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
        let type_ = type_.as_u32();

        (if type_ == _IS_BOOL {
            MAY_BE_BOOL
        } else if type_ == IS_MIXED {
            MAY_BE_ANY
        } else {
            1 << type_
        }) | (if allow_null {
            _ZEND_TYPE_NULLABLE_BIT
        } else {
            0
        }) | Self::arg_info_flags(pass_by_ref, is_variadic)
    }
}
