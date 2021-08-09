//! The base value in PHP. A Zval can contain any PHP type, and the type that it contains is
//! determined by a property inside the struct. The content of the Zval is stored in a union.

use core::slice;
use std::{convert::TryFrom, fmt::Debug, ptr};

use crate::{
    bindings::{
        _call_user_function_impl, _zval_struct__bindgen_ty_1, _zval_struct__bindgen_ty_2,
        ext_php_rs_zend_string_release, zend_is_callable, zend_resource, zend_value, zval,
    },
    errors::{Error, Result},
    php::pack::Pack,
};

use crate::php::{
    enums::DataType,
    flags::ZvalTypeFlags,
    types::{long::ZendLong, string::ZendString},
};

use super::{array::ZendHashTable, object::ZendObject};

/// Zend value. Represents most data types that are in the Zend engine.
pub type Zval = zval;

impl<'a> Zval {
    /// Creates a new, empty zval.
    pub(crate) fn new() -> Self {
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
    pub fn string(&self) -> Option<String> {
        if self.is_string() {
            // SAFETY: Zend strings have a length that we know we can read.
            // By reading this many bytes we will not run into any issues.
            //
            // We can safely cast our *const c_char into a *const u8 as both
            // only occupy one byte.
            unsafe {
                let _str = std::str::from_utf8(slice::from_raw_parts(
                    (*self.value.str_).val.as_ptr() as *const u8,
                    (*self.value.str_).len as usize,
                ))
                .ok()?;

                Some(_str.to_string())
            }
        } else {
            self.double().map(|x| x.to_string())
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
    /// representation of some types on different platforms. Consult the [`pack`](https://www.php.net/manual/en/function.pack.php)
    /// function documentation for more details.
    pub unsafe fn binary<T: Pack>(&self) -> Option<Vec<T>> {
        if self.is_string() {
            Some(T::unpack_into(self.value.str_.as_ref()?))
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
    pub fn array(&self) -> Option<ZendHashTable> {
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

    /// Attempts to call the argument as a callable with a list of arguments to pass to the function.
    /// Note that a thrown exception inside the callable is not detectable, therefore you should
    /// check if the return value is valid rather than unwrapping. Returns a result containing the
    /// return value of the function, or an error.
    ///
    /// You should not call this function directly, rather through the [`call_user_func`] macro.
    ///
    /// # Parameters
    ///
    /// * `params` - A list of parameters to call the function with.
    pub fn try_call(&self, params: Vec<&dyn IntoZval>) -> Result<Zval> {
        let mut retval = Zval::new();
        let len = params.len();
        let params = params
            .into_iter()
            .map(|val| val.as_zval(false))
            .collect::<Result<Vec<_>>>()?;
        let packed = Box::into_raw(params.into_boxed_slice()) as *mut Self;
        let ptr: *const Self = self;

        if !self.is_callable() {
            return Err(Error::Callable);
        }

        let result = unsafe {
            _call_user_function_impl(
                std::ptr::null_mut(),
                ptr as *mut Self,
                &mut retval,
                len as _,
                packed,
                std::ptr::null_mut(),
            )
        };

        // SAFETY: We just boxed this vector, and the `_call_user_function_impl` does not modify the parameters.
        // We can safely reclaim the memory knowing it will have the same length and size.
        // If any parameters are zend strings, they must be released.
        unsafe {
            let params = Vec::from_raw_parts(packed, len, len);

            for param in params {
                if param.is_string() {
                    ext_php_rs_zend_string_release(param.value.str_);
                }
            }
        };

        if result < 0 {
            Err(Error::Callable)
        } else {
            Ok(retval)
        }
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
    pub fn set_string<S>(&mut self, val: S, persistent: bool) -> Result<()>
    where
        S: AsRef<str>,
    {
        let zend_str = ZendString::new(val, persistent)?;
        self.value.str_ = zend_str.release();
        self.u1.type_info = ZvalTypeFlags::StringEx.bits();
        Ok(())
    }

    /// Sets the value of the zval as a binary string.
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
    pub fn set_interned_string<S>(&mut self, val: S) -> Result<()>
    where
        S: AsRef<str>,
    {
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
    pub fn set_double<T: Into<libc::c_double>>(&mut self, val: T) {
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

    /// Sets the value of the zval as an object.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    /// * `copy` - Whether to copy the object or pass as a reference.
    pub fn set_object(&mut self, val: &ZendObject, _copy: bool) {
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

    /// Sets the reference count of the Zval.
    pub(crate) fn set_refcount(&mut self, rc: u32) {
        unsafe { (*self.value.counted).gc.refcount = rc }
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
            };
        }

        dbg.finish()
    }
}

/// Provides implementations for converting Rust primitive types into PHP zvals. Alternative to the
/// built-in Rust [`From`] and [`TryFrom`] implementations, allowing the caller to specify whether
/// the Zval contents will persist between requests.
pub trait IntoZval {
    /// Converts a Rust primitive type into a Zval. Returns a result containing the Zval if
    /// successful.
    ///
    /// # Parameters
    ///
    /// * `persistent` - Whether the contents of the Zval will persist between requests.
    fn as_zval(&self, persistent: bool) -> Result<Zval> {
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
    fn set_zval(&self, zv: &mut Zval, persistent: bool) -> Result<()>;
}

macro_rules! try_from_zval {
    ($type: ty, $fn: ident) => {
        impl TryFrom<&Zval> for $type {
            type Error = Error;

            fn try_from(value: &Zval) -> Result<Self> {
                match value.$fn() {
                    Some(v) => match <$type>::try_from(v) {
                        Ok(v) => Ok(v),
                        Err(_) => Err(Error::ZvalConversion(value.get_type()?)),
                    },
                    _ => Err(Error::ZvalConversion(value.get_type()?)),
                }
            }
        }
    };
}

try_from_zval!(i8, long);
try_from_zval!(i16, long);
try_from_zval!(i32, long);
try_from_zval!(i64, long);

try_from_zval!(u8, long);
try_from_zval!(u16, long);
try_from_zval!(u32, long);
try_from_zval!(u64, long);

try_from_zval!(usize, long);
try_from_zval!(isize, long);

try_from_zval!(f64, double);
try_from_zval!(bool, bool);
try_from_zval!(String, string);
try_from_zval!(ZendHashTable, array);

/// Implements the trait `Into<T>` on Zval for a given type.
macro_rules! into_zval {
    ($type: ty, $fn: ident) => {
        impl From<$type> for Zval {
            fn from(val: $type) -> Self {
                let mut zv = Self::new();
                zv.$fn(val);
                zv
            }
        }

        impl IntoZval for $type {
            fn set_zval(&self, zv: &mut Zval, _: bool) -> Result<()> {
                zv.$fn(*self);
                Ok(())
            }
        }
    };
}

into_zval!(i8, set_long);
into_zval!(i16, set_long);
into_zval!(i32, set_long);
into_zval!(i64, set_long);

into_zval!(u8, set_long);
into_zval!(u16, set_long);
into_zval!(u32, set_long);

into_zval!(f32, set_double);
into_zval!(f64, set_double);

into_zval!(bool, set_bool);

macro_rules! try_into_zval_str {
    ($type: ty) => {
        impl TryFrom<$type> for Zval {
            type Error = Error;

            fn try_from(value: $type) -> Result<Self> {
                let mut zv = Self::new();
                zv.set_string(value, false)?;
                Ok(zv)
            }
        }

        impl IntoZval for $type {
            fn set_zval(&self, zv: &mut Zval, persistent: bool) -> Result<()> {
                zv.set_string(self, persistent)
            }
        }
    };
}

try_into_zval_str!(String);
try_into_zval_str!(&String);
try_into_zval_str!(&str);
