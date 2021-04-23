use std::{error::Error as ErrorTrait, fmt::Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Error {
    IncorrectArguments(u32, u32),
    ZvalConversionError,
    UnknownDatatype(u32),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IncorrectArguments(n, expected) => write!(
                f,
                "Expected at least {} arguments, got {} arguments.",
                expected, n
            ),
            Error::ZvalConversionError => {
                write!(f, "Unable to convert from Zval to primitive type.")
            }
            Error::UnknownDatatype(dt) => write!(f, "Unknown datatype {}.", dt),
            _ => unreachable!(),
        }
    }
}
