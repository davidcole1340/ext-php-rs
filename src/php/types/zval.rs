//! The base value in PHP. A Zval can contain any PHP type, and the type that it contains is
//! determined by a property inside the struct. The content of the Zval is stored in a union.

use std::{
    convert::{TryFrom, TryInto},
    ffi::c_void,
    fmt::Debug,
    ptr,
};

use crate::{
    bindings::{
        _zval_struct__bindgen_ty_1, _zval_struct__bindgen_ty_2, zend_is_callable, zend_resource,
        zend_value, zval, zval_ptr_dtor,
    },
    errors::{Error, Result},
    php::{exceptions::PhpException, pack::Pack},
};

use crate::php::{
    enums::DataType,
    flags::ZvalTypeFlags,
    types::{long::ZendLong, string::ZendString},
};

use super::{
    array::{HashTable, OwnedHashTable},
    callable::Callable,
    object::ZendObject,
    rc::PhpRc,
    string::ZendStr,
};

/// Zend value. Represents most data types that are in the Zend engine.
pub type Zval = zval;

unsafe impl Send for Zval {}
unsafe impl Sync for Zval {}

impl Zval {
    /// Creates a new, empty zval.
    pub const fn new() -> Self {
        Self {
            value: zend_value {
                ptr: ptr::null_mut(),
            },
            u1: _zval_struct__bindgen_ty_1 {
                type_info: DataType::Null.as_u32(),
            },
            u2: _zval_struct__bindgen_ty_2 { next: 0 },
        }
    }

    /// Returns the value of the zval if it is a long.
    pub fn long(&self) -> Option<ZendLong> {
        if self.is_long() {
            Some(unsafe { self.value.lval })
        } else {
            None
        }
    }

    /// Returns the value of the zval if it is a bool.
    pub fn bool(&self) -> Option<bool> {
        if self.is_true() {
            Some(true)
        } else if self.is_false() {
            Some(false)
        } else {
            None
        }
    }

    /// Returns the value of the zval if it is a double.
    pub fn double(&self) -> Option<f64> {
        if self.is_double() {
            Some(unsafe { self.value.dval })
        } else {
            None
        }
    }

    /// Returns the value of the zval as a zend string, if it is a string.
    pub fn zend_str(&self) -> Option<&ZendStr> {
        if self.is_string() {
            unsafe { self.value.str_.as_ref() }
        } else {
            None
        }
    }

    /// Returns the value of the zval if it is a string.
    pub fn string(&self) -> Option<String> {
        self.str().map(|s| s.to_string())
    }

    /// Returns the value of the zval if it is a string.
    pub fn str(&self) -> Option<&str> {
        self.zend_str().and_then(|zs| zs.as_str())
    }

    /// Returns the value of the zval if it is a string and can be unpacked into a vector of a
    /// given type. Similar to the [`unpack`](https://www.php.net/manual/en/function.unpack.php)
    /// in PHP, except you can only unpack one type.
    ///
    /// # Safety
    ///
    /// There is no way to tell if the data stored in the string is actually of the given type.
    /// The results of this function can also differ from platform-to-platform due to the different
    /// representation of some types on different platforms. Consult the [`pack`] function
    /// documentation for more details.
    ///
    /// [`pack`]: https://www.php.net/manual/en/function.pack.php
    pub fn binary<T: Pack>(&self) -> Option<Vec<T>> {
        if self.is_string() {
            // SAFETY: Type is string therefore we are able to take a reference.
            Some(T::unpack_into(unsafe { self.value.str_.as_ref() }?))
        } else {
            None
        }
    }

    /// Returns the value of the zval if it is a resource.
    pub fn resource(&self) -> Option<*mut zend_resource> {
        // TODO: Can we improve this function? I haven't done much research into
        // resources so I don't know if this is the optimal way to return this.
        if self.is_resource() {
            Some(unsafe { self.value.res })
        } else {
            None
        }
    }

    /// Returns an immutable reference to the underlying zval hashtable if the zval contains an array.
    pub fn array(&self) -> Option<&HashTable> {
        if self.is_array() {
            unsafe { self.value.arr.as_ref() }
        } else {
            None
        }
    }

