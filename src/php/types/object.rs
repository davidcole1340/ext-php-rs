//! Represents an object in PHP. Allows for overriding the internal object used by classes,
//! allowing users to store Rust data inside a PHP object.

use std::{
    collections::HashMap,
    convert::TryInto,
    ffi::c_void,
    fmt::Debug,
    marker::PhantomData,
    mem::{self, MaybeUninit},
    ops::{Deref, DerefMut},
    os::raw::c_int,
    ptr::{self, NonNull},
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
};

use crate::{
    bindings::{
        ext_php_rs_zend_object_alloc, ext_php_rs_zend_object_release, object_properties_init,
        std_object_handlers, zend_is_true, zend_object, zend_object_handlers, zend_object_std_dtor,
        zend_object_std_init, zend_objects_clone_members, zend_std_get_properties,
        zend_std_has_property, zend_std_read_property, zend_std_write_property, zend_string,
        HashTable, ZEND_ISEMPTY, ZEND_PROPERTY_EXISTS, ZEND_PROPERTY_ISSET,
    },
    errors::{Error, Result},
    php::{
        class::ClassEntry,
        enums::DataType,
        exceptions::PhpException,
        flags::ZvalTypeFlags,
        types::{array::OwnedHashTable, string::ZendString},
    },
};

use super::{
    props::Property,
    zval::{FromZval, IntoZval, Zval},
};

pub type ZendObject = zend_object;
pub type ZendObjectHandlers = zend_object_handlers;

/// Different ways to query if a property exists.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PropertyQuery {
    /// Property exists and is not NULL.
    Isset = ZEND_PROPERTY_ISSET,
    /// Property is not empty.
    NotEmpty = ZEND_ISEMPTY,
    /// Property exists.
    Exists = ZEND_PROPERTY_EXISTS,
}

impl ZendObject {
    /// Attempts to retrieve the class name of the object.
    pub fn get_class_name(&self) -> Result<String> {
        unsafe {
            self.handlers()?
                .get_class_name
                .and_then(|f| f(self).as_ref())
                .ok_or(Error::InvalidScope)
                .and_then(|s| s.try_into())
        }
    }

    /// Checks if the given object is an instance of a registered class with Rust
    /// type `T`.
    pub fn is_instance<T: RegisteredClass>(&self) -> bool {
        (self.ce as *const ClassEntry).eq(&(T::get_metadata().ce() as *const _))
    }

    /// Attempts to read a property from the Object. Returns a result containing the
    /// value of the property if it exists and can be read, and an [`Error`] otherwise.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the property.
    pub fn get_property<'a, T: FromZval<'a>>(&self, name: &str) -> Result<T> {
        if !self.has_property(name, PropertyQuery::Exists)? {
            return Err(Error::InvalidProperty);
        }

        let mut name = ZendString::new(name, false)?;
        let mut rv = Zval::new();

        let zv = unsafe {
            self.handlers()?.read_property.ok_or(Error::InvalidScope)?(
                self.mut_ptr(),
                name.as_mut_zend_str(),
                1,
                std::ptr::null_mut(),
                &mut rv,
            )
            .as_ref()
        }
        .ok_or(Error::InvalidScope)?;

        T::from_zval(zv).ok_or_else(|| Error::ZvalConversion(zv.get_type()))
    }

    /// Attempts to set a property on the object.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the property.
    /// * `value` - The value to set the property to.
    pub fn set_property(&mut self, name: &str, value: impl IntoZval) -> Result<()> {
        let mut name = ZendString::new(name, false)?;
        let mut value = value.into_zval(false)?;

        unsafe {
            self.handlers()?.write_property.ok_or(Error::InvalidScope)?(
                self,
                name.as_mut_zend_str(),
                &mut value,
                std::ptr::null_mut(),
            )
            .as_ref()
        }
        .ok_or(Error::InvalidScope)?;
        Ok(())
    }

