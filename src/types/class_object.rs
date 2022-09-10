//! Represents an object in PHP. Allows for overriding the internal object used
//! by classes, allowing users to store Rust data inside a PHP object.

use std::{
    fmt::Debug,
    mem,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
};

use crate::{
    boxed::{ZBox, ZBoxable},
    class::RegisteredClass,
    convert::{FromZendObject, FromZendObjectMut, FromZval, FromZvalMut, IntoZval},
    error::{Error, Result},
    ffi::{
        ext_php_rs_zend_object_alloc, ext_php_rs_zend_object_release, object_properties_init,
        zend_object, zend_object_std_init, zend_objects_clone_members,
    },
    flags::DataType,
    types::{ZendObject, Zval},
};

/// Representation of a Zend class object in memory.
#[repr(C)]
#[derive(Debug)]
pub struct ZendClassObject<T> {
    pub obj: Option<T>,
    pub std: ZendObject,
}

impl<T: RegisteredClass> ZendClassObject<T> {
    /// Creates a new [`ZendClassObject`] of type `T`, where `T` is a
    /// [`RegisteredClass`] in PHP, storing the given value `val` inside the
    /// object.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to store inside the object.
    ///
    /// # Panics
    ///
    /// Panics if memory was unable to be allocated for the new object.
    pub fn new(val: T) -> ZBox<Self> {
        // SAFETY: We are providing a value to initialize the object with.
        unsafe { Self::internal_new(Some(val)) }
    }

    /// Creates a new [`ZendClassObject`] of type `T`, with an uninitialized
    /// internal object.
    ///
    /// # Safety
    ///
    /// As the object is uninitialized, the caller must ensure the following
    /// until the internal object is initialized:
    ///
    /// * The object is never dereferenced to `T`.
    /// * The [`Clone`] implementation is never called.
    /// * The [`Debug`] implementation is never called.
    ///
    /// If any of these conditions are not met while not initialized, the
    /// corresponding function will panic. Converting the object into its
    /// inner pointer with the [`into_raw`] function is valid, however.
    ///
    /// [`into_raw`]: #method.into_raw
    ///
    /// # Panics
    ///
    /// Panics if memory was unable to be allocated for the new object.
    pub unsafe fn new_uninit() -> ZBox<Self> {
        Self::internal_new(None)
    }

    /// Creates a new [`ZendObject`] of type `T`, storing the given (and
    /// potentially uninitialized) `val` inside the object.
    ///
    /// # Parameters
    ///
    /// * `val` - Value to store inside the object. See safety section.
    /// * `init` - Whether the given `val` was initialized.
    ///
    /// # Safety
    ///
    /// Providing an initialized variant of [`MaybeUninit<T>`] is safe.
    ///
    /// Providing an uninitialized variant of [`MaybeUninit<T>`] is unsafe. As
    /// the object is uninitialized, the caller must ensure the following
    /// until the internal object is initialized:
    ///
    /// * The object is never dereferenced to `T`.
    /// * The [`Clone`] implementation is never called.
    /// * The [`Debug`] implementation is never called.
    ///
    /// If any of these conditions are not met while not initialized, the
    /// corresponding function will panic. Converting the object into its
    /// inner with the [`into_raw`] function is valid, however. You can
    /// initialize the object with the [`initialize`] function.
    ///
    /// [`into_raw`]: #method.into_raw
    /// [`initialize`]: #method.initialize
    ///
    /// # Panics
    ///
    /// Panics if memory was unable to be allocated for the new object.
    unsafe fn internal_new(val: Option<T>) -> ZBox<Self> {
        let size = mem::size_of::<ZendClassObject<T>>();
        let meta = T::get_metadata();
        let ce = meta.ce() as *const _ as *mut _;
        let obj = ext_php_rs_zend_object_alloc(size as _, ce) as *mut ZendClassObject<T>;
        let obj = obj
            .as_mut()
            .expect("Failed to allocate for new Zend object");

        zend_object_std_init(&mut obj.std, ce);
        object_properties_init(&mut obj.std, ce);

        // SAFETY: `obj` is non-null and well aligned as it is a reference.
        // As the data in `obj.obj` is uninitialized, we don't want to drop
        // the data, but directly override it.
        ptr::write(&mut obj.obj, val);

        obj.std.handlers = meta.handlers();
        ZBox::from_raw(obj)
    }

    /// Initializes the class object with the value `val`.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to initialize the object with.
    ///
    /// # Returns
    ///
    /// Returns the old value in an [`Option`] if the object had already been
    /// initialized, [`None`] otherwise.
    pub fn initialize(&mut self, val: T) -> Option<T> {
        self.obj.replace(val)
    }