    /// Returns a mutable reference to the underlying zval hashtable if the zval contains an array.
    pub fn array_mut(&mut self) -> Option<&mut HashTable> {
        if self.is_array() {
            unsafe { self.value.arr.as_mut() }
        } else {
            None
        }
    }

    /// Returns the value of the zval if it is an object.
    pub fn object(&self) -> Option<&mut ZendObject> {
        if self.is_object() {
            unsafe { self.value.obj.as_mut() }
        } else {
            None
        }
    }

    /// Returns the value of the zval if it is a reference.
    pub fn reference(&self) -> Option<&Zval> {
        if self.is_reference() {
            Some(&unsafe { self.value.ref_.as_ref() }?.val)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the underlying zval if it is a reference.
    pub fn reference_mut(&mut self) -> Option<&mut Zval> {
        if self.is_reference() {
            Some(&mut unsafe { self.value.ref_.as_mut() }?.val)
        } else {
            None
        }
    }

    /// Returns the value of the zval if it is callable.
    pub fn callable(&self) -> Option<Callable> {
        // The Zval is checked if it is callable in the `new` function.
        Callable::new(self).ok()
    }

    /// Returns the value of the zval if it is a pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the pointer contained in the zval is in fact a pointer to an
    /// instance of `T`, as the zval has no way of defining the type of pointer.
    pub unsafe fn ptr<T>(&self) -> Option<*mut T> {
        if self.is_ptr() {
            Some(self.value.ptr as *mut T)
        } else {
            None
        }
    }

    /// Attempts to call the zval as a callable with a list of arguments to pass to the function.
    /// Note that a thrown exception inside the callable is not detectable, therefore you should
    /// check if the return value is valid rather than unwrapping. Returns a result containing the
    /// return value of the function, or an error.
    ///
    /// You should not call this function directly, rather through the [`call_user_func`] macro.
    ///
    /// # Parameters
    ///
    /// * `params` - A list of parameters to call the function with.
    pub fn try_call(&self, params: Vec<&dyn IntoZvalDyn>) -> Result<Zval> {
        self.callable().ok_or(Error::Callable)?.try_call(params)
    }

    /// Returns the type of the Zval.
    pub fn get_type(&self) -> DataType {
        DataType::from(unsafe { self.u1.v.type_ } as u32)
    }

    /// Returns true if the zval is a long, false otherwise.
    pub fn is_long(&self) -> bool {
        self.get_type() == DataType::Long
    }

    /// Returns true if the zval is null, false otherwise.
    pub fn is_null(&self) -> bool {
        self.get_type() == DataType::Null
    }

    /// Returns true if the zval is true, false otherwise.
    pub fn is_true(&self) -> bool {
        self.get_type() == DataType::False
    }

    /// Returns true if the zval is false, false otherwise.
    pub fn is_false(&self) -> bool {
        self.get_type() == DataType::False
    }

    /// Returns true if the zval is a bool, false otherwise.
    pub fn is_bool(&self) -> bool {
        self.is_true() || self.is_false()
    }

    /// Returns true if the zval is a double, false otherwise.
    pub fn is_double(&self) -> bool {
        self.get_type() == DataType::Double
    }

    /// Returns true if the zval is a string, false otherwise.
    pub fn is_string(&self) -> bool {
        self.get_type() == DataType::String
    }

    /// Returns true if the zval is a resource, false otherwise.
    pub fn is_resource(&self) -> bool {
        self.get_type() == DataType::Resource
    }

    /// Returns true if the zval is an array, false otherwise.
    pub fn is_array(&self) -> bool {
        self.get_type() == DataType::Array
    }

    /// Returns true if the zval is an object, false otherwise.
    pub fn is_object(&self) -> bool {
        matches!(self.get_type(), DataType::Object(_))
    }

    /// Returns true if the zval is a reference, false otherwise.
    pub fn is_reference(&self) -> bool {
        self.get_type() == DataType::Reference
    }

    /// Returns true if the zval is callable, false otherwise.
    pub fn is_callable(&self) -> bool {
        let ptr: *const Self = self;
        unsafe { zend_is_callable(ptr as *mut Self, 0, std::ptr::null_mut()) }
    }

    /// Returns true if the zval contains a pointer, false otherwise.
    pub fn is_ptr(&self) -> bool {
        self.get_type() == DataType::Ptr
    }

    /// Sets the value of the zval as a string. Returns nothing in a result when successful.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    /// * `persistent` - Whether the string should persist between requests.
    pub fn set_string(&mut self, val: &str, persistent: bool) -> Result<()> {
        self.set_zend_string(ZendString::new(val, persistent)?);
        Ok(())
    }

    /// Sets the value of the zval as a Zend string.
    ///
    /// # Parameters
    ///
    /// * `val` - String content.
    pub fn set_zend_string(&mut self, val: ZendString) {
        self.change_type(ZvalTypeFlags::StringEx);
        self.value.str_ = val.into_inner();
    }

    /// Sets the value of the zval as a binary string, which is represented in Rust as a vector.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    pub fn set_binary<T: Pack>(&mut self, val: Vec<T>) {
        self.change_type(ZvalTypeFlags::StringEx);
        let ptr = T::pack_into(val);
        self.value.str_ = ptr;
    }

    /// Sets the value of the zval as a interned string. Returns nothing in a result when successful.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    /// * `persistent` - Whether the string should persist between requests.
    pub fn set_interned_string(&mut self, val: &str, persistent: bool) -> Result<()> {
        self.set_zend_string(ZendString::new_interned(val, persistent)?);
        Ok(())
    }

    /// Sets the value of the zval as a long.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    pub fn set_long<T: Into<ZendLong>>(&mut self, val: T) {
        self._set_long(val.into())
    }