    /// Checks if a property exists on an object. Takes a property name and query parameter,
    /// which defines what classifies if a property exists or not. See [`PropertyQuery`] for
    /// more information.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the property.
    /// * `query` - The 'query' to classify if a property exists.
    pub fn has_property(&self, name: &str, query: PropertyQuery) -> Result<bool> {
        let mut name = ZendString::new(name, false)?;

        Ok(unsafe {
            self.handlers()?.has_property.ok_or(Error::InvalidScope)?(
                self.mut_ptr(),
                name.as_mut_zend_str(),
                query as _,
                std::ptr::null_mut(),
            )
        } > 0)
    }

    /// Attempts to retrieve the properties of the object. Returned inside a Zend Hashtable.
    pub fn get_properties(&self) -> Result<&HashTable> {
        unsafe {
            self.handlers()?
                .get_properties
                .and_then(|props| props(self.mut_ptr()).as_ref())
                .ok_or(Error::InvalidScope)
        }
    }

    /// Attempts to retrieve a reference to the object handlers.
    #[inline]
    unsafe fn handlers(&self) -> Result<&ZendObjectHandlers> {
        self.handlers.as_ref().ok_or(Error::InvalidScope)
    }

    /// Returns a mutable pointer to `self`, regardless of the type of reference.
    /// Only to be used in situations where a C function requires a mutable pointer
    /// but does not modify the underlying data.
    #[inline]
    fn mut_ptr(&self) -> *mut Self {
        (self as *const Self) as *mut Self
    }
}

impl Debug for ZendObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_struct(
            self.get_class_name()
                .unwrap_or_else(|_| "ZendObject".to_string())
                .as_str(),
        );

        if let Ok(props) = self.get_properties() {
            for (id, key, val) in props.iter() {
                dbg.field(key.unwrap_or_else(|| id.to_string()).as_str(), val);
            }
        }

        dbg.finish()
    }
}

/// Wrapper struct used to return a reference to a PHP object.
pub struct ClassRef<'a, T: RegisteredClass + 'a> {
    ptr: &'a mut ZendClassObject<T>,
}

impl<'a, T: RegisteredClass> ClassRef<'a, T> {
    /// Creates a new class reference from a Rust type reference.
    pub fn from_ref(obj: &'a T) -> Option<Self> {
        let ptr = unsafe { ZendClassObject::from_obj_ptr(obj)? };
        Some(Self { ptr })
    }
}

impl<'a, T: RegisteredClass> IntoZval for ClassRef<'a, T> {
    const TYPE: DataType = DataType::Object(Some(T::CLASS_NAME));

    fn set_zval(self, zv: &mut Zval, _: bool) -> Result<()> {
        zv.set_object(&mut self.ptr.std);
        Ok(())
    }
}

pub struct ClassObject<'a, T: RegisteredClass> {
    ptr: &'a mut ZendClassObject<T>,
    free: bool,
}

impl<T: RegisteredClass> Default for ClassObject<'_, T> {
    fn default() -> Self {
        let ptr = unsafe {
            ZendClassObject::new_ptr(None)
                .as_mut()
                .expect("Failed to allocate memory for class object.")
        };

        Self { ptr, free: true }
    }
}