    /// Returns a mutable reference to the [`ZendClassObject`] of a given zend
    /// object `obj`. Returns [`None`] if the given object is not of the
    /// type `T`.
    ///
    /// # Parameters
    ///
    /// * `obj` - The zend object to get the [`ZendClassObject`] for.
    pub fn from_zend_obj(std: &zend_object) -> Option<&Self> {
        Some(Self::_from_zend_obj(std)?)
    }

    /// Returns a mutable reference to the [`ZendClassObject`] of a given zend
    /// object `obj`. Returns [`None`] if the given object is not of the
    /// type `T`.
    ///
    /// # Parameters
    ///
    /// * `obj` - The zend object to get the [`ZendClassObject`] for.
    pub fn from_zend_obj_mut(std: &mut zend_object) -> Option<&mut Self> {
        Self::_from_zend_obj(std)
    }

    fn _from_zend_obj(std: &zend_object) -> Option<&mut Self> {
        let std = std as *const zend_object as *const i8;
        let ptr = unsafe {
            let ptr = std.offset(0 - Self::std_offset() as isize) as *const Self;
            (ptr as *mut Self).as_mut()?
        };

        if ptr.std.is_instance::<T>() {
            Some(ptr)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the underlying Zend object.
    pub fn get_mut_zend_obj(&mut self) -> &mut zend_object {
        &mut self.std
    }

    /// Returns the offset of the `std` property in the class object.
    pub(crate) fn std_offset() -> usize {
        unsafe {
            let null = NonNull::<Self>::dangling();
            let base = null.as_ref() as *const Self;
            let std = &null.as_ref().std as *const zend_object;

            (std as usize) - (base as usize)
        }
    }
}

impl<'a, T: RegisteredClass> FromZval<'a> for &'a ZendClassObject<T> {
    const TYPE: DataType = DataType::Object(Some(T::CLASS_NAME));

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        Self::from_zend_object(zval.object()?).ok()
    }
}

impl<'a, T: RegisteredClass> FromZendObject<'a> for &'a ZendClassObject<T> {
    fn from_zend_object(obj: &'a ZendObject) -> Result<Self> {
        // TODO(david): replace with better error
        ZendClassObject::from_zend_obj(obj).ok_or(Error::InvalidScope)
    }
}

impl<'a, T: RegisteredClass> FromZvalMut<'a> for &'a mut ZendClassObject<T> {
    const TYPE: DataType = DataType::Object(Some(T::CLASS_NAME));

    fn from_zval_mut(zval: &'a mut Zval) -> Option<Self> {
        Self::from_zend_object_mut(zval.object_mut()?).ok()
    }
}

impl<'a, T: RegisteredClass> FromZendObjectMut<'a> for &'a mut ZendClassObject<T> {
    fn from_zend_object_mut(obj: &'a mut ZendObject) -> Result<Self> {
        ZendClassObject::from_zend_obj_mut(obj).ok_or(Error::InvalidScope)
    }
}

unsafe impl<T: RegisteredClass> ZBoxable for ZendClassObject<T> {
    fn free(&mut self) {
        // SAFETY: All constructors guarantee that `self` contains a valid pointer.
        // Further, all constructors guarantee that the `std` field of
        // `ZendClassObject` will be initialized.
        unsafe { ext_php_rs_zend_object_release(&mut self.std) }
    }
}

impl<T> Deref for ZendClassObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.obj
            .as_ref()
            .expect("Attempted to access uninitialized class object")
    }
}

impl<T> DerefMut for ZendClassObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.obj
            .as_mut()
            .expect("Attempted to access uninitialized class object")
    }
}

impl<T: RegisteredClass + Default> Default for ZBox<ZendClassObject<T>> {
    #[inline]
    fn default() -> Self {
        ZendClassObject::new(T::default())
    }
}

impl<T: RegisteredClass + Clone> Clone for ZBox<ZendClassObject<T>> {
    fn clone(&self) -> Self {
        // SAFETY: All constructors of `NewClassObject` guarantee that it will contain a
        // valid pointer. The constructor also guarantees that the internal
        // `ZendClassObject` pointer will contain a valid, initialized `obj`,
        // therefore we can dereference both safely.
        unsafe {
            let mut new = ZendClassObject::new((***self).clone());
            zend_objects_clone_members(&mut new.std, &self.std as *const _ as *mut _);
            new
        }
    }
}

impl<T: RegisteredClass> IntoZval for ZBox<ZendClassObject<T>> {
    const TYPE: DataType = DataType::Object(Some(T::CLASS_NAME));

    fn set_zval(self, zv: &mut Zval, _: bool) -> Result<()> {
        let obj = self.into_raw();
        zv.set_object(&mut obj.std);
        Ok(())
    }
}

impl<T: RegisteredClass> IntoZval for &mut ZendClassObject<T> {
    const TYPE: DataType = DataType::Object(Some(T::CLASS_NAME));

    #[inline]
    fn set_zval(self, zv: &mut Zval, _: bool) -> Result<()> {
        zv.set_object(&mut self.std);
        Ok(())
    }
}
