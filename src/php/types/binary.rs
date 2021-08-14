//! Types relating to binary data transmission between Rust and PHP.

use std::{
    convert::TryFrom,
    ops::{Deref, DerefMut},
};

use crate::{
    errors::{Error, Result},
    php::pack::Pack,
};

use super::zval::{IntoZval, Zval};

/// Acts as a wrapper around [`Vec<T>`] where `T` implements [`Pack`]. Primarily used for passing
/// binary data into Rust functions. Can be treated as a [`Vec`] in most situations, or can be
/// 'unwrapped' into a [`Vec`] through the [`From`] implementation on [`Vec`].
#[derive(Debug)]
pub struct Binary<T: Pack>(Vec<T>);

impl<T: Pack> Binary<T> {
    /// Creates a new binary wrapper from a set of data which can be converted into a vector.
    ///
    /// # Parameters
    ///
    /// * `data` - Data to store inside the binary wrapper.
    pub fn new(data: impl Into<Vec<T>>) -> Self {
        Self(data.into())
    }
}

impl<T: Pack> Deref for Binary<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Pack> DerefMut for Binary<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Pack> TryFrom<&Zval> for Binary<T> {
    type Error = Error;

    fn try_from(value: &Zval) -> Result<Self> {
        match value.binary() {
            Some(b) => Ok(Binary(b)),
            None => Err(Error::Callable),
        }
    }
}

impl<T: Pack> IntoZval for Binary<T> {
    fn set_zval(&self, zv: &mut Zval, _: bool) -> Result<()> {
        zv.set_binary(&self.0);
        Ok(())
    }
}

impl<T: Pack> From<Binary<T>> for Vec<T> {
    fn from(value: Binary<T>) -> Self {
        value.0
    }
}
