use std::{ffi::CString, ptr};

use crate::{
    builders::FunctionBuilder,
    enum_::EnumCase,
    error::Result,
    ffi::{zend_enum_add_case, zend_register_internal_enum},
    flags::{DataType, MethodFlags},
    types::{ZendStr, Zval},
    zend::FunctionEntry,
};

#[must_use]
pub struct EnumBuilder {
    pub(crate) name: String,
    pub(crate) methods: Vec<(FunctionBuilder<'static>, MethodFlags)>,
    pub(crate) cases: Vec<&'static EnumCase>,
    pub(crate) datatype: DataType,
}

impl EnumBuilder {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Self {
            name: name.into(),
            methods: Vec::default(),
            cases: Vec::default(),
            datatype: DataType::Undef,
        }
    }

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

    pub fn add_method(mut self, method: FunctionBuilder<'static>, flags: MethodFlags) -> Self {
        self.methods.push((method, flags));
        self
    }

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
            let name = ZendStr::new(case.name, true);
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

        Ok(())
    }
}