impl<'a, T: RegisteredClass + 'a> ClassObject<'a, T> {
    /// Creates a class object from a pre-existing Rust object.
    ///
    /// # Parameters
    ///
    /// * `obj` - The object to create a class object for.
    pub fn new(obj: T) -> Self {
        let ptr = unsafe {
            ZendClassObject::new_ptr(Some(obj))
                .as_mut()
                .expect("Failed to allocate memory for class object.")
        };

        Self { ptr, free: true }
    }

    /// Consumes the class object, releasing the internal pointer without releasing the internal object.
    ///
    /// Used to transfer ownership of the object to PHP.
    pub(crate) fn into_raw(mut self) -> *mut ZendClassObject<T> {
        self.free = false;
        self.ptr
    }

    /// Returns an immutable reference to the underlying class object.
    pub(crate) fn internal(&self) -> &ZendClassObject<T> {
        self.ptr
    }

    /// Returns a mutable reference to the underlying class object.
    pub(crate) fn internal_mut(&mut self) -> &mut ZendClassObject<T> {
        self.ptr
    }

    /// Creates a new instance of [`ClassObject`] around a pre-existing class object.
    ///
    /// # Parameters
    ///
    /// * `ptr` - Pointer to the class object.
    /// * `free` - Whether to release the underlying object which `ptr` points to.
    ///
    /// # Safety
    ///
    /// Caller must guarantee that `ptr` points to an aligned, non-null instance of
    /// [`ZendClassObject`]. Caller must also guarantee that `ptr` will at least live for
    /// the lifetime `'a` (as long as the resulting object lives).
    ///
    /// Caller must also guarantee that it is expected to free `ptr` after dropping the
    /// resulting [`ClassObject`] to prevent use-after-free situations.
    ///
    /// # Panics
    ///
    /// Panics when the given `ptr` is null.
    pub(crate) unsafe fn from_zend_class_object(ptr: *mut ZendClassObject<T>, free: bool) -> Self {
        Self {
            ptr: ptr.as_mut().expect("Given pointer was null"),
            free,
        }
    }
}

impl<T: RegisteredClass> Drop for ClassObject<'_, T> {
    fn drop(&mut self) {
        if self.free {
            unsafe { ext_php_rs_zend_object_release(&mut (*self.ptr).std) };
        }
    }
}

impl<T: RegisteredClass> Deref for ClassObject<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: Class object constructor guarantees memory is allocated.
        unsafe { &*self.ptr.obj.as_ptr() }
    }
}

impl<T: RegisteredClass> DerefMut for ClassObject<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: Class object constructor guarantees memory is allocated.
        unsafe { &mut *self.ptr.obj.as_mut_ptr() }
    }
}

impl<T: RegisteredClass + Clone> Clone for ClassObject<'_, T> {
    fn clone(&self) -> Self {
        // SAFETY: Class object constructor guarantees memory is allocated.
        let mut new = Self::new(unsafe { &*self.internal().obj.as_ptr() }.clone());
        unsafe {
            zend_objects_clone_members(
                &mut new.internal_mut().std,
                &self.internal().std as *const _ as *mut _,
            )
        }
        new
    }
}

impl<T: RegisteredClass + Debug> Debug for ClassObject<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.internal().obj.fmt(f)
    }
}

impl<T: RegisteredClass> IntoZval for ClassObject<'_, T> {
    const TYPE: DataType = DataType::Object(Some(T::CLASS_NAME));

    fn set_zval(self, zv: &mut Zval, _: bool) -> Result<()> {
        unsafe { zv.set_object(&mut (*self.into_raw()).std) };

        Ok(())
    }
}

impl<T: RegisteredClass> IntoZval for T {
    const TYPE: DataType = DataType::Object(Some(T::CLASS_NAME));

    fn set_zval(self, zv: &mut Zval, persistent: bool) -> Result<()> {
        ClassObject::new(self).set_zval(zv, persistent)
    }
}

impl<'a, T: RegisteredClass> FromZval<'a> for &'a T {
    const TYPE: DataType = DataType::Object(Some(T::CLASS_NAME));

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        let cobj = ZendClassObject::<T>::from_zend_obj_ptr(zval.object()?)?;

        Some(unsafe { &*cobj.obj.as_mut_ptr() })
    }
}

impl<'a, T: RegisteredClass> FromZval<'a> for &'a mut T {
    const TYPE: DataType = DataType::Object(Some(T::CLASS_NAME));

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        let cobj = ZendClassObject::<T>::from_zend_obj_ptr(zval.object()?)?;

        Some(unsafe { &mut *cobj.obj.as_mut_ptr() })
    }
}