    fn _set_long(&mut self, val: ZendLong) {
        self.change_type(ZvalTypeFlags::Long);
        self.value.lval = val;
    }

    /// Sets the value of the zval as a double.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    pub fn set_double<T: Into<f64>>(&mut self, val: T) {
        self._set_double(val.into())
    }

    fn _set_double(&mut self, val: f64) {
        self.change_type(ZvalTypeFlags::Double);
        self.value.dval = val;
    }

    /// Sets the value of the zval as a boolean.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    pub fn set_bool<T: Into<bool>>(&mut self, val: T) {
        self._set_bool(val.into())
    }

    fn _set_bool(&mut self, val: bool) {
        self.change_type(if val {
            ZvalTypeFlags::True
        } else {
            ZvalTypeFlags::False
        });
    }

    /// Sets the value of the zval as null.
    ///
    /// This is the default of a zval.
    pub fn set_null(&mut self) {
        self.change_type(ZvalTypeFlags::Null);
    }

    /// Sets the value of the zval as a resource.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    pub fn set_resource(&mut self, val: *mut zend_resource) {
        self.change_type(ZvalTypeFlags::ResourceEx);
        self.value.res = val;
    }

    /// Sets the value of the zval as a reference to an object.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    pub fn set_object(&mut self, val: &mut ZendObject) {
        self.change_type(ZvalTypeFlags::ObjectEx);
        val.inc_count(); // TODO(david): not sure if this is needed :/
        self.value.obj = (val as *const ZendObject) as *mut ZendObject;
    }

    /// Sets the value of the zval as an array. Returns nothing in a result on success.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    pub fn set_array<T: TryInto<OwnedHashTable, Error = Error>>(&mut self, val: T) -> Result<()> {
        self.set_hashtable(val.try_into()?);
        Ok(())
    }

    /// Sets the value of the zval as an array. Returns nothing in a result on success.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    pub fn set_hashtable(&mut self, val: OwnedHashTable) {
        self.change_type(ZvalTypeFlags::ArrayEx);
        self.value.arr = val.into_inner();
    }

    /// Sets the value of the zval as a pointer.
    ///
    /// # Parameters
    ///
    /// * `ptr` - The pointer to set the zval as.
    pub fn set_ptr<T>(&mut self, ptr: *mut T) {
        self.u1.type_info = ZvalTypeFlags::Ptr.bits();
        self.value.ptr = ptr as *mut c_void;
    }

    /// Used to drop the Zval but keep the value of the zval intact.
    ///
    /// This is important when copying the value of the zval, as the actual value
    /// will not be copied, but the pointer to the value (string for example) will be
    /// copied.
    pub(crate) fn release(mut self) {
        // NOTE(david): don't use `change_type` here as we are wanting to keep the contents intact.
        self.u1.type_info = ZvalTypeFlags::Null.bits();
    }

    /// Changes the type of the zval, freeing the current contents when applicable.
    ///
    /// # Parameters
    ///
    /// * `ty` - The new type of the zval.
    fn change_type(&mut self, ty: ZvalTypeFlags) {
        // SAFETY: we have exclusive mutable access to this zval so can free the contents.
        unsafe { zval_ptr_dtor(self) };
        self.u1.type_info = ty.bits();
    }
}

