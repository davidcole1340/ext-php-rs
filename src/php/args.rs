//! Builder and objects relating to function and method arguments.

use std::convert::{TryFrom, TryInto};

use super::{
    enums::DataType,
    execution_data::ExecutionData,
    types::zval::{IntoZvalDyn, Zval},
};

use crate::{
    bindings::{
        _zend_expected_type, _zend_expected_type_Z_EXPECTED_ARRAY,
        _zend_expected_type_Z_EXPECTED_BOOL, _zend_expected_type_Z_EXPECTED_DOUBLE,
        _zend_expected_type_Z_EXPECTED_LONG, _zend_expected_type_Z_EXPECTED_OBJECT,
        _zend_expected_type_Z_EXPECTED_RESOURCE, _zend_expected_type_Z_EXPECTED_STRING,
        zend_internal_arg_info, zend_wrong_parameters_count_error,
    },
    errors::{Error, Result},
};

/// Represents an argument to a function.
#[derive(Debug, Clone)]
pub struct Arg<'a> {
    pub(crate) name: String,
    pub(crate) _type: DataType,
    pub(crate) as_ref: bool,
    pub(crate) allow_null: bool,
    pub(crate) default_value: Option<String>,
    pub(crate) zval: Option<&'a Zval>,
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

    /// Attempts to retrieve the value of the argument.
    /// This will be None until the ArgParser is used to parse
    /// the arguments.
    pub fn val<T: TryFrom<&'a Zval>>(&self) -> Option<T> {
        self.zval.and_then(|zv| zv.try_into().ok())
    }

    /// Attempts to return a reference to the arguments internal Zval.
    ///
    /// # Returns
    ///
    /// * `Some(&Zval)` - The internal zval.
    /// * `None` - The argument was empty.
    pub fn zval(&self) -> Option<&'a Zval> {
        self.zval
    }

    /// Attempts to call the argument as a callable with a list of arguments to pass to the function.
    /// Note that a thrown exception inside the callable is not detectable, therefore you should
    /// check if the return value is valid rather than unwrapping. Returns a result containing the
    /// return value of the function, or an error.
    ///
    /// You should not call this function directly, rather through the [`call_user_func`] macro.
    ///
    /// # Parameters
    ///
    /// * `params` - A list of parameters to call the function with.
    pub fn try_call(&self, params: Vec<&dyn IntoZvalDyn>) -> Result<Zval> {
        self.zval().ok_or(Error::Callable)?.try_call(params)
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
            DataType::Object => _zend_expected_type_Z_EXPECTED_OBJECT,
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
pub struct ArgParser<'a, 'arg, 'zval> {
    args: Vec<&'arg mut Arg<'zval>>,
    min_num_args: Option<u32>,
    execute_data: &'a ExecutionData,
}

impl<'a, 'arg, 'zval> ArgParser<'a, 'arg, 'zval> {
    /// Builds a new function argument parser.
    pub fn new(execute_data: &'a ExecutionData) -> Self {
        ArgParser {
            args: vec![],
            min_num_args: None,
            execute_data,
        }
    }

    /// Adds a new argument to the parser.
    ///
    /// # Parameters
    ///
    /// * `arg` - The argument to add to the parser.
    pub fn arg(mut self, arg: &'arg mut Arg<'zval>) -> Self {
        self.args.push(arg);
        self
    }

    /// Sets the next arguments to be added as not required.
    pub fn not_required(mut self) -> Self {
        self.min_num_args = Some(self.args.len() as u32);
        self
    }

    /// Uses the argument parser to parse the arguments contained in the given
    /// `ExecutionData` object. Returns successfully if the arguments were parsed.
    ///
    /// This function can only be safely called from within an exported PHP function.
    ///
    /// # Parameters
    ///
    /// * `execute_data` - The execution data from the function.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] type if there were too many or too little arguments passed to the
    /// function. The user has already been notified so you should break execution after seeing an
    /// error type.
    pub fn parse(mut self) -> Result<()> {
        let num_args = unsafe { self.execute_data.This.u2.num_args };
        let max_num_args = self.args.len() as u32;
        let min_num_args = match self.min_num_args {
            Some(n) => n,
            None => max_num_args,
        };

        if num_args < min_num_args || num_args > max_num_args {
            // SAFETY: Exported C function is safe, return value is unused and parameters are copied.
            unsafe { zend_wrong_parameters_count_error(min_num_args, max_num_args) };

            return Err(Error::IncorrectArguments(num_args, min_num_args));
        }

        for (i, arg) in self.args.iter_mut().enumerate() {
            arg.zval = unsafe { self.execute_data.zend_call_arg(i) };
        }

        Ok(())
    }
}
