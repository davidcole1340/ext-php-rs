//! Traits used to convert between Zend/PHP and Rust types.

use crate::{
    boxed::ZBox,
    error::Result,
    exception::PhpException,
    flags::DataType,
    types::{ZendObject, Zval},
};

/// Allows zvals to be converted into Rust types in a fallible way. Reciprocal
/// of the [`IntoZval`] trait.
pub trait FromZval<'a>: Sized {
    /// The corresponding type of the implemented value in PHP.
    const TYPE: DataType;

    /// Attempts to retrieve an instance of `Self` from a reference to a
    /// [`Zval`].
    ///
    /// # Parameters
    ///
    /// * `zval` - Zval to get value from.
    fn from_zval(zval: &'a Zval) -> Option<Self>;
}

impl<'a, T> FromZval<'a> for Option<T>
where
    T: FromZval<'a>,
{
    const TYPE: DataType = T::TYPE;

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        Some(T::from_zval(zval))
    }
}

/// Allows mutable zvals to be converted into Rust types in a fallible way.
///
/// If `Self` does not require the zval to be mutable to be extracted, you
/// should implement [`FromZval`] instead, as this trait is generically
/// implemented for any type that implements [`FromZval`].
pub trait FromZvalMut<'a>: Sized {
    /// The corresponding type of the implemented value in PHP.
    const TYPE: DataType;

    /// Attempts to retrieve an instance of `Self` from a mutable reference to a
    /// [`Zval`].
    ///
    /// # Parameters
    ///
    /// * `zval` - Zval to get value from.
    fn from_zval_mut(zval: &'a mut Zval) -> Option<Self>;
}

impl<'a, T> FromZvalMut<'a> for T
where
    T: FromZval<'a>,
{
    const TYPE: DataType = <T as FromZval>::TYPE;

    #[inline]
    fn from_zval_mut(zval: &'a mut Zval) -> Option<Self> {
        Self::from_zval(zval)
    }
}

/// `FromZendObject` is implemented by types which can be extracted from a Zend
/// object.
///
/// Normal usage is through the helper method `ZendObject::extract`:
///
/// ```rust,ignore
/// let obj: ZendObject = ...;
/// let repr: String = obj.extract();
/// let props: HashMap = obj.extract();
/// ```
///
/// Should be functionally equivalent to casting an object to another compatible
/// type.
pub trait FromZendObject<'a>: Sized {
    /// Extracts `Self` from the source `ZendObject`.
    fn from_zend_object(obj: &'a ZendObject) -> Result<Self>;
}

/// Implemented on types which can be extracted from a mutable zend object.
///
/// If `Self` does not require the object to be mutable, it should implement
/// [`FromZendObject`] instead, as this trait is generically implemented for
/// any types that also implement [`FromZendObject`].
pub trait FromZendObjectMut<'a>: Sized {
    /// Extracts `Self` from the source `ZendObject`.
    fn from_zend_object_mut(obj: &'a mut ZendObject) -> Result<Self>;
}

impl<'a, T> FromZendObjectMut<'a> for T
where
    T: FromZendObject<'a>,
{
    #[inline]
    fn from_zend_object_mut(obj: &'a mut ZendObject) -> Result<Self> {
        Self::from_zend_object(obj)
    }
}

/// Implemented on types which can be converted into a Zend object. It is up to
/// the implementation to determine the type of object which is produced.
pub trait IntoZendObject {
    /// Attempts to convert `self` into a Zend object.
    fn into_zend_object(self) -> Result<ZBox<ZendObject>>;
}

/// Provides implementations for converting Rust primitive types into PHP zvals.
/// Alternative to the built-in Rust [`From`] and [`TryFrom`] implementations,
/// allowing the caller to specify whether the Zval contents will persist
/// between requests.
///
/// [`TryFrom`]: std::convert::TryFrom
pub trait IntoZval: Sized {
    /// The corresponding type of the implemented value in PHP.
    const TYPE: DataType;

    /// Converts a Rust primitive type into a Zval. Returns a result containing
    /// the Zval if successful.
    ///
    /// # Parameters
    ///
    /// * `persistent` - Whether the contents of the Zval will persist between
    ///   requests.
    fn into_zval(self, persistent: bool) -> Result<Zval> {
        let mut zval = Zval::new();
        self.set_zval(&mut zval, persistent)?;
        Ok(zval)
    }

    /// Sets the content of a pre-existing zval. Returns a result containing
    /// nothing if setting the content was successful.
    ///
    /// # Parameters
    ///
    /// * `zv` - The Zval to set the content of.
    /// * `persistent` - Whether the contents of the Zval will persist between
    ///   requests.
    fn set_zval(self, zv: &mut Zval, persistent: bool) -> Result<()>;
}

impl IntoZval for () {
    const TYPE: DataType = DataType::Void;

    #[inline]
    fn set_zval(self, zv: &mut Zval, _: bool) -> Result<()> {
        zv.set_null();
        Ok(())
    }
}

impl<T> IntoZval for Option<T>
where
    T: IntoZval,
{
    const TYPE: DataType = T::TYPE;

    #[inline]
    fn set_zval(self, zv: &mut Zval, persistent: bool) -> Result<()> {
        match self {
            Some(val) => val.set_zval(zv, persistent),
            None => {
                zv.set_null();
                Ok(())
            }
        }
    }
}

impl<T, E> IntoZval for std::result::Result<T, E>
where
    T: IntoZval,
    E: Into<PhpException>,
{
    const TYPE: DataType = T::TYPE;

    fn set_zval(self, zv: &mut Zval, persistent: bool) -> Result<()> {
        match self {
            Ok(val) => val.set_zval(zv, persistent),
            Err(e) => {
                let ex: PhpException = e.into();
                ex.throw()
            }
        }
    }
}

/// An object-safe version of the [`IntoZval`] trait.
///
/// This trait is automatically implemented on any type that implements both
/// [`IntoZval`] and [`Clone`]. You avoid implementing this trait directly,
/// rather implement these two other traits.
pub trait IntoZvalDyn {
    /// Converts a Rust primitive type into a Zval. Returns a result containing
    /// the Zval if successful. `self` is cloned before being converted into
    /// a zval.
    ///
    /// # Parameters
    ///
    /// * `persistent` - Whether the contents of the Zval will persist between
    ///   requests.
    fn as_zval(&self, persistent: bool) -> Result<Zval>;

    /// Returns the PHP type of the type.
    fn get_type(&self) -> DataType;
}

impl<T: IntoZval + Clone> IntoZvalDyn for T {
    fn as_zval(&self, persistent: bool) -> Result<Zval> {
        self.clone().into_zval(persistent)
    }

    fn get_type(&self) -> DataType {
        Self::TYPE
    }
}

impl IntoZvalDyn for Zval {
    fn as_zval(&self, _persistent: bool) -> Result<Zval> {
        Ok(self.shallow_clone())
    }

    fn get_type(&self) -> DataType {
        self.get_type()
    }
}