/// Implemented on Rust types which are exported to PHP. Allows users to get and set PHP properties on
/// the object.
pub trait RegisteredClass: Default + Sized
where
    Self: 'static,
{
    /// PHP class name of the registered class.
    const CLASS_NAME: &'static str;

    /// Returns a reference to the class metadata, which stores the class entry and handlers.
    ///
    /// This must be statically allocated, and is usually done through the [`macro@php_class`]
    /// macro.
    ///
    /// [`macro@php_class`]: crate::php_class
    fn get_metadata() -> &'static ClassMetadata<Self>;

    /// Attempts to retrieve a property from the class object.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the property.
    ///
    /// # Returns
    ///
    /// Returns a given type `T` inside an option which is the value of the zval, or [`None`]
    /// if the property could not be found.
    ///
    /// # Safety
    ///
    /// Caller must guarantee that the object the function is called on is immediately followed
    /// by a [`zend_object`], which is true when the object was instantiated by PHP.
    unsafe fn get_property<'a, T: FromZval<'a>>(&'a self, name: &str) -> Option<T> {
        let obj = ZendClassObject::<Self>::from_obj_ptr(self)?;
        obj.std.get_property(name).ok()
    }

    /// Attempts to set the value of a property on the class object.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the property to set.
    /// * `value` - The value to set the property to.
    ///
    /// # Returns
    ///
    /// Returns nothing in an option if the property was successfully set. Returns none if setting
    /// the value failed.
    ///
    /// # Safety
    ///
    /// Caller must guarantee that the object the function is called on is immediately followed
    /// by a [`zend_object`], which is true when the object was instantiated by PHP.
    unsafe fn set_property(&mut self, name: &str, value: impl IntoZval) -> Option<()> {
        let obj = ZendClassObject::<Self>::from_obj_ptr(self)?;
        obj.std.set_property(name, value).ok()?;
        Some(())
    }

    /// Returns a hash table containing the properties of the class.
    ///
    /// The key should be the name of the property and the value should be a reference to the property
    /// with reference to `self`. The value is a trait object for [`Prop`].
    fn get_properties<'a>() -> HashMap<&'static str, Property<'a, Self>>;
}

/// Representation of a Zend class object in memory. Usually seen through its managed variant
/// of [`ClassObject`].
#[repr(C)]
pub(crate) struct ZendClassObject<T> {
    obj: MaybeUninit<T>,
    std: zend_object,
}

impl<T: RegisteredClass> ZendClassObject<T> {
    /// Allocates memory for a new PHP object. The memory is allocated using the Zend memory manager,
    /// and therefore it is returned as a pointer.
    pub(crate) fn new_ptr(val: Option<T>) -> *mut Self {
        let size = mem::size_of::<Self>();
        let meta = T::get_metadata();
        let ce = meta.ce() as *const _ as *mut _;
        unsafe {
            let obj = (ext_php_rs_zend_object_alloc(size as _, ce) as *mut Self)
                .as_mut()
                .expect("Failed to allocate memory for new class object.");

            zend_object_std_init(&mut obj.std, ce);
            object_properties_init(&mut obj.std, ce);

            obj.obj = MaybeUninit::new(val.unwrap_or_default());
            obj.std.handlers = meta.handlers();
            obj
        }
    }

    /// Returns a reference to the [`ZendClassObject`] of a given object `T`. Returns [`None`]
    /// if the given object is not of the type `T`.
    ///
    /// # Parameters
    ///
    /// * `obj` - The object to get the [`ZendClassObject`] for.
    ///
    /// # Safety
    ///
    /// Caller must guarantee that the given `obj` was created by Zend, which means that it
    /// is immediately followed by a [`zend_object`].
    pub(crate) unsafe fn from_obj_ptr(obj: &T) -> Option<&mut Self> {
        let ptr = (obj as *const T as *mut Self).as_mut()?;

        if ptr.std.is_instance::<T>() {
            Some(ptr)
        } else {
            None
        }
    }

