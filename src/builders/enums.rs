//! Builders for registering enums with PHP. Only valid for PHP 8.1.

use std::ffi::CString;

use crate::{
    convert::IntoZval,
    error::Result,
    ffi::{zend_enum_add_case_cstr, zend_register_internal_enum},
    flags::DataType,
    zend::ClassEntry,
};

/// Builder for an unbacked enum.
pub struct UnbackedEnumBuilder<'a> {
    name: &'a str,
    cases: Vec<&'a str>,
}

impl<'a> UnbackedEnumBuilder<'a> {
    /// Creates a new unbacked enum builder.
    ///
    /// An unbacked enum has no value associated with each case. It can only be
    /// identified by the enum case name. See [`BackedEnumBuilder`] for enums
    /// with backing.
    ///
    /// # Parameters
    ///
    /// * `name` - Name of the enum.
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            cases: vec![],
        }
    }

    /// Adds a case to the enum.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the enum case.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ext_php_rs::builders::UnbackedEnumBuilder;
    ///
    /// # fn test() -> Result<(), Box<dyn std::error::Error>> {
    /// UnbackedEnumBuilder::new("TestEnum")
    ///     .case("ExampleCase")
    ///     .build()?;
    /// // TestEnum::ExampleCase
    /// # Ok(())
    /// # }
    /// ```
    pub fn case(mut self, name: &'a str) -> Self {
        self.cases.push(name);
        self
    }

    /// Builds the enum, registering it with PHP. Returns a reference to the
    /// registered class entry in a result, or an error if the registration
    /// fails.
    pub fn build(self) -> Result<&'static mut ClassEntry> {
        let name = CString::new(self.name)?;

        let ce = unsafe {
            zend_register_internal_enum(
                name.as_ptr(),
                DataType::Undef.as_u32() as u8,
                std::ptr::null(),
            )
            .as_mut()
        }
        .expect("Failed to allocate for enum class object");

        for case in self.cases {
            let name = CString::new(case)?;
            unsafe { zend_enum_add_case_cstr(ce, name.as_ptr(), std::ptr::null_mut()) };
        }

        Ok(ce)
    }
}

/// Implemented on types which can be used to 'back' an enum.
///
/// This doesn't actually have any methods but is rather a marker for the types
/// that are valid for being a backer. All functions come from the [`IntoZval`]
/// requirement.
pub trait EnumBacking: IntoZval {}

macro_rules! backing {
    ($($t: ty),*) => {
        $(impl EnumBacking for $t {})*
    };
}

impl<'a> EnumBacking for &'a str {}
backing!(String, i8, i16, i32, i64, u8, u16, u32, u64, isize, usize);

/// Builder for an enum backed by a string or long.
pub struct BackedEnumBuilder<'a, T: EnumBacking> {
    name: &'a str,
    cases: Vec<(&'a str, T)>,
}

impl<'a, T: EnumBacking> BackedEnumBuilder<'a, T> {
    /// Creates a new backed enum builder.
    ///
    /// A backed enum has a value associated with each case. This can be used to
    /// convert to and from other primitive PHP types like longs and strings.
    /// See [`UnbackedEnumBuilder`] for enums without backing.
    ///
    /// # Parameters
    ///
    /// * `name` - Name of the enum.
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            cases: vec![],
        }
    }

    /// Adds a case to the enum.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the enum case.
    /// * `val` - The value associated with the enum.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ext_php_rs::builders::BackedEnumBuilder;
    ///
    /// # fn test() -> Result<(), Box<dyn std::error::Error>> {
    /// BackedEnumBuilder::new("TestEnum")
    ///     .case("ExampleCase", 5)
    ///     .build()?;
    /// // TestEnum::ExampleCase->value == 5
    ///
    /// BackedEnumBuilder::new("StringBacked")
    ///     .case("ExampleCase", "Hello")
    ///     .build()?;
    /// // StringBacked::ExampleCase->value == "Hello"
    /// # Ok(())
    /// # }
    /// ```
    pub fn case(mut self, name: &'a str, val: T) -> Self {
        self.cases.push((name, val));
        self
    }

    /// Builds the enum, registering it with PHP. Returns a reference to the
    /// registered class entry in a result, or an error if the registration
    /// fails.
    ///
    /// # Errors
    ///
    /// * Enum name cannot be converted to a C string due to nul bytes.
    /// * Case name cannot be converted to a C string due to nul bytes.
    /// * Case value cannot be converted to a zval.
    pub fn build(self) -> Result<&'static mut ClassEntry> {
        let name = CString::new(self.name)?;

        let ce = unsafe {
            zend_register_internal_enum(name.as_ptr(), T::TYPE.as_u32() as u8, std::ptr::null())
                .as_mut()
        }
        .expect("Failed to allocate for enum class object");

        for (case, val) in self.cases {
            let name = CString::new(case)?;
            let mut val = val.into_zval(true)?;
            unsafe { zend_enum_add_case_cstr(ce, name.as_ptr(), &mut val) };
        }

        Ok(ce)
    }
}
