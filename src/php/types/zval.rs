use core::slice;
use std::{convert::TryFrom, ptr};

use crate::bindings::{
    _zval_struct__bindgen_ty_1, _zval_struct__bindgen_ty_2, zend_object, zend_resource, zend_value,
    zval, IS_INTERNED_STRING_EX, IS_STRING_EX,
};

use crate::php::{
    enums::DataType,
    types::{long::ZendLong, string::ZendString},
};

use super::array::ZendHashTable;

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
                let len = (*self.value.str).len;
                let ptr = (*self.value.str).val.as_ptr() as *const u8;
                let _str = std::str::from_utf8(slice::from_raw_parts(ptr, len as usize)).unwrap();

                Some(_str.to_string())
            }
        } else {
            self.double().map(|x| x.to_string())
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
            Some(ZendHashTable::from_ptr(unsafe { self.value.arr }))
        } else {
            None
        }
    }

    /// Returns the value of the zval if it is an object.
    pub fn object(&self) -> Option<*mut zend_object> {
        // TODO: Can we improve this function? I haven't done much research into
        // objects so I don't know if this is the optimal way to return this.
        if self.is_object() {
            Some(unsafe { self.value.obj })
        } else {
            None
        }
    }

    /// Returns the value of the zval if it is a reference.
    pub fn reference(&self) -> Option<Zval> {
        if self.is_reference() {
            Some(unsafe { (*self.value.ref_).val })
        } else {
            None
        }
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
}

/// Used to set the value of the zval.
///
/// This needs to be a trait to be implemented on a pointer that
/// points to a zval.
pub trait SetZval {
    /// Sets the value of the zval as a string.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    fn set_string<S>(&mut self, val: S) -> Result<(), String>
    where
        S: AsRef<str>;

    /// Sets the value of the zval as a long.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    fn set_long(&mut self, val: ZendLong) -> Result<(), String>;

    /// Sets the value of the zval as a double.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    fn set_double(&mut self, val: f64) -> Result<(), String>;

    /// Sets the value of the zval as a boolean.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    fn set_bool(&mut self, val: bool) -> Result<(), String>;

    /// Sets the value of the zval as null.
    /// This is the default of a zval.
    fn set_null(&mut self) -> Result<(), String>;

    /// Sets the value of the zval as a resource.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    fn set_resource(&mut self, val: *mut zend_resource) -> Result<(), String>;

    /// Sets the value of the zval as an object.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    /// * `copy` - Whether to copy the object or pass as a reference.
    fn set_object(&mut self, val: *mut zend_object, copy: bool) -> Result<(), String>;

    /// Sets the value of the zval as an array.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    fn set_array<V>(&mut self, val: V) -> Result<(), String>
    where
        V: Into<ZendHashTable>;
}

impl SetZval for Zval {
    fn set_string<S>(&mut self, val: S) -> Result<(), String>
    where
        S: AsRef<str>,
    {
        let zend_str = ZendString::new(val);
        self.value.str = zend_str;
        self.u1.type_info = if unsafe { zend_str.as_ref().unwrap().is_interned() } {
            IS_INTERNED_STRING_EX
        } else {
            IS_STRING_EX
        };
        Ok(())
    }

    fn set_long(&mut self, val: ZendLong) -> Result<(), String> {
        self.value.lval = val;
        self.u1.type_info = DataType::Long as u32;
        Ok(())
    }

    fn set_double(&mut self, val: f64) -> Result<(), String> {
        self.value.dval = val;
        self.u1.type_info = DataType::Double as u32;
        Ok(())
    }

    fn set_bool(&mut self, val: bool) -> Result<(), String> {
        self.u1.type_info = if val {
            DataType::True as u32
        } else {
            DataType::False as u32
        };
        Ok(())
    }

    fn set_null(&mut self) -> Result<(), String> {
        self.u1.type_info = DataType::Null as u32;
        Ok(())
    }

    fn set_resource(&mut self, val: *mut zend_resource) -> Result<(), String> {
        self.u1.type_info = DataType::Resource as u32;
        self.value.res = val;
        Ok(())
    }

    fn set_object(&mut self, val: *mut zend_object, _copy: bool) -> Result<(), String> {
        self.u1.type_info = DataType::Object as u32;
        self.value.obj = val;
        Ok(())
    }

    fn set_array<V>(&mut self, val: V) -> Result<(), String>
    where
        V: Into<ZendHashTable>,
    {
        self.u1.type_info = DataType::Array as u32;
        self.value.arr = val.into().into_ptr();
        Ok(())
    }
}

