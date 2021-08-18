//! Types relating to registering constants in PHP.

use std::ffi::CString;

use super::flags::GlobalConstantFlags;
use crate::bindings::{
    zend_register_bool_constant, zend_register_double_constant, zend_register_long_constant,
    zend_register_string_constant,
};
use crate::errors::Result;

pub trait IntoConst: Sized {
    /// Registers a global module constant in PHP, with the value as the content of self.
    /// This function _must_ be called in the module startup function, which is called after
    /// the module is initialized. The second parameter of the startup function will be the
    /// module number. By default, the case-insensitive and persistent flags are set when
    /// registering the constant.
    ///
    /// Returns a result containing nothing if the constant was successfully registered.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the constant.
    /// * `module_number` - The module number that we are registering the constant under.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ext_php_rs::php::{constants::IntoConst, flags::ZendResult};
    ///
    /// pub extern "C" fn startup_function(_type: i32, module_number: i32) -> i32 {
    ///     5.register_constant("MY_CONST_NAME", module_number); // MY_CONST_NAME == 5
    ///     "Hello, world!".register_constant("STRING_CONSTANT", module_number); // STRING_CONSTANT == "Hello, world!"
    ///     0
    /// }
    /// ```
    fn register_constant(&self, name: &str, module_number: i32) -> Result<()> {
        self.register_constant_flags(
            name,
            module_number,
            GlobalConstantFlags::CaseSensitive | GlobalConstantFlags::Persistent,
        )
    }

    /// Registers a global module constant in PHP, with the value as the content of self.
    /// This function _must_ be called in the module startup function, which is called after
    /// the module is initialized. The second parameter of the startup function will be the
    /// module number. This function allows you to pass any extra flags in if you require.
    /// Note that the case-sensitive and persistent flags *are not* set when you use this function,
    /// you must set these yourself.
    ///
    /// Returns a result containing nothing if the constant was successfully registered.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the constant.
    /// * `module_number` - The module number that we are registering the constant under.
    /// * `flags` - Flags to register the constant with.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ext_php_rs::php::{constants::IntoConst, flags::{GlobalConstantFlags, ZendResult}};
    ///
    /// pub extern "C" fn startup_function(_type: i32, module_number: i32) -> i32 {
    ///     42.register_constant_flags("MY_CONST_NAME", module_number, GlobalConstantFlags::Persistent | GlobalConstantFlags::Deprecated);
    ///     0
    /// }
    /// ```
    fn register_constant_flags(
        &self,
        name: &str,
        module_number: i32,
        flags: GlobalConstantFlags,
    ) -> Result<()>;
}

impl IntoConst for String {
    fn register_constant_flags(
        &self,
        name: &str,
        module_number: i32,
        flags: GlobalConstantFlags,
    ) -> Result<()> {
        self.as_str()
            .register_constant_flags(name, module_number, flags)
    }
}

impl IntoConst for &str {
    fn register_constant_flags(
        &self,
        name: &str,
        module_number: i32,
        flags: GlobalConstantFlags,
    ) -> Result<()> {
        unsafe {
            zend_register_string_constant(
                CString::new(name)?.as_ptr(),
                name.len() as _,
                CString::new(*self)?.as_ptr(),
                flags.bits() as _,
                module_number,
            )
        };
        Ok(())
    }
}

impl IntoConst for bool {
    fn register_constant_flags(
        &self,
        name: &str,
        module_number: i32,
        flags: GlobalConstantFlags,
    ) -> Result<()> {
        unsafe {
            zend_register_bool_constant(
                CString::new(name)?.as_ptr(),
                name.len() as _,
                *self,
                flags.bits() as _,
                module_number,
            )
        };
        Ok(())
    }
}

/// Implements the `IntoConst` trait for a given number type using a given function.
macro_rules! into_const_num {
    ($type: ty, $fn: expr) => {
        impl IntoConst for $type {
            fn register_constant_flags(
                &self,
                name: &str,
                module_number: i32,
                flags: GlobalConstantFlags,
            ) -> Result<()> {
                Ok(unsafe {
                    $fn(
                        CString::new(name)?.as_ptr(),
                        name.len() as _,
                        *self as _,
                        flags.bits() as _,
                        module_number,
                    )
                })
            }
        }
    };
}

into_const_num!(i8, zend_register_long_constant);
into_const_num!(i16, zend_register_long_constant);
into_const_num!(i32, zend_register_long_constant);
into_const_num!(i64, zend_register_long_constant);
into_const_num!(f32, zend_register_double_constant);
into_const_num!(f64, zend_register_double_constant);
