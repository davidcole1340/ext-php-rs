use std::{fmt::Display, sync::PoisonError};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(String);

impl Error {
    pub fn new(inner: String) -> Self {
        Self(inner)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for Error {
    fn from(val: String) -> Self {
        Self(val)
    }
}

impl From<&str> for Error {
    fn from(val: &str) -> Self {
        Self(val.to_string())
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Self {
        Self("Unable to lock `ext-php-rs-derive` state.".into())
    }
}