impl SetZval for *mut Zval {
    fn set_string<S>(&mut self, val: S) -> Result<(), String>
    where
        S: AsRef<str>,
    {
        let _self = match unsafe { self.as_mut() } {
            Some(val) => val,
            None => {
                return Err(String::from(
                    "Could not retrieve mutable reference of zend value.",
                ))
            }
        };

        _self.set_string(val)
    }

    fn set_long(&mut self, val: ZendLong) -> Result<(), String> {
        let _self = match unsafe { self.as_mut() } {
            Some(val) => val,
            None => {
                return Err(String::from(
                    "Could not retrieve mutable reference of zend value.",
                ))
            }
        };

        _self.set_long(val)
    }

    fn set_double(&mut self, val: f64) -> Result<(), String> {
        let _self = match unsafe { self.as_mut() } {
            Some(val) => val,
            None => {
                return Err(String::from(
                    "Could not retrieve mutable reference of zend value.",
                ))
            }
        };

        _self.set_double(val)
    }

    fn set_bool(&mut self, val: bool) -> Result<(), String> {
        let _self = match unsafe { self.as_mut() } {
            Some(val) => val,
            None => {
                return Err(String::from(
                    "Could not retrieve mutable reference of zend value.",
                ))
            }
        };

        _self.set_bool(val)
    }

    fn set_null(&mut self) -> Result<(), String> {
        let _self = match unsafe { self.as_mut() } {
            Some(val) => val,
            None => {
                return Err(String::from(
                    "Could not retrieve mutable reference of zend value.",
                ))
            }
        };

        _self.set_null()
    }

    fn set_resource(&mut self, val: *mut zend_resource) -> Result<(), String> {
        let _self = match unsafe { self.as_mut() } {
            Some(val) => val,
            None => {
                return Err(String::from(
                    "Could not retrieve mutable reference of zend value.",
                ))
            }
        };

        _self.set_resource(val)
    }

    fn set_object(&mut self, val: *mut zend_object, _copy: bool) -> Result<(), String> {
        let _self = match unsafe { self.as_mut() } {
            Some(val) => val,
            None => {
                return Err(String::from(
                    "Could not retrieve mutable reference of zend value.",
                ))
            }
        };

        _self.set_object(val, _copy)
    }

    fn set_array<V>(&mut self, val: V) -> Result<(), String>
    where
        V: Into<ZendHashTable>,
    {
        let _self = match unsafe { self.as_mut() } {
            Some(val) => val,
            None => {
                return Err(String::from(
                    "Could not retrieve mutable reference of zend value.",
                ))
            }
        };

        _self.set_array(val)
    }
}

impl TryFrom<&Zval> for ZendLong {
    type Error = ();
    fn try_from(value: &Zval) -> Result<Self, Self::Error> {
        match value.long() {
            Some(val) => Ok(val),
            _ => Err(()),
        }
    }
}

impl TryFrom<&Zval> for bool {
    type Error = ();
    fn try_from(value: &Zval) -> Result<Self, Self::Error> {
        match value.bool() {
            Some(val) => Ok(val),
            _ => Err(()),
        }
    }
}

impl TryFrom<&Zval> for f64 {
    type Error = ();
    fn try_from(value: &Zval) -> Result<Self, Self::Error> {
        match value.double() {
            Some(val) => Ok(val),
            _ => Err(()),
        }
    }
}

impl TryFrom<&Zval> for String {
    type Error = ();
    fn try_from(value: &Zval) -> Result<Self, Self::Error> {
        match value.string() {
            Some(val) => Ok(val),
            _ => Err(()),
        }
    }
}

impl<'a, 'b> TryFrom<&'b Zval> for ZendHashTable {
    type Error = ();
    fn try_from(value: &'b Zval) -> Result<Self, Self::Error> {
        match value.array() {
            Some(val) => Ok(val),
            _ => Err(()),
        }
    }
}

impl From<ZendLong> for Zval {
    fn from(val: ZendLong) -> Self {
        let mut zv = Self::new();
        zv.set_long(val).unwrap(); // this can never fail
        zv
    }
}

impl From<bool> for Zval {
    fn from(val: bool) -> Self {
        let mut zv = Self::new();
        zv.set_bool(val).unwrap(); // this can never fail
        zv
    }
}
impl From<f64> for Zval {
    fn from(val: f64) -> Self {
        let mut zv = Self::new();
        zv.set_double(val).unwrap(); // this can never fail
        zv
    }
}

impl From<String> for Zval {
    fn from(val: String) -> Self {
        let mut zv = Self::new();
        zv.set_string(val).unwrap(); // this can never fail
        zv
    }
}

impl From<&str> for Zval {
    fn from(val: &str) -> Self {
        let mut zv = Self::new();
        zv.set_string(val).unwrap(); // this can never fail
        zv
    }
}
