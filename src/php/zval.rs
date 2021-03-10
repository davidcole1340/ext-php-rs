use core::slice;

use crate::bindings::{zend_object, zend_resource, zval};

use super::{enums::DataType, types::ZendLong};

/// Zend value. Represents most data types that are in the Zend engine.
pub type Zval = zval;

impl Zval {
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
