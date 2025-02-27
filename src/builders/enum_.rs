use std::{ffi::CString, ptr};

use crate::{
    convert::IntoZval,
    error::{Error, Result},
    ffi::{zend_enum_add_case, zend_register_internal_enum},
    flags::{DataType, MethodFlags},
    types::ZendStr,
    zend::{ClassEntry, FunctionEntry},
};

/// Builder for registering an enum in PHP.
pub struct EnumBuilder<T: IntoZval> {
    name: String,
    methods: Vec<FunctionEntry>,
    type_: DataType,
    cases: Vec<EnumBuilderCase<T>>,
}

impl<T: IntoZval> EnumBuilder<T> {
    /// Creates a new enum builder, used to build enums
    /// to be exported to PHP.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the class.
    pub fn new<N: Into<String>>(name: N, _type: DataType) -> Self {
        Self {
            name: name.into(),
            methods: vec![],
            type_: _type,
            cases: vec![],
        }
    }

    /// Adds a method to the class.
    ///
    /// # Parameters
    ///
    /// * `func` - The function entry to add to the class.
    /// * `flags` - Flags relating to the function. See [`MethodFlags`].
    pub fn method(mut self, mut func: FunctionEntry, flags: MethodFlags) -> Self {
        func.flags |= flags.bits();
        self.methods.push(func);
        self
    }

    /// Add a new case to the enum.
    pub fn case(mut self, case: EnumBuilderCase<T>) -> Self {
        self.cases.push(case);
        self
    }

    /// Builds the enum, returning a reference to the class entry.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] variant if the class could not be registered.
    pub fn build(mut self) -> Result<&'static mut ClassEntry> {
        self.methods.push(FunctionEntry::end());
        let func = Box::into_raw(self.methods.into_boxed_slice()) as *const FunctionEntry;

        let class = unsafe {
            zend_register_internal_enum(
                CString::new(self.name.as_str())?.as_ptr(),
                self.type_.as_u32() as _,
                func,
            )
            .as_mut()
            .ok_or(Error::InvalidPointer)?
        };

        for case in self.cases {
            let name = ZendStr::new(&case.name, true);
            let value = match case.value {
                Some(value) => {
                    let zval = value.into_zval(true)?;
                    let mut zv = core::mem::ManuallyDrop::new(zval);
                    core::ptr::addr_of_mut!(zv).cast()
                }
                None => ptr::null_mut(),
            };
            unsafe {
                zend_enum_add_case(class, name.into_raw(), value);
            }
        }

        Ok(class)
    }
}

pub struct EnumBuilderCase<T> {
    pub name: String,
    pub value: Option<T>,
}