impl Debug for Zval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_struct("Zval");
        let ty = self.get_type();
        dbg.field("type", &ty);

        macro_rules! field {
            ($value: expr) => {
                dbg.field("val", &$value)
            };
        }

        match ty {
            DataType::Undef => field!(Option::<()>::None),
            DataType::Null => field!(Option::<()>::None),
            DataType::False => field!(false),
            DataType::True => field!(true),
            DataType::Long => field!(self.long()),
            DataType::Double => field!(self.double()),
            DataType::String | DataType::Mixed => field!(self.string()),
            DataType::Array => field!(self.array()),
            DataType::Object(_) => field!(self.object()),
            DataType::Resource => field!(self.resource()),
            DataType::Reference => field!(self.reference()),
            DataType::Callable => field!(self.string()),
            DataType::ConstantExpression => field!(Option::<()>::None),
            DataType::Void => field!(Option::<()>::None),
            DataType::Bool => field!(self.bool()),
            // SAFETY: We are not accessing the pointer.
            DataType::Ptr => field!(unsafe { self.ptr::<c_void>() }),
        };

        dbg.finish()
    }
}

impl Drop for Zval {
    fn drop(&mut self) {
        self.change_type(ZvalTypeFlags::Null);
    }
}

impl Default for Zval {
    fn default() -> Self {
        Self::new()
    }
}

/// Provides implementations for converting Rust primitive types into PHP zvals. Alternative to the
/// built-in Rust [`From`] and [`TryFrom`] implementations, allowing the caller to specify whether
/// the Zval contents will persist between requests.
pub trait IntoZval: Sized {
    /// The corresponding type of the implemented value in PHP.
    const TYPE: DataType;

    /// Converts a Rust primitive type into a Zval. Returns a result containing the Zval if
    /// successful.
    ///
    /// # Parameters
    ///
    /// * `persistent` - Whether the contents of the Zval will persist between requests.
    fn into_zval(self, persistent: bool) -> Result<Zval> {
        let mut zval = Zval::new();
        self.set_zval(&mut zval, persistent)?;
        Ok(zval)
    }

    /// Sets the content of a pre-existing zval. Returns a result containing nothing if setting
    /// the content was successful.
    ///
    /// # Parameters
    ///
    /// * `zv` - The Zval to set the content of.
    /// * `persistent` - Whether the contents of the Zval will persist between requests.
    fn set_zval(self, zv: &mut Zval, persistent: bool) -> Result<()>;
}

impl IntoZval for Zval {
    const TYPE: DataType = DataType::Mixed;

    fn set_zval(self, zv: &mut Zval, _: bool) -> Result<()> {
        *zv = self;
        Ok(())
    }
}