    /// Returns a reference to the [`ZendClassObject`] of a given zend object `obj`. Returns [`None`]
    /// if the given object is not of the type `T`.
    ///
    /// # Parameters
    ///
    /// * `obj` - The zend object to get the [`ZendClassObject`] for.
    pub(crate) fn from_zend_obj_ptr<'a>(obj: *const zend_object) -> Option<&'a mut Self> {
        let ptr = obj as *const zend_object as *const i8;
        let ptr = unsafe {
            let ptr = ptr.offset(0 - Self::std_offset() as isize) as *const Self;
            (ptr as *mut Self).as_mut()?
        };

        if ptr.std.is_instance::<T>() {
            Some(ptr)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the underlying Zend object.
    pub(crate) fn get_mut_zend_obj(&mut self) -> &mut zend_object {
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

impl<T> Drop for ZendClassObject<T> {
    fn drop(&mut self) {
        // SAFETY: All constructors guarantee that `obj` is valid.
        unsafe { std::ptr::drop_in_place(self.obj.as_mut_ptr()) };
    }
}

/// Stores the class entry and handlers for a Rust type which has been exported to PHP.
pub struct ClassMetadata<T> {
    handlers_init: AtomicBool,
    handlers: MaybeUninit<ZendObjectHandlers>,
    ce: AtomicPtr<ClassEntry>,

    phantom: PhantomData<T>,
}

impl<T> ClassMetadata<T> {
    /// Creates a new class metadata instance.
    pub const fn new() -> Self {
        Self {
            handlers_init: AtomicBool::new(false),
            handlers: MaybeUninit::uninit(),
            ce: AtomicPtr::new(std::ptr::null_mut()),
            phantom: PhantomData,
        }
    }
}

impl<T: RegisteredClass> ClassMetadata<T> {
    /// Returns an immutable reference to the object handlers contained inside the class metadata.
    pub fn handlers(&self) -> &ZendObjectHandlers {
        self.check_handlers();

        // SAFETY: `check_handlers` guarantees that `handlers` has been initialized.
        unsafe { &*self.handlers.as_ptr() }
    }

    /// Checks if the class entry has been stored, returning a boolean.
    pub fn has_ce(&self) -> bool {
        !self.ce.load(Ordering::SeqCst).is_null()
    }

    /// Retrieves a reference to the stored class entry.
    ///
    /// # Panics
    ///
    /// Panics if there is no class entry stored inside the class metadata.
    pub fn ce(&self) -> &'static ClassEntry {
        // SAFETY: There are only two values that can be stored in the atomic ptr: null or a static reference
        // to a class entry. On the latter case, `as_ref()` will return `None` and the function will panic.
        unsafe { self.ce.load(Ordering::SeqCst).as_ref() }
            .expect("Attempted to retrieve class entry before it has been stored.")
    }

    /// Stores a reference to a class entry inside the class metadata.
    ///
    /// # Parameters
    ///
    /// * `ce` - The class entry to store.
    ///
    /// # Panics
    ///
    /// Panics if the class entry has already been set in the class metadata. This function should
    /// only be called once.
    pub fn set_ce(&self, ce: &'static mut ClassEntry) {
        if !self.ce.load(Ordering::SeqCst).is_null() {
            panic!("Class entry has already been set.");
        }

        self.ce.store(ce, Ordering::SeqCst);
    }

    /// Checks if the handlers have been initialized, and initializes them if they are not.
    fn check_handlers(&self) {
        if !self.handlers_init.load(Ordering::Acquire) {
            // SAFETY: `MaybeUninit` has the same size as the handlers.
            unsafe { ZendObjectHandlers::init::<T>(self.handlers.as_ptr() as *mut _) };
            self.handlers_init.store(true, Ordering::Release);
        }
    }
}

impl ZendObjectHandlers {
    /// Initializes a given set of object handlers by copying the standard object handlers into
    /// the memory location, as well as setting up the `T` type destructor.
    ///
    /// # Parameters
    ///
    //// * `ptr` - Pointer to memory location to copy the standard handlers to.
    ///
    /// # Safety
    ///
    /// Caller must guarantee that the `ptr` given is a valid memory location.
    pub unsafe fn init<T: RegisteredClass>(ptr: *mut ZendObjectHandlers) {
        std::ptr::copy_nonoverlapping(&std_object_handlers, ptr, 1);
        let offset = ZendClassObject::<T>::std_offset();
        (*ptr).offset = offset as _;
        (*ptr).free_obj = Some(Self::free_obj::<T>);
        (*ptr).read_property = Some(Self::read_property::<T>);
        (*ptr).write_property = Some(Self::write_property::<T>);
        (*ptr).get_properties = Some(Self::get_properties::<T>);
        (*ptr).has_property = Some(Self::has_property::<T>);
    }

    unsafe extern "C" fn free_obj<T: RegisteredClass>(object: *mut zend_object) {
        let obj = ZendClassObject::<T>::from_zend_obj_ptr(object)
            .expect("Invalid object pointer given for `free_obj`");

        // Manually drop the object as it is wrapped with `MaybeUninit`.
        ptr::drop_in_place(obj.obj.as_mut_ptr());

        zend_object_std_dtor(object)
    }

    unsafe extern "C" fn read_property<T: RegisteredClass>(
        object: *mut zend_object,
        member: *mut zend_string,
        type_: c_int,
        cache_slot: *mut *mut c_void,
        rv: *mut Zval,
    ) -> *mut Zval {
        #[inline(always)]
        unsafe fn internal<T: RegisteredClass>(
            object: *mut zend_object,
            member: *mut zend_string,
            type_: c_int,
            cache_slot: *mut *mut c_void,
            rv: *mut Zval,
        ) -> std::result::Result<*mut Zval, PhpException<'static>> {
            let obj = object
                .as_ref()
                .and_then(|obj| ZendClassObject::<T>::from_zend_obj_ptr(obj))
                .ok_or("Invalid object pointer given")?;
            let prop_name = member
                .as_ref()
                .ok_or("Invalid property name pointer given")?;
            let self_ = obj.obj.assume_init_mut();
            let mut props = T::get_properties();
            let prop = props.remove(prop_name.as_str().ok_or("Invalid property name given")?);

            // retval needs to be treated as initialized, so we set the type to null
            let rv_mut = rv.as_mut().ok_or("Invalid return zval given")?;
            rv_mut.u1.type_info = ZvalTypeFlags::Null.bits();

            Ok(match prop {
                Some(prop) => {
                    prop.get(self_, rv_mut)?;
                    rv
                }
                None => zend_std_read_property(object, member, type_, cache_slot, rv),
            })
        }

        match internal::<T>(object, member, type_, cache_slot, rv) {
            Ok(rv) => rv,
            Err(e) => {
                let _ = e.throw();
                (&mut *rv).set_null();
                rv
            }
        }
    }

    unsafe extern "C" fn write_property<T: RegisteredClass>(
        object: *mut zend_object,
        member: *mut zend_string,
        value: *mut Zval,
        cache_slot: *mut *mut c_void,
    ) -> *mut Zval {
        #[inline(always)]
        unsafe fn internal<T: RegisteredClass>(
            object: *mut zend_object,
            member: *mut zend_string,
            value: *mut Zval,
            cache_slot: *mut *mut c_void,
        ) -> std::result::Result<*mut Zval, PhpException<'static>> {
            let obj = object
                .as_ref()
                .and_then(|obj| ZendClassObject::<T>::from_zend_obj_ptr(obj))
                .ok_or("Invalid object pointer given")?;
            let prop_name = member
                .as_ref()
                .ok_or("Invalid property name pointer given")?;
            let self_ = obj.obj.assume_init_mut();
            let mut props = T::get_properties();
            let prop = props.remove(prop_name.as_str().ok_or("Invalid property name given")?);
            let value_mut = value.as_mut().ok_or("Invalid return zval given")?;

            Ok(match prop {
                Some(prop) => {
                    prop.set(self_, value_mut)?;
                    value
                }
                None => zend_std_write_property(object, member, value, cache_slot),
            })
        }

        match internal::<T>(object, member, value, cache_slot) {
            Ok(rv) => rv,
            Err(e) => {
                let _ = e.throw();
                value
            }
        }
    }

    unsafe extern "C" fn get_properties<T: RegisteredClass>(
        object: *mut zend_object,
    ) -> *mut HashTable {
        #[inline(always)]
        unsafe fn internal<T: RegisteredClass>(
            object: *mut zend_object,
            props: &mut HashTable,
        ) -> std::result::Result<(), PhpException<'static>> {
            let obj = object
                .as_ref()
                .and_then(|obj| ZendClassObject::<T>::from_zend_obj_ptr(obj))
                .ok_or("Invalid object pointer given")?;
            let self_ = obj.obj.assume_init_mut();
            let struct_props = T::get_properties();

            for (name, val) in struct_props.into_iter() {
                let mut zv = Zval::new();
                if val.get(self_, &mut zv).is_err() {
                    continue;
                }
                props.insert(name, zv).map_err(|e| {
                    format!("Failed to insert value into properties hashtable: {:?}", e)
                })?;
            }

            Ok(())
        }

        let props = zend_std_get_properties(object)
            .as_mut()
            .or_else(|| OwnedHashTable::new().into_inner().as_mut())
            .expect("Failed to get property hashtable");

        if let Err(e) = internal::<T>(object, props) {
            let _ = e.throw();
        }

        props
    }

