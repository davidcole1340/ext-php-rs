//! The base value in PHP. A Zval can contain any PHP type, and the type that it
//! contains is determined by a property inside the struct. The content of the
//! Zval is stored in a union.

use std::{convert::TryInto, ffi::c_void, fmt::Debug, ptr};

use crate::types::iterable::Iterable;
use crate::types::ZendIterator;
use crate::{
    binary::Pack,
    binary_slice::PackSlice,
    boxed::ZBox,
    convert::{FromZval, FromZvalMut, IntoZval, IntoZvalDyn},
    error::{Error, Result},
    ffi::{
        _zval_struct__bindgen_ty_1, _zval_struct__bindgen_ty_2, zend_is_callable,
        zend_is_identical, zend_is_iterable, zend_resource, zend_value, zval, zval_ptr_dtor,
    },
    flags::DataType,
    flags::ZvalTypeFlags,
    rc::PhpRc,
    types::{ZendCallable, ZendHashTable, ZendLong, ZendObject, ZendStr},
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

    /// Dereference the zval, if it is a reference.
    pub fn dereference(&self) -> &Self {
        self.reference().or_else(|| self.indirect()).unwrap_or(self)
    }

    /// Dereference the zval mutable, if it is a reference.
    pub fn dereference_mut(&mut self) -> &mut Self {
        // TODO: probably more ZTS work is needed here
        if self.is_reference() {
            #[allow(clippy::unwrap_used)]
            return self.reference_mut().unwrap();
        }
        if self.is_indirect() {
            #[allow(clippy::unwrap_used)]
            return self.indirect_mut().unwrap();
        }
        self
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
    /// [`str()`]: #method.str
    pub fn string(&self) -> Option<String> {
        self.str().map(|s| s.to_string())
    }

    /// Returns the value of the zval if it is a string.
    ///
    /// Note that this functions output will not be the same as
    /// [`string()`](#method.string), as this function does not attempt to
    /// convert other types into a [`String`], as it could not pass back a
    /// [`&str`] in those cases.
    pub fn str(&self) -> Option<&str> {
        self.zend_str().and_then(|zs| zs.as_str().ok())
    }

    /// Returns the value of the zval if it is a string and can be unpacked into
    /// a vector of a given type. Similar to the [`unpack`] function in PHP,
    /// except you can only unpack one type.
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
    /// [`unpack`]: https://www.php.net/manual/en/function.unpack.php
    pub fn binary<T: Pack>(&self) -> Option<Vec<T>> {
        self.zend_str().map(T::unpack_into)
    }

    /// Returns the value of the zval if it is a string and can be unpacked into
    /// a slice of a given type. Similar to the [`unpack`] function in PHP,
    /// except you can only unpack one type.
    ///
    /// This function is similar to [`Zval::binary`] except that a slice is
    /// returned instead of a vector, meaning the contents of the string is
    /// not copied.
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
    /// [`unpack`]: https://www.php.net/manual/en/function.unpack.php
    pub fn binary_slice<T: PackSlice>(&self) -> Option<&[T]> {
        self.zend_str().map(T::unpack_into)
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
    pub fn array(&self) -> Option<&ZendHashTable> {
        if self.is_array() {
            unsafe { self.value.arr.as_ref() }
        } else {
            None
        }
    }

    /// Returns a mutable reference to the underlying zval hashtable if the zval
    /// contains an array.
    pub fn array_mut(&mut self) -> Option<&mut ZendHashTable> {
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

    #[inline(always)]
    pub fn try_call_method(&self, name: &str, params: Vec<&dyn IntoZvalDyn>) -> Result<Zval> {
        self.object()
            .ok_or(Error::Object)?
            .try_call_method(name, params)
    }

    /// Returns the value of the zval if it is an internal indirect reference.
    pub fn indirect(&self) -> Option<&Zval> {
        if self.is_indirect() {
            Some(unsafe { &*(self.value.zv as *mut Zval) })
        } else {
            None
        }
    }

    /// Returns a mutable reference to the zval if it is an internal indirect
    /// reference.
    pub fn indirect_mut(&self) -> Option<&mut Zval> {
        if self.is_indirect() {
            Some(unsafe { &mut *(self.value.zv as *mut Zval) })
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
    pub fn callable(&self) -> Option<ZendCallable> {
        // The Zval is checked if it is callable in the `new` function.
        ZendCallable::new(self).ok()
    }

    /// Returns an iterator over the zval if it is traversable.
    pub fn traversable(&self) -> Option<&mut ZendIterator> {
        if self.is_traversable() {
            self.object()?.get_class_entry().get_iterator(self, false)
        } else {
            None
        }
    }

    /// Returns an iterable over the zval if it is an array or traversable. (is
    /// iterable)
    pub fn iterable(&self) -> Option<Iterable> {
        if self.is_iterable() {
            Iterable::from_zval(self)
        } else {
            None
        }
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
    #[inline(always)]
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
        self.get_type() == DataType::True
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

    /// Returns true if the zval is a reference, false otherwise.
    pub fn is_indirect(&self) -> bool {
        self.get_type() == DataType::Indirect
    }

    /// Returns true if the zval is callable, false otherwise.
    pub fn is_callable(&self) -> bool {
        let ptr: *const Self = self;
        unsafe { zend_is_callable(ptr as *mut Self, 0, std::ptr::null_mut()) }
    }

    /// Checks if the zval is identical to another one.
    /// This works like `===` in php.
    ///
    /// # Parameters
    ///
    /// * `other` - The the zval to check identity against.
    pub fn is_identical(&self, other: &Self) -> bool {
        let self_p: *const Self = self;
        let other_p: *const Self = other;
        unsafe { zend_is_identical(self_p as *mut Self, other_p as *mut Self) }
    }

    /// Returns true if the zval is traversable, false otherwise.
    pub fn is_traversable(&self) -> bool {
        match self.object() {
            None => false,
            Some(obj) => obj.is_traversable(),
        }
    }

    /// Returns true if the zval is iterable (array or traversable), false
    /// otherwise.
    pub fn is_iterable(&self) -> bool {
        let ptr: *const Self = self;
        unsafe { zend_is_iterable(ptr as *mut Self) }
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
        self.set_zend_string(ZendStr::new(val, persistent));
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
        self.set_zend_string(ZendStr::new_interned(val, persistent));
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
    pub fn set_array<T: TryInto<ZBox<ZendHashTable>, Error = Error>>(
        &mut self,
        val: T,
    ) -> Result<()> {
        self.set_hashtable(val.try_into()?);
        Ok(())
    }

    /// Sets the value of the zval as an array. Returns nothing in a result on
    /// success.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to set the zval as.
    pub fn set_hashtable(&mut self, val: ZBox<ZendHashTable>) {
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

    /// Coerce the value into a string. Mimics the PHP type coercion rules.
    pub fn coerce_into_string(&mut self) -> Result<()> {
        if self.is_string() {
            return Ok(());
        }

        if let Some(val) = self.string() {
            self.set_string(&val, false)?;
            return Ok(());
        } else if let Some(val) = self.double() {
            self.set_string(&val.to_string(), false)?;
            return Ok(());
        } else if let Some(val) = self.long() {
            self.set_string(&val.to_string(), false)?;
            return Ok(());
        } else if let Some(val) = self.bool() {
            self.set_string(if val { "1" } else { "0" }, false)?;
            return Ok(());
        } else if self.is_array() {
            self.set_string("Array", false)?;
            return Ok(());
        }

        Err(Error::ZvalConversion(self.get_type()))
    }

    /// Coerce the value into a boolean. Mimics the PHP type coercion rules.
    pub fn coerce_into_bool(&mut self) -> Result<()> {
        if self.is_bool() {
            return Ok(());
        }

        if let Some(val) = self.long() {
            self.set_bool(val != 0 );
            return Ok(());
        } else if let Some(val) = self.double() {
            self.set_bool(val != 0.0 );
            return Ok(());
        } else if let Some(val) = self.string() {
            self.set_bool(val != "0" && val != "");
            return Ok(());
        } else if let Some(val) = self.array() {
            self.set_bool(val.len() != 0);
        }

        Err(Error::ZvalConversion(self.get_type()))
    }

    /// Coerce the value into a long. Mimics the PHP type coercion rules.
    pub fn coerce_into_long(&mut self) -> Result<()> {
        if self.is_long() {
            return Ok(());
        }

        if let Some(val) = self.double() {
            self.set_long(val as i64);
            return Ok(());
        } else if let Some(val) = self.string() {
            self.set_long(val.parse::<i64>().map_err(|_| Error::ZvalConversion(self.get_type()))?);
            return Ok(());
        } else if let Some(val) = self.array() {
            self.set_long(if val.len() > 0 { 1 } else { 0 });
        }

        Err(Error::ZvalConversion(self.get_type()))
    }

    /// Coerce the value into a double. Mimics the PHP type coercion rules.
    pub fn coerce_into_double(&mut self) -> Result<()> {
        if self.is_double() {
            return Ok(());
        }

        if let Some(val) = self.long() {
            self.set_double(val as f64);
            return Ok(());
        } else if let Some(val) = self.string() {
            self.set_double(val.parse::<f64>().map_err(|_| Error::ZvalConversion(self.get_type()))?);
            return Ok(());
        } else if let Some(val) = self.array() {
            self.set_double(if val.len() > 0 { 1.0 } else { 0.0 });
        }

        Err(Error::ZvalConversion(self.get_type()))
    }

    /// Creates a shallow clone of the [`Zval`].
    ///
    /// This copies the contents of the [`Zval`], and increments the reference
    /// counter of the underlying value (if it is reference counted).
    ///
    /// For example, if the zval contains a long, it will simply copy the value.
    /// However, if the zval contains an object, the new zval will point to the
    /// same object, and the objects reference counter will be incremented.
    ///
    /// # Returns
    ///
    /// The cloned zval.
    pub fn shallow_clone(&self) -> Zval {
        let mut new = Zval::new();
        new.u1 = self.u1;
        new.value = self.value;

        // SAFETY: `u1` union is only used for easier bitmasking. It is valid to read
        // from either of the variants.
        //
        // SAFETY: If the value if refcounted (`self.u1.type_info & Z_TYPE_FLAGS_MASK`)
        // then it is valid to dereference `self.value.counted`.
        unsafe {
            let flags = ZvalTypeFlags::from_bits_retain(self.u1.type_info);
            if flags.contains(ZvalTypeFlags::RefCounted) {
                (*self.value.counted).gc.refcount += 1;
            }
        }

        new
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
            DataType::Indirect => field!(self.indirect()),
            DataType::Iterable => field!(self.iterable()),
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

impl<'a> FromZval<'a> for &'a Zval {
    const TYPE: DataType = DataType::Mixed;

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        Some(zval)
    }
}

impl<'a> FromZvalMut<'a> for &'a mut Zval {
    const TYPE: DataType = DataType::Mixed;

    fn from_zval_mut(zval: &'a mut Zval) -> Option<Self> {
        Some(zval)
    }
}
