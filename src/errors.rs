use std::{error::Error as ErrorTrait, fmt::Display};

use crate::php::{enums::DataType, flags::ZvalTypeFlags};

/// The main result type which is passed by the library.
pub type Result<T> = std::result::Result<T, Error>;

/// The main error type which is passed by the library inside the custom
/// [`Result`] type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Error {
    /// An incorrect number of arguments was given to a PHP function.
    ///
    /// The enum carries two integers - the first representing the minimum
    /// number of arguments expected, and the second representing the number of
    /// arguments that were received.
    IncorrectArguments(u32, u32),
    /// There was an error converting a Zval into a primitive type.
    ///
    /// The enum carries the data type of the Zval.
    ZvalConversion(DataType),
    /// The type of the Zval is unknown.
    ///
    /// The enum carries the integer representation of the type of Zval.
    UnknownDatatype(u32),
    /// Attempted to convert a [`ZvalTypeFlags`] struct to a [`DataType`].
    /// The flags did not contain a datatype.
    ///
    /// The enum carries the flags that were attempted to be converted to a [`DataType`].
    InvalidTypeToDatatype(ZvalTypeFlags),
    /// The function called was called in an invalid scope (calling class-related functions
    /// inside of a non-class bound function).
    InvalidScope,
    /// The pointer inside a given type was invalid, either null or pointing to garbage.
    InvalidPointer,
    /// The given property name does not exist.
    InvalidProperty,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IncorrectArguments(n, expected) => write!(
                f,
                "Expected at least {} arguments, got {} arguments.",
                expected, n
            ),
            Error::ZvalConversion(ty) => write!(
                f,
                "Could not convert Zval from type {} into primitive type.",
                ty
            ),
            Error::UnknownDatatype(dt) => write!(f, "Unknown datatype {}.", dt),
            Error::InvalidTypeToDatatype(dt) => {
                write!(f, "Type flags did not contain a datatype: {:?}", dt)
            }
            Error::InvalidScope => write!(f, "Invalid scope."),
            Error::InvalidPointer => write!(f, "Invalid pointer."),
            Error::InvalidProperty => write!(f, "Property does not exist on object."),
        }
    }
}

impl ErrorTrait for Error {}
