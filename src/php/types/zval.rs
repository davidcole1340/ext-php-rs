//! The base value in PHP. A Zval can contain any PHP type, and the type that it contains is
//! determined by a property inside the struct. The content of the Zval is stored in a union.

use core::slice;
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    fmt::Debug,
    ptr,
};

use crate::{
    bindings::{
        _zval_struct__bindgen_ty_1, _zval_struct__bindgen_ty_2, ext_php_rs_zend_string_release,
        zend_is_callable, zend_resource, zend_value, zval,
    },
    errors::{Error, Result},
    php::pack::Pack,
};

use crate::php::{
    enums::DataType,
    flags::ZvalTypeFlags,
    types::{long::ZendLong, string::ZendString},
};

use super::{array::ZendHashTable, callable::Callable, object::ZendObject};

/// Zend value. Represents most data types that are in the Zend engine.
pub type Zval = zval;

unsafe impl Send for Zval {}
unsafe impl Sync for Zval {}

impl<'a> Zval {
    /// Creates a new, empty zval.
    pub(crate) const fn new() -> Self {
        Self {
            value: zend_value {
                ptr: ptr::null_mut(),
            },
            u1: _zval_struct__bindgen_ty_1 {
                type_info: DataType::Null as u32,
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
            self.long().map(|x| x as f64)
        }
    }

    /// Returns the value of the zval if it is a string.
    ///
    /// If the zval does not contain a string, the function will check if it contains a
    /// double or a long, and if so it will convert the value to a [`String`] and return it.
    /// Don't rely on this logic, as there is potential for this to change to match the output
    /// of the [`str()`](#method.str) function.
    pub fn string(&self) -> Option<String> {
        self.str()
            .map(|s| s.to_string())
            .or_else(|| self.double().map(|x| x.to_string()))
    }

    /// Returns the value of the zval if it is a string.
    ///
    /// Note that this functions output will not be the same as [`string()`](#method.string), as
    /// this function does not attempt to convert other types into a [`String`], as it could not
    /// pass back a [`&str`] in those cases.
    pub fn str(&'a self) -> Option<&'a str> {
        if self.is_string() {
            // SAFETY: Zend strings have a length that we know we can read.
            // By reading this many bytes we will not run into any issues.
            //
            // We can safely cast our *const c_char into a *const u8 as both
            // only occupy one byte.
            unsafe {
                std::str::from_utf8(slice::from_raw_parts(
                    (*self.value.str_).val.as_ptr() as *const u8,
                    (*self.value.str_).len as usize,
                ))
                .ok()
            }
        } else {
            None
        }
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

    /// Returns the value of the zval if it is an array.
    pub fn array(&self) -> Option<ZendHashTable<'a>> {
        if self.is_array() {
            unsafe { ZendHashTable::from_ptr(self.value.arr, false) }.ok()
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
    pub fn reference(&self) -> Option<&mut Zval> {
        if self.is_reference() {
            Some(&mut unsafe { self.value.ref_.as_mut() }?.val)
        } else {
            None
        }
    }

    /// Returns the value of the zval if it is callable.
    pub fn callable(&'a self) -> Option<Callable<'a>> {
        // The Zval is checked if it is callable in the `new` function.
        Callable::new(self).ok()
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
    pub fn get_type(&self) -> Result<DataType> {
        DataType::try_from(unsafe { self.u1.v.type_ } as u32)
    }

    /// Returns true if the zval is a long, false otherwise.
    pub fn is_long(&self) -> bool {
        unsafe { self.u1.v.type_ == DataType::Long as u8 }
    }

    /// Returns true if the zval is null, false otherwise.
    pub fn is_null(&self) -> bool {
        unsafe { self.u1.v.type_ == DataType::Null as u8 }
    }

    /// Returns true if the zval is true, false otherwise.
    pub fn is_true(&self) -> bool {
        unsafe { self.u1.v.type_ == DataType::True as u8 }
    }

    /// Returns true if the zval is false, false otherwise.
    pub fn is_false(&self) -> bool {
        unsafe { self.u1.v.type_ == DataType::False as u8 }
    }

    /// Returns true if the zval is a bool, false otherwise.
    pub fn is_bool(&self) -> bool {
        self.is_true() || self.is_false()
    }

    /// Returns true if the zval is a double, false otherwise.
    pub fn is_double(&self) -> bool {
        unsafe { self.u1.v.type_ == DataType::Double as u8 }
    }

    /// Returns true if the zval is a string, false otherwise.
    pub fn is_string(&self) -> bool {
        unsafe { self.u1.v.type_ == DataType::String as u8 }
    }

    /// Returns true if the zval is a resource, false otherwise.
    pub fn is_resource(&self) -> bool {
        unsafe { self.u1.v.type_ == DataType::Resource as u8 }
    }

    /// Returns true if the zval is an array, false otherwise.
    pub fn is_array(&self) -> bool {
        unsafe { self.u1.v.type_ == DataType::Array as u8 }
    }

    /// Returns true if the zval is an object, false otherwise.
    pub fn is_object(&self) -> bool {
        unsafe { self.u1.v.type_ == DataType::Object as u8 }
    }

    /// Returns true if the zval is a reference, false otherwise.
    pub fn is_reference(&self) -> bool {
        unsafe { self.u1.v.type_ == DataType::Reference as u8 }
    }

    /// Returns true if the zval is callable, false otherwise.
    pub fn is_callable(&self) -> bool {
        let ptr: *const Self = self;
        unsafe { zend_is_callable(ptr as *mut Self, 0, std::ptr::null_mut()) }
    }

    /// Sets the value of the zval as a string. Returns nothing in a result when successful.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    /// * `persistent` - Whether the string should persist between requests.
    pub fn set_string(&mut self, val: &str, persistent: bool) -> Result<()> {
        let zend_str = ZendString::new(val, persistent)?;
        self.value.str_ = zend_str.release();
        self.u1.type_info = ZvalTypeFlags::StringEx.bits();
        Ok(())
    }

    /// Sets the value of the zval as a binary string, which is represented in Rust as an array.
    ///
    /// The data is cloned before being packed into a string.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    pub fn set_binary<T: Pack, U: AsRef<[T]>>(&mut self, val: U) {
        let ptr = T::pack_into(val.as_ref().to_vec());
        self.value.str_ = ptr;
        self.u1.type_info = ZvalTypeFlags::StringEx.bits();
    }

    /// Sets the value of the zval as a interned string. Returns nothing in a result when successful.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    pub fn set_interned_string(&mut self, val: &str) -> Result<()> {
        let zend_str = ZendString::new_interned(val)?;
        self.value.str_ = zend_str.release();
        self.u1.type_info = ZvalTypeFlags::InternedStringEx.bits();
        Ok(())
    }

    /// Sets the value of the zval as a long.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    pub fn set_long<T: Into<ZendLong>>(&mut self, val: T) {
        self.value.lval = val.into();
        self.u1.type_info = ZvalTypeFlags::Long.bits();
    }

    /// Sets the value of the zval as a double.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    pub fn set_double<T: Into<f64>>(&mut self, val: T) {
        self.value.dval = val.into();
        self.u1.type_info = ZvalTypeFlags::Double.bits();
    }

    /// Sets the value of the zval as a boolean.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    pub fn set_bool<T: Into<bool>>(&mut self, val: T) {
        self.u1.type_info = if val.into() {
            DataType::True as u32
        } else {
            DataType::False as u32
        };
    }

    /// Sets the value of the zval as null.
    ///
    /// This is the default of a zval.
    pub fn set_null(&mut self) {
        self.u1.type_info = ZvalTypeFlags::Null.bits();
    }

    /// Sets the value of the zval as a resource.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    pub fn set_resource(&mut self, val: *mut zend_resource) {
        self.u1.type_info = ZvalTypeFlags::ResourceEx.bits();
        self.value.res = val;
    }

    /// Sets the value of the zval as a reference to an object.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    pub fn set_object(&mut self, val: &mut ZendObject) {
        val.refcount_inc();
        self.u1.type_info = ZvalTypeFlags::ObjectEx.bits();
        self.value.obj = (val as *const ZendObject) as *mut ZendObject;
    }

    /// Sets the value of the zval as an array. Returns nothng in a result on success.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    pub fn set_array(&mut self, val: ZendHashTable) {
        self.u1.type_info = ZvalTypeFlags::ArrayEx.bits();
        self.value.arr = val.into_ptr();
    }

    /// Used to drop the Zval but keep the value of the zval intact.
    ///
    /// This is important when copying the value of the zval, as the actual value
    /// will not be copied, but the pointer to the value (string for example) will be
    /// copied.
    pub(crate) fn release(mut self) {
        self.u1.type_info = ZvalTypeFlags::Null.bits();
    }
}