    unsafe extern "C" fn has_property<T: RegisteredClass>(
        object: *mut zend_object,
        member: *mut zend_string,
        has_set_exists: c_int,
        cache_slot: *mut *mut c_void,
    ) -> c_int {
        #[inline(always)]
        unsafe fn internal<T: RegisteredClass>(
            object: *mut zend_object,
            member: *mut zend_string,
            has_set_exists: c_int,
            cache_slot: *mut *mut c_void,
        ) -> std::result::Result<c_int, PhpException<'static>> {
            let obj = object
                .as_ref()
                .and_then(|obj| ZendClassObject::<T>::from_zend_obj_ptr(obj))
                .ok_or("Invalid object pointer given")?;
            let prop_name = member
                .as_ref()
                .ok_or("Invalid property name pointer given")?;
            let props = T::get_properties();
            let prop = props.get(prop_name.as_str().ok_or("Invalid property name given")?);
            let self_ = obj.obj.assume_init_mut();

            match has_set_exists {
                // * 0 (has) whether property exists and is not NULL
                0 => {
                    if let Some(val) = prop {
                        let mut zv = Zval::new();
                        val.get(self_, &mut zv)?;
                        if !zv.is_null() {
                            return Ok(1);
                        }
                    }
                }
                // * 1 (set) whether property exists and is true
                1 => {
                    if let Some(val) = prop {
                        let mut zv = Zval::new();
                        val.get(self_, &mut zv)?;

                        if zend_is_true(&mut zv) == 1 {
                            return Ok(1);
                        }
                    }
                }
                // * 2 (exists) whether property exists
                2 => {
                    if prop.is_some() {
                        return Ok(1);
                    }
                }
                _ => return Err(
                    "Invalid value given for `has_set_exists` in struct `has_property` function."
                        .into(),
                ),
            };

            Ok(zend_std_has_property(
                object,
                member,
                has_set_exists,
                cache_slot,
            ))
        }

        match internal::<T>(object, member, has_set_exists, cache_slot) {
            Ok(rv) => rv,
            Err(e) => {
                let _ = e.throw();
                0
            }
        }
    }
}
