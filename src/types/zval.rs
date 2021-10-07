//! The base value in PHP. A Zval can contain any PHP type, and the type that it
//! contains is determined by a property inside the struct. The content of the
//! Zval is stored in a union.

use std::{convert::TryInto, ffi::c_void, fmt::Debug, ptr};

use crate::{
    binary::Pack,
    boxed::ZBox,
    convert::{FromZval, IntoZval, IntoZvalDyn},
    error::{Error, Result},
    ffi::{
        _zval_struct__bindgen_ty_1, _zval_struct__bindgen_ty_2, zend_is_callable, zend_resource,
        zend_value, zval, zval_ptr_dtor,
    },
    flags::DataType,
    flags::ZvalTypeFlags,
    rc::PhpRc,
    types::{Callable, HashTable, ZendLong, ZendObject, ZendStr},
};

/// A zend value. This is the primary storage container used throughout the Zend
/// engine.
///
/// A zval can be thought of as a Rust enum, a type that can contain different
/// values such as integers, strings, objects etc.
pub type Zval = zval;

// TODO(david): can we make zval send+sync? main problem is that refcounted
// types do not have atomic refcounters, so technically two threads could
// reference the same object and attempt to modify refcounter at the same time.
// need to look into how ZTS works.

// unsafe impl Send for Zval {}
// unsafe impl Sync for Zval {}

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
            self.long().map(|x| x as f64)
        }
    }

    /// Returns the value of the zval as a zend string, if it is a string.
    ///
    /// Note that this functions output will not be the same as
    /// [`string()`](#method.string), as this function does not attempt to
    /// convert other types into a [`String`].
    pub fn zend_str(&self) -> Option<&ZendStr> {
        if self.is_string() {
            unsafe { self.value.str_.as_ref() }
        } else {
            None
        }
    }

    /// Returns the value of the zval if it is a string.
    ///
    /// If the zval does not contain a string, the function will check if it
    /// contains a double or a long, and if so it will convert the value to
    /// a [`String`] and return it. Don't rely on this logic, as there is
    /// potential for this to change to match the output of the [`str()`]
    /// function.
    ///
    /// [`str()`]: #method.str
    pub fn string(&self) -> Option<String> {
        self.str()
            .map(|s| s.to_string())
            .or_else(|| self.double().map(|x| x.to_string()))
    }

    /// Returns the value of the zval if it is a string.
    ///
    /// Note that this functions output will not be the same as
    /// [`string()`](#method.string), as this function does not attempt to
    /// convert other types into a [`String`], as it could not pass back a
    /// [`&str`] in those cases.
    pub fn str(&self) -> Option<&str> {
        self.zend_str().and_then(|zs| zs.as_str())
    }

    /// Returns the value of the zval if it is a string and can be unpacked into
    /// a vector of a given type. Similar to the [`unpack`](https://www.php.net/manual/en/function.unpack.php)
    /// in PHP, except you can only unpack one type.
    ///
    /// # Safety
    ///
    /// There is no way to tell if the data stored in the string is actually of
    /// the given type. The results of this function can also differ from
    /// platform-to-platform due to the different representation of some
    /// types on different platforms. Consult the [`pack`] function
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

    /// Returns an immutable reference to the underlying zval hashtable if the
    /// zval contains an array.
    pub fn array(&self) -> Option<&HashTable> {
        if self.is_array() {
            unsafe { self.value.arr.as_ref() }
        } else {
            None
        }
    }

    /// Returns a mutable reference to the underlying zval hashtable if the zval
    /// contains an array.
    pub fn array_mut(&mut self) -> Option<&mut HashTable> {
        if self.is_array() {
            unsafe { self.value.arr.as_mut() }
        } else {
            None
        }
    }

    /// Returns the value of the zval if it is an object.
    pub fn object(&self) -> Option<&ZendObject> {
        if self.is_object() {
            unsafe { self.value.obj.as_ref() }
        } else {
            None
        }
    }

    /// Returns a mutable reference to the object contained in the [`Zval`], if
    /// any.
    pub fn object_mut(&mut self) -> Option<&mut ZendObject> {
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
    /// The caller must ensure that the pointer contained in the zval is in fact
    /// a pointer to an instance of `T`, as the zval has no way of defining
    /// the type of pointer.
    pub unsafe fn ptr<T>(&self) -> Option<*mut T> {
        if self.is_ptr() {
            Some(self.value.ptr as *mut T)
        } else {
            None
        }
    }

    /// Attempts to call the zval as a callable with a list of arguments to pass
    /// to the function. Note that a thrown exception inside the callable is
    /// not detectable, therefore you should check if the return value is
    /// valid rather than unwrapping. Returns a result containing the return
    /// value of the function, or an error.
    ///
    /// You should not call this function directly, rather through the
    /// [`call_user_func`] macro.
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

    /// Sets the value of the zval as a string. Returns nothing in a result when
    /// successful.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    /// * `persistent` - Whether the string should persist between requests.
    pub fn set_string(&mut self, val: &str, persistent: bool) -> Result<()> {
        self.set_zend_string(ZendStr::new(val, persistent)?);
        Ok(())
    }

    /// Sets the value of the zval as a Zend string.
    ///
    /// # Parameters
    ///
    /// * `val` - String content.
    pub fn set_zend_string(&mut self, val: ZBox<ZendStr>) {
        self.change_type(ZvalTypeFlags::StringEx);
        self.value.str_ = val.into_raw();
    }

    /// Sets the value of the zval as a binary string, which is represented in
    /// Rust as a vector.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    pub fn set_binary<T: Pack>(&mut self, val: Vec<T>) {
        self.change_type(ZvalTypeFlags::StringEx);
        let ptr = T::pack_into(val);
        self.value.str_ = ptr;
    }

    /// Sets the value of the zval as a interned string. Returns nothing in a
    /// result when successful.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    /// * `persistent` - Whether the string should persist between requests.
    pub fn set_interned_string(&mut self, val: &str, persistent: bool) -> Result<()> {
        self.set_zend_string(ZendStr::new_interned(val, persistent)?);
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

    /// Sets the value of the zval as an array. Returns nothing in a result on
    /// success.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    pub fn set_array<T: TryInto<ZBox<HashTable>, Error = Error>>(&mut self, val: T) -> Result<()> {
        self.set_hashtable(val.try_into()?);
        Ok(())
    }

    /// Sets the value of the zval as an array. Returns nothing in a result on
    /// success.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    pub fn set_hashtable(&mut self, val: ZBox<HashTable>) {
        self.change_type(ZvalTypeFlags::ArrayEx);
        self.value.arr = val.into_raw();
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
    /// This is important when copying the value of the zval, as the actual
    /// value will not be copied, but the pointer to the value (string for
    /// example) will be copied.
    pub(crate) fn release(mut self) {
        // NOTE(david): don't use `change_type` here as we are wanting to keep the
        // contents intact.
        self.u1.type_info = ZvalTypeFlags::Null.bits();
    }

    /// Changes the type of the zval, freeing the current contents when
    /// applicable.
    ///
    /// # Parameters
    ///
    /// * `ty` - The new type of the zval.
    fn change_type(&mut self, ty: ZvalTypeFlags) {
        // SAFETY: we have exclusive mutable access to this zval so can free the
        // contents.
        unsafe { zval_ptr_dtor(self) };
        self.u1.type_info = ty.bits();
    }

    /// Extracts some type from a `Zval`.
    ///
    /// This is a wrapper function around `TryFrom`.
    pub fn extract<'a, T>(&'a self) -> Option<T>
    where
        T: FromZval<'a>,
    {
        FromZval::from_zval(self)
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

impl IntoZval for Zval {
    const TYPE: DataType = DataType::Mixed;

    fn set_zval(self, zv: &mut Zval, _: bool) -> Result<()> {
        *zv = self;
        Ok(())
    }
}
