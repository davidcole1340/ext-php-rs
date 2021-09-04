//! Types relating to binary data transmission between Rust and PHP.

use std::{
    convert::{TryFrom, TryInto},
    iter::FromIterator,
    ops::{Deref, DerefMut},
};

use crate::{
    errors::{Error, Result},
    php::{enums::DataType, pack::Pack},
};

use super::zval::{FromZval, IntoZval, Zval};

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

impl<'a, T: Pack> FromZval<'a> for Binary<T> {
    const TYPE: DataType = DataType::String;
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

impl<T: Pack> TryFrom<Zval> for Binary<T> {
    type Error = Error;

    fn try_from(value: Zval) -> Result<Self> {
        (&value).try_into()
    }
}

impl<T: Pack> IntoZval for Binary<T> {
    const TYPE: DataType = DataType::String;

    fn set_zval(self, zv: &mut Zval, _: bool) -> Result<()> {
        zv.set_binary(self.0);
        Ok(())
    }
}

impl<T: Pack> From<Binary<T>> for Vec<T> {
    fn from(value: Binary<T>) -> Self {
        value.0
    }
}

impl<T: Pack> From<Vec<T>> for Binary<T> {
    fn from(value: Vec<T>) -> Self {
        Self::new(value)
    }
}

impl<T: Pack> FromIterator<T> for Binary<T> {
    fn from_iter<U: IntoIterator<Item = T>>(iter: U) -> Self {
        Self(iter.into_iter().collect::<Vec<_>>())
    }
}
