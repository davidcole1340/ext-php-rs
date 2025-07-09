use std::{ffi::CString, ptr};

use crate::{
    builders::FunctionBuilder,
    describe::DocComments,
    enum_::EnumCase,
    error::Result,
    ffi::{zend_enum_add_case, zend_register_internal_enum},
    flags::{DataType, MethodFlags},
    types::{ZendStr, Zval},
    zend::{ClassEntry, FunctionEntry},
};

/// A builder for PHP enums.
#[must_use]
pub struct EnumBuilder {
    pub(crate) name: String,
    pub(crate) methods: Vec<(FunctionBuilder<'static>, MethodFlags)>,
    pub(crate) cases: Vec<&'static EnumCase>,
    pub(crate) datatype: DataType,
    register: Option<fn(&'static mut ClassEntry)>,
    pub(crate) docs: DocComments,
}

impl EnumBuilder {
    /// Creates a new enum builder with the given name.
    pub fn new<T: Into<String>>(name: T) -> Self {
        Self {
            name: name.into(),
            methods: Vec::default(),
            cases: Vec::default(),
            datatype: DataType::Undef,
            register: None,
            docs: DocComments::default(),
        }
    }

    /// Adds an enum case to the enum.
    ///
    /// # Panics
    ///
    /// If the case's data type does not match the enum's data type
    pub fn case(mut self, case: &'static EnumCase) -> Self {
        let data_type = case.data_type();
        assert!(
            data_type == self.datatype || self.cases.is_empty(),
            "Cannot add case with data type {:?} to enum with data type {:?}",
            data_type,
            self.datatype
        );

        self.datatype = data_type;
        self.cases.push(case);

        self
    }

    /// Adds a method to the enum.
    pub fn method(mut self, method: FunctionBuilder<'static>, flags: MethodFlags) -> Self {
        self.methods.push((method, flags));
        self
    }

    /// Function to register the class with PHP. This function is called after
    /// the class is built.
    ///
    /// # Parameters
    ///
    /// * `register` - The function to call to register the class.
    pub fn registration(mut self, register: fn(&'static mut ClassEntry)) -> Self {
        self.register = Some(register);
        self
    }

    /// Add documentation comments to the enum.
    pub fn docs(mut self, docs: DocComments) -> Self {
        self.docs = docs;
        self
    }

    /// Registers the enum with PHP.
    ///
    /// # Panics
    ///
    /// If the registration function was not set prior to calling this
    /// method.
    ///
    /// # Errors
    ///
    /// If the enum could not be registered, e.g. due to an invalid name or
    /// data type.
    pub fn register(self) -> Result<()> {
        let mut methods = self
            .methods
            .into_iter()
            .map(|(method, flags)| {
                method.build().map(|mut method| {
                    method.flags |= flags.bits();
                    method
                })
            })
            .collect::<Result<Vec<_>>>()?;
        methods.push(FunctionEntry::end());

        let class = unsafe {
            zend_register_internal_enum(
                CString::new(self.name)?.as_ptr(),
                self.datatype.as_u32().try_into()?,
                methods.into_boxed_slice().as_ptr(),
            )
        };

        for case in self.cases {
            let name = ZendStr::new_interned(case.name, true);
            let value = match &case.discriminant {
                Some(value) => {
                    let value: Zval = value.try_into()?;
                    let mut zv = core::mem::ManuallyDrop::new(value);
                    (&raw mut zv).cast()
                }
                None => ptr::null_mut(),
            };
            unsafe {
                zend_enum_add_case(class, name.into_raw(), value);
            }
        }

        if let Some(register) = self.register {
            register(unsafe { &mut *class });
        } else {
            panic!("Enum was not registered with a registration function",);
        }

        Ok(())
    }
}
