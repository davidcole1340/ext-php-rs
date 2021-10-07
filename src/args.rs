//! Builder and objects relating to function and method arguments.

use std::{ffi::CString, ptr};

use crate::{
    convert::{FromZvalMut, IntoZvalDyn},
    error::{Error, Result},
    ffi::{
        _zend_expected_type, _zend_expected_type_Z_EXPECTED_ARRAY,
        _zend_expected_type_Z_EXPECTED_BOOL, _zend_expected_type_Z_EXPECTED_DOUBLE,
        _zend_expected_type_Z_EXPECTED_LONG, _zend_expected_type_Z_EXPECTED_OBJECT,
        _zend_expected_type_Z_EXPECTED_RESOURCE, _zend_expected_type_Z_EXPECTED_STRING,
        zend_internal_arg_info, zend_wrong_parameters_count_error,
    },
    flags::DataType,
    types::Zval,
    zend::ZendType,
};

/// Represents an argument to a function.
#[derive(Debug)]
pub struct Arg<'a> {
    name: String,
    _type: DataType,
    as_ref: bool,
    allow_null: bool,
    variadic: bool,
    default_value: Option<String>,
    zval: Option<&'a mut Zval>,
}

impl<'a> Arg<'a> {
    /// Creates a new argument.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the parameter.
    /// * `_type` - The type of the parameter.
    pub fn new<T: Into<String>>(name: T, _type: DataType) -> Self {
        Arg {
            name: name.into(),
            _type,
            as_ref: false,
            allow_null: false,
            variadic: false,
            default_value: None,
            zval: None,
        }
    }

    /// Sets the argument as a reference.
    #[allow(clippy::wrong_self_convention)]
    pub fn as_ref(mut self) -> Self {
        self.as_ref = true;
        self
    }

    /// Sets the argument as variadic.
    pub fn is_variadic(mut self) -> Self {
        self.variadic = true;
        self
    }

    /// Sets the argument as nullable.
    pub fn allow_null(mut self) -> Self {
        self.allow_null = true;
        self
    }

    /// Sets the default value for the argument.
    pub fn default<T: Into<String>>(mut self, default: T) -> Self {
        self.default_value = Some(default.into());
        self
    }

    /// Attempts to consume the argument, converting the inner type into `T`.
    /// Upon success, the result is returned in a [`Result`].
    ///
    /// If the conversion fails (or the argument contains no value), the
    /// argument is returned in an [`Err`] variant.
    ///
    /// As this function consumes, it cannot return a reference to the
    /// underlying zval.
    pub fn consume<T>(mut self) -> Result<T, Self>
    where
        for<'b> T: FromZvalMut<'b>,
    {
        self.zval
            .as_mut()
            .and_then(|zv| T::from_zval_mut(zv))
            .ok_or(self)
    }

    /// Attempts to retrieve the value of the argument.
    /// This will be None until the ArgParser is used to parse
    /// the arguments.
    pub fn val<T>(&'a mut self) -> Option<T>
    where
        T: FromZvalMut<'a>,
    {
        self.zval.as_mut().and_then(|zv| T::from_zval_mut(zv))
    }

    /// Attempts to return a reference to the arguments internal Zval.
    ///
    /// # Returns
    ///
    /// * `Some(&Zval)` - The internal zval.
    /// * `None` - The argument was empty.
    pub fn zval(&mut self) -> Option<&mut &'a mut Zval> {
        self.zval.as_mut()
    }

    /// Attempts to call the argument as a callable with a list of arguments to
    /// pass to the function. Note that a thrown exception inside the
    /// callable is not detectable, therefore you should check if the return
    /// value is valid rather than unwrapping. Returns a result containing the
    /// return value of the function, or an error.
    ///
    /// You should not call this function directly, rather through the
    /// [`call_user_func`] macro.
    ///
    /// # Parameters
    ///
    /// * `params` - A list of parameters to call the function with.
    pub fn try_call(&self, params: Vec<&dyn IntoZvalDyn>) -> Result<Zval> {
        self.zval.as_ref().ok_or(Error::Callable)?.try_call(params)
    }

