use std::any::Any;

use super::{enums::DataType, function::ExecutionData};

use crate::bindings::zend_internal_arg_info;

/// Represents an argument to a function.
pub struct Arg {
    pub(crate) name: String,
    pub(crate) _type: DataType,
    pub(crate) as_ref: bool,
    pub(crate) allow_null: bool,
    pub(crate) default_value: Option<String>,
}

impl Arg {
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
}

pub type ArgInfo = zend_internal_arg_info;

pub struct ArgParser {
    args: Vec<Arg>,
    n_req: Option<u32>,
}

impl ArgParser {
    /// Builds a new function argument parser.
    pub fn new() -> Self {
        ArgParser {
            args: vec![],
            n_req: None,
        }
    }

    /// Adds a new argument to the parser.
    ///
    /// # Parameters
    ///
    /// * `arg` - The argument to add to the parser.
    pub fn arg(mut self, arg: Arg) -> Self {
        self.args.push(arg);
        self
    }

    /// Sets the next arguments to be added as not required.
    pub fn not_required(mut self) -> Self {
        self.n_req = Some(self.args.len() as u32);
        self
    }

    pub fn parse(mut self, execute_data: &mut ExecutionData) -> Result<(), String> {
        if let Some(n_req) = self.n_req {
            let num_args = unsafe { execute_data.This.u2.num_args };
            if num_args < n_req || num_args > self.args.len() as u32 {
                return Err(format!(
                    "Expected {} arguments, got {} arguments.",
                    n_req, num_args,
                ));
            }
        }

        for arg in self.args {
            match arg._type {
                DataType::Long => {}
                _ => return Err(String::from("argument type is not implemented")),
            }
        }

        Ok(())
    }
}