impl Debug for Zval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_struct("Zval");
        let ty = self.get_type();
        dbg.field("type", &ty);

        if let Ok(ty) = ty {
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
                DataType::String => field!(self.string()),
                DataType::Array => field!(self.array()),
                DataType::Object => field!(self.object()),
                DataType::Resource => field!(self.resource()),
                DataType::Reference => field!(self.reference()),
                DataType::Callable => field!(self.string()),
                DataType::ConstantExpression => field!(Option::<()>::None),
                DataType::Void => field!(Option::<()>::None),
                DataType::Bool => field!(self.bool()),
            };
        }

        dbg.finish()
    }
}

impl Drop for Zval {
    fn drop(&mut self) {
        if self.is_string() {
            unsafe { ext_php_rs_zend_string_release(self.value.str_) };
        }
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
    fn as_zval(self, persistent: bool) -> Result<Zval> {
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

/// An object-safe version of the [`IntoZval`] trait.
///
/// This trait is automatically implemented on any type that implements both [`IntoZval`] and [`Clone`].
/// You avoid implementing this trait directly, rather implement these two other traits.
pub trait IntoZvalDyn {
    /// Converts a Rust primitive type into a Zval. Returns a result containing the Zval if
    /// successful.
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
        self.clone().as_zval(persistent)
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
    const TYPE: DataType = DataType::Null;

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

impl<'a> IntoZval for ZendHashTable<'a> {
    const TYPE: DataType = DataType::Array;

    fn set_zval(self, zv: &mut Zval, _: bool) -> Result<()> {
        zv.set_array(self.clone());
        Ok(())
    }
}

impl<T> IntoZval for Vec<T>
where
    T: IntoZval,
{
    const TYPE: DataType = DataType::Array;

    fn set_zval(self, zv: &mut Zval, _: bool) -> Result<()> {
        let hm = self
            .try_into()
            .map_err(|_| Error::ZvalConversion(DataType::Array))?;
        zv.set_array(hm);
        Ok(())
    }
}

impl<K, V> IntoZval for HashMap<K, V>
where
    K: AsRef<str>,
    V: IntoZval,
{
    const TYPE: DataType = DataType::Array;

    fn set_zval(self, zv: &mut Zval, _: bool) -> Result<()> {
        let hm = self.try_into()?;
        zv.set_array(hm);
        Ok(())
    }
}

/// Allows zvals to be converted into Rust types in a fallible way. Reciprocal of the [`IntoZval`]
/// trait.
///
/// This trait requires the [`TryFrom`] trait to be implemented. All this trait does is contain the
/// type of data that is expected when parsing the value, which is used when parsing arguments.
pub trait FromZval<'a>: TryFrom<&'a Zval> {
    /// The corresponding type of the implemented value in PHP.
    const TYPE: DataType;
}

impl<'a, T> FromZval<'a> for Option<T>
where
    T: FromZval<'a>,
{
    const TYPE: DataType = T::TYPE;
}

// Converting to an option is infallible.
impl<'a, T> From<&'a Zval> for Option<T>
where
    T: FromZval<'a>,
{
    fn from(val: &'a Zval) -> Self {
        val.try_into().ok()
    }
}

macro_rules! try_from_zval {
    ($type: ty, $fn: ident, $dt: ident) => {
        impl<'a> FromZval<'a> for $type {
            const TYPE: DataType = DataType::$dt;
        }

        impl TryFrom<&Zval> for $type {
            type Error = Error;

            fn try_from(value: &Zval) -> Result<Self> {
                value
                    .$fn()
                    .and_then(|val| val.try_into().ok())
                    .ok_or(Error::ZvalConversion(value.get_type()?))
            }
        }

        impl TryFrom<Zval> for $type {
            type Error = Error;

            fn try_from(value: Zval) -> Result<Self> {
                (&value).try_into()
            }
        }
    };
}

try_from_zval!(i8, long, Long);
try_from_zval!(i16, long, Long);
try_from_zval!(i32, long, Long);
try_from_zval!(i64, long, Long);

try_from_zval!(u8, long, Long);
try_from_zval!(u16, long, Long);
try_from_zval!(u32, long, Long);
try_from_zval!(u64, long, Long);

try_from_zval!(usize, long, Long);
try_from_zval!(isize, long, Long);

try_from_zval!(f64, double, Double);
try_from_zval!(bool, bool, Bool);
try_from_zval!(String, string, String);

impl<'a> FromZval<'a> for f32 {
    const TYPE: DataType = DataType::Double;
}

impl<'a> TryFrom<&'a Zval> for f32 {
    type Error = Error;

    fn try_from(value: &'a Zval) -> Result<Self> {
        value
            .double()
            .map(|v| v as f32)
            .ok_or(Error::ZvalConversion(value.get_type()?))
    }
}

impl<'a> FromZval<'a> for &'a str {
    const TYPE: DataType = DataType::String;
}

impl<'a> TryFrom<&'a Zval> for &'a str {
    type Error = Error;

    fn try_from(value: &'a Zval) -> Result<Self> {
        value.str().ok_or(Error::ZvalConversion(value.get_type()?))
    }
}

impl<'a> FromZval<'a> for ZendHashTable<'a> {
    const TYPE: DataType = DataType::Array;
}

impl<'a> TryFrom<&'a Zval> for ZendHashTable<'a> {
    type Error = Error;

