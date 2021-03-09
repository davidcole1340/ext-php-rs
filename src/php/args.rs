use super::enums::DataType;

use crate::bindings::zend_internal_arg_info;

/// Represents an argument to a function.
pub struct Arg {
    pub(crate) name: String,
    pub(crate) _type: DataType,
    pub(crate) required: bool,
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
            required: true,
            as_ref: false,
            allow_null: false,
            default_value: None,
        }
    }

    /// Sets the argument as not required.
    pub fn not_required(mut self) -> Self {
        self.required = false;
        self
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
