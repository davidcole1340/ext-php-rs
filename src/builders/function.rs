use crate::{
    args::{Arg, ArgInfo},
    error::{Error, Result},
    flags::DataType,
    types::Zval,
    zend::{ExecuteData, FunctionEntry, ZendType},
};
use std::{ffi::CString, mem, ptr};

/// Function representation in Rust.
#[cfg(not(windows))]
pub type FunctionHandler = extern "C" fn(execute_data: &mut ExecuteData, retval: &mut Zval);
#[cfg(windows)]
pub type FunctionHandler =
    extern "vectorcall" fn(execute_data: &mut ExecuteData, retval: &mut Zval);

/// Function representation in Rust using pointers.
#[cfg(not(windows))]
type FunctionPointerHandler = extern "C" fn(execute_data: *mut ExecuteData, retval: *mut Zval);
#[cfg(windows)]
type FunctionPointerHandler =
    extern "vectorcall" fn(execute_data: *mut ExecuteData, retval: *mut Zval);

/// Builder for registering a function in PHP.
#[derive(Debug)]
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
    /// * `handler` - The handler to be called when the function is invoked from
    ///   PHP.
    pub fn new<T: Into<String>>(name: T, handler: FunctionHandler) -> Self {
        Self {
            name: name.into(),
            function: FunctionEntry {
                fname: ptr::null(),
                // SAFETY: `*mut T` and `&mut T` have the same ABI as long as `*mut T` is non-null,
                // aligned and pointing to a `T`. PHP guarantees that these conditions will be met.
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
    /// * `handler` - The handler to be called when the function is invoked from
    ///   PHP.
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
    /// * `as_ref` - Whether the function returns a reference.
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
            name: self.n_req.unwrap_or(self.args.len()) as *const _,
            type_: match self.retval {
                Some(retval) => {
                    ZendType::empty_from_type(retval, self.ret_as_ref, false, self.ret_as_null)
                        .ok_or(Error::InvalidCString)?
                }
                None => ZendType::empty(false, false),
            },
            default_value: ptr::null(),
        });

        // arguments
        args.extend(
            self.args
                .iter()
                .map(|arg| arg.as_arg_info())
                .collect::<Result<Vec<_>>>()?,
        );

        self.function.fname = CString::new(self.name)?.into_raw();
        self.function.num_args = (args.len() - 1) as u32;
        self.function.arg_info = Box::into_raw(args.into_boxed_slice()) as *const ArgInfo;

        Ok(self.function)
    }
}