    fn try_from(value: &'a Zval) -> Result<Self> {
        value
            .array()
            .ok_or(Error::ZvalConversion(value.get_type()?))
    }
}

impl<'a, T> FromZval<'a> for Vec<T>
where
    T: FromZval<'a>,
{
    const TYPE: DataType = DataType::Array;
}

impl<'a, T> TryFrom<&'a Zval> for Vec<T>
where
    T: FromZval<'a>,
{
    type Error = Error;

    fn try_from(value: &'a Zval) -> Result<Self> {
        value
            .array()
            .ok_or(Error::ZvalConversion(value.get_type()?))?
            .try_into()
    }
}

impl<'a, T> TryFrom<Zval> for Vec<T>
where
    T: FromZval<'a>,
{
    type Error = Error;

    fn try_from(value: Zval) -> Result<Self> {
        value
            .array()
            .ok_or(Error::ZvalConversion(value.get_type()?))?
            .try_into()
    }
}

impl<'a, T> FromZval<'a> for HashMap<String, T>
where
    T: FromZval<'a>,
{
    const TYPE: DataType = DataType::Array;
}

impl<'a, T> TryFrom<&'a Zval> for HashMap<String, T>
where
    T: FromZval<'a>,
{
    type Error = Error;

    fn try_from(value: &'a Zval) -> Result<Self> {
        value
            .array()
            .ok_or(Error::ZvalConversion(value.get_type()?))?
            .try_into()
    }
}

impl<'a, T> TryFrom<Zval> for HashMap<String, T>
where
    T: FromZval<'a>,
{
    type Error = Error;

    fn try_from(value: Zval) -> Result<Self> {
        value
            .array()
            .ok_or(Error::ZvalConversion(value.get_type()?))?
            .try_into()
    }
}

impl<'a> FromZval<'a> for Callable<'a> {
    const TYPE: DataType = DataType::Callable;
}

impl<'a> TryFrom<&'a Zval> for Callable<'a> {
    type Error = Error;

    fn try_from(value: &'a Zval) -> Result<Self> {
        Callable::new(value)
    }
}

impl<'a> TryFrom<Zval> for Callable<'a> {
    type Error = Error;

    fn try_from(value: Zval) -> Result<Self> {
        Callable::new_owned(value)
    }
}
