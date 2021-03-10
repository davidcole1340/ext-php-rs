use std::convert::{TryFrom, TryInto};

use super::{enums::DataType, execution_data::ExecutionData, zval::Zval};

use crate::bindings::{zend_internal_arg_info, zend_wrong_parameters_count_error};

/// Represents an argument to a function.
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
    pub fn new<S>(name: S, _type: DataType) -> Self
    where
        S: ToString,
    {
        Arg {
            name: name.to_string(),
            _type,
            as_ref: false,
            allow_null: false,
            default_value: None,
            zval: None,
        }
    }

    /// Sets the argument as a reference.
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
    pub fn default<S>(mut self, default: S) -> Self
    where
        S: ToString,
    {
        self.default_value = Some(default.to_string());
        self
    }

    /// Attempts to retrieve the value of the argument.
    /// This will be None until the ArgParser is used to parse
    /// the arguments.
    pub fn val<T>(&self) -> Option<T>
    where
        T: TryFrom<&'a Zval>,
    {
        match self.zval {
            Some(zval) => match zval.try_into() {
                Ok(val) => Some(val),
                Err(_) => None,
            },
            None => None,
        }
    }
}

/// Internal argument information used by Zend.
pub type ArgInfo = zend_internal_arg_info;

/// Parses the arguments of a function.
pub struct ArgParser<'a, 'b> {
    args: Vec<&'a mut Arg<'b>>,
    min_num_args: Option<u32>,
    execute_data: *mut ExecutionData,
}

impl<'a, 'b> ArgParser<'a, 'b> {
    /// Builds a new function argument parser.
    pub fn new(execute_data: *mut ExecutionData) -> Self {
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
    pub fn arg(mut self, arg: &'a mut Arg<'b>) -> Self {
        self.args.push(arg);
        self
    }

    /// Sets the next arguments to be added as not required.
    pub fn not_required(mut self) -> Self {
        self.min_num_args = Some(self.args.len() as u32);
        self
    }

    /// Uses the argument parser to parse the arguments contained in the given
    /// `ExecutionData` object.
    ///
    /// # Parameters
    ///
    /// * `execute_data` - The execution data from the function.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The arguments were successfully parsed.
    /// * `Err(String)` - There were too many or too little arguments
    /// passed to the function. The user has already been notified so you
    /// can discard and return from the function if an `Err` is received.
    pub fn parse(mut self) -> Result<(), String> {
        let execute_data = unsafe { self.execute_data.as_ref() }.unwrap();
        let num_args = unsafe { execute_data.This.u2.num_args };
        let max_num_args = self.args.len() as u32;
        let min_num_args = match self.min_num_args {
            Some(n) => n,
            None => max_num_args,
        };

        if num_args < min_num_args || num_args > max_num_args {
            unsafe { zend_wrong_parameters_count_error(min_num_args, max_num_args) };

            return Err(format!(
                "Expected at least {} arguments, got {} arguments.",
                min_num_args, num_args,
            ));
        }

        for (i, arg) in self.args.iter_mut().enumerate() {
            let zval = unsafe { execute_data.zend_call_arg(i) };

            if let Some(zval) = zval {
                arg.zval = Some(zval);
            }
        }

        Ok(())
    }
}
