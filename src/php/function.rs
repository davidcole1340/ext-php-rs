//! Builder and objects used to create functions and methods in PHP.

use std::{ffi::CString, mem, os::raw::c_char, ptr};

use crate::bindings::zend_function_entry;
use crate::errors::Result;

use super::{
    args::{Arg, ArgInfo},
    enums::DataType,
    execution_data::ExecutionData,
    types::zval::Zval,
    types::ZendType,
};

/// A Zend function entry. Alias.
pub type FunctionEntry = zend_function_entry;

impl FunctionEntry {
    /// Returns an empty function entry, signifing the end of a function list.
    pub fn end() -> Self {
        Self {
            fname: ptr::null() as *const c_char,
            handler: None,
            arg_info: ptr::null(),
            num_args: 0,
            flags: 0,
        }
    }

    /// Converts the function entry into a raw and  pointer, releasing it to the C world.
    pub fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }
}

/// Function representation in Rust.
pub type FunctionHandler = extern "C" fn(execute_data: &mut ExecutionData, retval: &mut Zval);

/// Function representation in Rust using pointers.
type FunctionPointerHandler = extern "C" fn(execute_data: *mut ExecutionData, retval: *mut Zval);

/// Builds a function to be exported as a PHP function.
#[derive(Debug, Clone)]
pub struct FunctionBuilder<'a> {
    name: String,
    function: FunctionEntry,
    args: Vec<Arg<'a>>,
    n_req: Option<usize>,
    retval: Option<DataType>,
    ret_as_ref: bool,
    ret_as_null: bool,
}

impl<'a> FunctionBuilder<'a> {
    /// Creates a new function builder, used to build functions
    /// to be exported to PHP.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the function.
    /// * `handler` - The handler to be called when the function is invoked from PHP.
    pub fn new<N>(name: N, handler: FunctionHandler) -> Self
    where
        N: AsRef<str>,
    {
        Self {
            name: name.as_ref().to_string(),
            function: FunctionEntry {
                fname: ptr::null(),
                handler: Some(unsafe {
                    mem::transmute::<FunctionHandler, FunctionPointerHandler>(handler)
                }),
                arg_info: ptr::null(),
                num_args: 0,
                flags: 0, // TBD?
            },
            args: vec![],
            n_req: None,
            retval: None,
            ret_as_ref: false,
            ret_as_null: false,
        }
    }

    /// Creates a constructor builder, used to build the constructor
    /// for classes.
    ///
    /// # Parameters
    ///
    /// * `handler` - The handler to be called when the function is invoked from PHP.
    pub fn constructor(handler: FunctionHandler) -> Self {
        Self::new("__construct", handler)
    }

    /// Adds an argument to the function.
    ///
    /// # Parameters
    ///
    /// * `arg` - The argument to add to the function.
    pub fn arg(mut self, arg: Arg<'a>) -> Self {
        self.args.push(arg);
        self
    }

    /// Sets the rest of the given arguments as not required.
    pub fn not_required(mut self) -> Self {
        self.n_req = Some(self.args.len());
        self
    }

    /// Sets the return value of the function.
    ///
    /// # Parameters
    ///
    /// * `type_` - The return type of the function.
    /// * `as_ref` - Whether the fucntion returns a reference.
    /// * `allow_null` - Whether the function return value is nullable.
    pub fn returns(mut self, type_: DataType, as_ref: bool, allow_null: bool) -> Self {
        self.retval = Some(type_);
        self.ret_as_ref = as_ref;
        self.ret_as_null = allow_null;
        self
    }

    /// Builds the function converting it into a Zend function entry.
    ///
    /// Returns a result containing the function entry if successful.
    pub fn build(mut self) -> Result<FunctionEntry> {
        let mut args = Vec::with_capacity(self.args.len() + 1);

        // argument header, retval etc
        args.push(ArgInfo {
            name: ((match self.n_req {
                Some(req) => req,
                None => self.args.len(),
            }) as libc::uintptr_t) as *const i8,
            type_: match self.retval {
                Some(retval) => {
                    ZendType::empty_from_type(retval, self.ret_as_ref, false, self.ret_as_null)
                }
                None => ZendType::empty(false, false),
            },
            default_value: ptr::null(),
        });

        // arguments
        for arg in self.args.iter() {
            args.push(ArgInfo {
                name: CString::new(arg.name.as_str())?.into_raw(),
                type_: ZendType::empty_from_type(arg._type, arg.as_ref, false, arg.allow_null),
                default_value: match &arg.default_value {
                    Some(val) => CString::new(val.as_str())?.into_raw(),
                    None => ptr::null(),
                },
            });
        }

        self.function.fname = CString::new(self.name)?.into_raw();
        self.function.num_args = (args.len() - 1) as u32;
        self.function.arg_info = Box::into_raw(args.into_boxed_slice()) as *const ArgInfo;

        Ok(self.function)
    }
}