/// An object-safe version of the [`IntoZval`] trait.
///
/// This trait is automatically implemented on any type that implements both [`IntoZval`] and [`Clone`].
/// You avoid implementing this trait directly, rather implement these two other traits.
pub trait IntoZvalDyn {
    /// Converts a Rust primitive type into a Zval. Returns a result containing the Zval if
    /// successful. `self` is cloned before being converted into a zval.
    ///
    /// # Parameters
    ///
    /// * `persistent` - Whether the contents of the Zval will persist between requests.
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

macro_rules! into_zval {
    ($type: ty, $fn: ident, $dt: ident) => {
        impl From<$type> for Zval {
            fn from(val: $type) -> Self {
                let mut zv = Self::new();
                zv.$fn(val);
                zv
            }
        }

        impl IntoZval for $type {
            const TYPE: DataType = DataType::$dt;

            fn set_zval(self, zv: &mut Zval, _: bool) -> Result<()> {
                zv.$fn(self);
                Ok(())
            }
        }
    };
}

into_zval!(i8, set_long, Long);
into_zval!(i16, set_long, Long);
into_zval!(i32, set_long, Long);

into_zval!(u8, set_long, Long);
into_zval!(u16, set_long, Long);

into_zval!(f32, set_double, Double);
into_zval!(f64, set_double, Double);

into_zval!(bool, set_bool, Bool);

macro_rules! try_into_zval_int {
    ($type: ty) => {
        impl TryFrom<$type> for Zval {
            type Error = Error;

            fn try_from(val: $type) -> Result<Self> {
                let mut zv = Self::new();
                let val: ZendLong = val.try_into().map_err(|_| Error::IntegerOverflow)?;
                zv.set_long(val);
                Ok(zv)
            }
        }

        impl IntoZval for $type {
            const TYPE: DataType = DataType::Long;

            fn set_zval(self, zv: &mut Zval, _: bool) -> Result<()> {
                let val: ZendLong = self.try_into().map_err(|_| Error::IntegerOverflow)?;
                zv.set_long(val);
                Ok(())
            }
        }
    };
}

try_into_zval_int!(i64);
try_into_zval_int!(u32);
try_into_zval_int!(u64);

try_into_zval_int!(isize);
try_into_zval_int!(usize);

macro_rules! try_into_zval_str {
    ($type: ty) => {
        impl TryFrom<$type> for Zval {
            type Error = Error;

            fn try_from(value: $type) -> Result<Self> {
                let mut zv = Self::new();
                zv.set_string(&value, false)?;
                Ok(zv)
            }
        }

        impl IntoZval for $type {
            const TYPE: DataType = DataType::String;

            fn set_zval(self, zv: &mut Zval, persistent: bool) -> Result<()> {
                zv.set_string(&self, persistent)
            }
        }
    };
}

try_into_zval_str!(String);
try_into_zval_str!(&str);

impl IntoZval for () {
    const TYPE: DataType = DataType::Void;

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

/// Allows zvals to be converted into Rust types in a fallible way. Reciprocal of the [`IntoZval`]
/// trait.
pub trait FromZval<'a>: Sized {
    /// The corresponding type of the implemented value in PHP.
    const TYPE: DataType;

    /// Attempts to retrieve an instance of `Self` from a reference to a [`Zval`].
    ///
    /// # Parameters
    ///
    /// * `zval` - Zval to get value from.
    fn from_zval(zval: &'a Zval) -> Option<Self>;

    /// Attempts to retrieve an instance of `Self` from a reference to a [`Zval], coercing through
    /// other types if required.
    ///
    /// For example, [`String`] may implement `from_zval_coerce` by checking for a string, returning if
    /// found, and then checking for a long and converting that to a string.
    ///
    /// # Parameters
    ///
    /// * `zval` - Zval to get value from.
    fn from_zval_coerce(zval: &'a Zval) -> Option<Self> {
        Self::from_zval(zval)
    }
}

impl<'a, T> FromZval<'a> for Option<T>
where
    T: FromZval<'a>,
{
    const TYPE: DataType = T::TYPE;

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        Some(T::from_zval(zval))
    }

