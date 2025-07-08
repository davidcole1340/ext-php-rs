//! This module defines the `PhpEnum` trait and related types for Rust enums that are exported to PHP.
use crate::{
    convert::IntoZval,
    describe::DocComments,
    error::{Error, Result},
    flags::DataType,
    types::Zval,
};

/// Implemented on Rust enums which are exported to PHP.
pub trait PhpEnum {
    /// The cases of the enum.
    const CASES: &'static [EnumCase];
}

/// Represents a case in a PHP enum.
pub struct EnumCase {
    /// The identifier of the enum case, e.g. `Bar` in `enum Foo { Bar }`.
    pub name: &'static str,
    /// The value of the enum case, which can be an integer or a string.
    pub discriminant: Option<Discriminant>,
    /// The documentation comments for the enum case.
    pub docs: DocComments,
}

impl EnumCase {
    /// Gets the PHP data type of the enum case's discriminant.
    #[must_use]
    pub fn data_type(&self) -> DataType {
        match self.discriminant {
            Some(Discriminant::Int(_)) => DataType::Long,
            Some(Discriminant::String(_)) => DataType::String,
            None => DataType::Undef,
        }
    }
}

/// Represents the discriminant of an enum case in PHP, which can be either an integer or a string.
pub enum Discriminant {
    /// An integer discriminant.
    Int(i64),
    /// A string discriminant.
    String(&'static str),
}

impl TryFrom<&Discriminant> for Zval {
    type Error = Error;

    fn try_from(value: &Discriminant) -> Result<Self> {
        match value {
            Discriminant::Int(i) => i.into_zval(false),
            Discriminant::String(s) => s.into_zval(true),
        }
    }
}