    /// Returns the internal PHP argument info.
    pub(crate) fn as_arg_info(&self) -> Result<ArgInfo> {
        Ok(ArgInfo {
            name: CString::new(self.name.as_str())?.into_raw(),
            type_: ZendType::empty_from_type(
                self._type,
                self.as_ref,
                self.variadic,
                self.allow_null,
            )
            .ok_or(Error::InvalidCString)?,
            default_value: match &self.default_value {
                Some(val) => CString::new(val.as_str())?.into_raw(),
                None => ptr::null(),
            },
        })
    }
}

impl From<Arg<'_>> for _zend_expected_type {
    fn from(arg: Arg) -> Self {
        let err = match arg._type {
            DataType::False | DataType::True => _zend_expected_type_Z_EXPECTED_BOOL,
            DataType::Long => _zend_expected_type_Z_EXPECTED_LONG,
            DataType::Double => _zend_expected_type_Z_EXPECTED_DOUBLE,
            DataType::String => _zend_expected_type_Z_EXPECTED_STRING,
            DataType::Array => _zend_expected_type_Z_EXPECTED_ARRAY,
            DataType::Object(_) => _zend_expected_type_Z_EXPECTED_OBJECT,
            DataType::Resource => _zend_expected_type_Z_EXPECTED_RESOURCE,
            _ => unreachable!(),
        };

        if arg.allow_null {
            err + 1
        } else {
            err
        }
    }
}

/// Internal argument information used by Zend.
pub type ArgInfo = zend_internal_arg_info;

/// Parses the arguments of a function.
pub struct ArgParser<'a, 'b> {
    args: Vec<&'b mut Arg<'a>>,
    min_num_args: Option<usize>,
    arg_zvals: Vec<Option<&'a mut Zval>>,
}

impl<'a, 'b> ArgParser<'a, 'b> {
    /// Builds a new function argument parser.
    pub fn new(arg_zvals: Vec<Option<&'a mut Zval>>) -> Self {
        ArgParser {
            args: vec![],
            min_num_args: None,
            arg_zvals,
        }
    }

    /// Adds a new argument to the parser.
    ///
    /// # Parameters
    ///
    /// * `arg` - The argument to add to the parser.
    pub fn arg(mut self, arg: &'b mut Arg<'a>) -> Self {
        self.args.push(arg);
        self
    }

    /// Sets the next arguments to be added as not required.
    pub fn not_required(mut self) -> Self {
        self.min_num_args = Some(self.args.len());
        self
    }

    /// Uses the argument parser to parse the arguments contained in the given
    /// `ExecutionData` object. Returns successfully if the arguments were
    /// parsed.
    ///
    /// This function can only be safely called from within an exported PHP
    /// function.
    ///
    /// # Parameters
    ///
    /// * `execute_data` - The execution data from the function.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] type if there were too many or too little arguments
    /// passed to the function. The user has already been notified so you
    /// should break execution after seeing an error type.
    pub fn parse(mut self) -> Result<()> {
        let max_num_args = self.args.len();
        let min_num_args = self.min_num_args.unwrap_or(max_num_args);
        let num_args = self.arg_zvals.len();

        if num_args < min_num_args || num_args > max_num_args {
            // SAFETY: Exported C function is safe, return value is unused and parameters
            // are copied.
            unsafe { zend_wrong_parameters_count_error(min_num_args as _, max_num_args as _) };
            return Err(Error::IncorrectArguments(num_args, min_num_args));
        }

        for (i, arg_zval) in self.arg_zvals.into_iter().enumerate() {
            if let Some(arg) = self.args.get_mut(i) {
                arg.zval = arg_zval;
            }
        }

        Ok(())
    }
}