    fn from_zval_coerce(zval: &'a Zval) -> Option<Self> {
        Some(T::from_zval_coerce(zval))
    }
}

// Coercion type juggling: https://www.php.net/manual/en/language.types.type-juggling.php

macro_rules! try_from_zval {
    ($($type: ty),*) => {
        $(
            impl TryFrom<Zval> for $type {
                type Error = Error;

                fn try_from(value: Zval) -> Result<Self> {
                    Self::from_zval(&value).ok_or(Error::ZvalConversion(value.get_type()))
                }
            }
        )*
    };
}

macro_rules! from_zval_long {
    ($($t: ty),*) => {
        $(
            impl FromZval<'_> for $t {
                const TYPE: DataType = DataType::Long;

                fn from_zval(zval: &Zval) -> Option<Self> {
                    zval.long().and_then(|val| val.try_into().ok())
                }

                fn from_zval_coerce(zval: &Zval) -> Option<Self> {
                    // https://www.php.net/manual/en/language.types.integer.php#language.types.integer.casting
                    zval.long()
                        .and_then(|val| val.try_into().ok())
                        .or_else(|| zval.bool().map(|b| b.into()))
                        .or_else(|| {
                            zval.double()
                                .map(|d| if d.is_normal() { d.floor() as _ } else { 0 })
                        })
                        .or_else(|| if zval.is_null() { Some(0) } else { None })
                }
            }
        )*
        try_from_zval!($($t),*);
    };
}

try_from_zval!(String, f32, f64, bool);
from_zval_long!(i8, i16, i32, i64, u8, u16, u32, u64, usize, isize);

impl FromZval<'_> for String {
    const TYPE: DataType = DataType::String;

    fn from_zval(zval: &Zval) -> Option<Self> {
        zval.string()
    }

    fn from_zval_coerce(zval: &Zval) -> Option<Self> {
        // https://www.php.net/manual/en/language.types.string.php#language.types.string.casting
        zval.string()
            .or_else(|| zval.long().map(|l| l.to_string()))
            .or_else(|| zval.double().map(|d| d.to_string()))
            .or_else(|| zval.bool().map(|b| if b { "1" } else { "" }.to_string()))
            .or_else(|| {
                if zval.is_null() {
                    Some("".to_string())
                } else {
                    None
                }
            })
    }
}

impl FromZval<'_> for f64 {
    const TYPE: DataType = DataType::Double;

    fn from_zval(zval: &Zval) -> Option<Self> {
        zval.double()
    }

    fn from_zval_coerce(zval: &Zval) -> Option<Self> {
        zval.double()
            .or_else(|| i64::from_zval_coerce(zval).map(|l| l as f64))
    }
}

impl FromZval<'_> for f32 {
    const TYPE: DataType = DataType::Double;

    fn from_zval(zval: &Zval) -> Option<Self> {
        zval.double().map(|v| v as f32)
    }

    fn from_zval_coerce(zval: &Zval) -> Option<Self> {
        f64::from_zval_coerce(zval).map(|v| v as f32)
    }
}

impl FromZval<'_> for bool {
    const TYPE: DataType = DataType::Bool;

    fn from_zval(zval: &Zval) -> Option<Self> {
        zval.bool()
    }

    fn from_zval_coerce(zval: &Zval) -> Option<Self> {
        // https://www.php.net/manual/en/language.types.boolean.php#language.types.boolean.casting
        zval.bool()
            .or_else(|| zval.long().map(|l| l != 0))
            .or_else(|| zval.double().map(|d| d != 0.0))
            .or_else(|| zval.str().map(|s| !(s.len() == 0 || s == "0") || s == "1"))
            .or_else(|| zval.array().map(|arr| arr.len() != 0))
            .or_else(|| if zval.is_null() { Some(false) } else { None })
    }
}

impl<'a> FromZval<'a> for &'a str {
    const TYPE: DataType = DataType::String;

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        zval.str()
    }
}

impl<'a> FromZval<'a> for Callable<'a> {
    const TYPE: DataType = DataType::Callable;

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        Callable::new(zval).ok()
    }
}

impl<'a> TryFrom<Zval> for Callable<'a> {
    type Error = Error;

    fn try_from(value: Zval) -> Result<Self> {
        Callable::new_owned(value)
    }
}
