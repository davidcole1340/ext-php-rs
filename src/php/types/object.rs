//! Represents an object in PHP. Allows for overriding the internal object used by classes,
//! allowing users to store Rust data inside a PHP object.

use std::{
    collections::HashMap,
    convert::TryInto,
    ffi::c_void,
    fmt::Debug,
    marker::PhantomData,
    mem::{self, ManuallyDrop, MaybeUninit},
    ops::{Deref, DerefMut},
    os::raw::c_int,
    ptr::{self, NonNull},
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
};

use crate::{
    bindings::{
        ext_php_rs_zend_object_alloc, ext_php_rs_zend_object_release, object_properties_init,
        std_object_handlers, zend_call_known_function, zend_is_true, zend_object,
        zend_object_handlers, zend_object_std_dtor, zend_object_std_init,
        zend_objects_clone_members, zend_objects_new, zend_standard_class_def,
        zend_std_get_properties, zend_std_has_property, zend_std_read_property,
        zend_std_write_property, zend_string, HashTable, ZEND_ISEMPTY, ZEND_PROPERTY_EXISTS,
        ZEND_PROPERTY_ISSET,
    },
    errors::{Error, Result},
    php::{
        boxed::{ZBox, ZBoxable},
        class::ClassEntry,
        enums::DataType,
        exceptions::{PhpException, PhpResult},
        execution_data::ExecutionData,
        flags::ZvalTypeFlags,
        function::FunctionBuilder,
        globals::ExecutorGlobals,
        types::array::OwnedHashTable,
    },
};

use super::{
    props::Property,
    rc::PhpRc,
    string::ZendStr,
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
    /// Creates a new [`ZendObject`], returned inside an [`ZBox<ZendObject>`] wrapper.
    ///
    /// # Parameters
    ///
    /// * `ce` - The type of class the new object should be an instance of.
    ///
    /// # Panics
    ///
    /// Panics when allocating memory for the new object fails.
    pub fn new(ce: &ClassEntry) -> ZBox<Self> {
        // SAFETY: Using emalloc to allocate memory inside Zend arena. Casting `ce` to `*mut` is valid
        // as the function will not mutate `ce`.
        unsafe {
            let ptr = zend_objects_new(ce as *const _ as *mut _);
            ZBox::from_raw(
                ptr.as_mut()
                    .expect("Failed to allocate memory for Zend object"),
            )
        }
    }

    /// Creates a new `stdClass` instance, returned inside an [`ZBox<ZendObject>`] wrapper.
    ///
    /// # Panics
    ///
    /// Panics if allocating memory for the object fails, or if the `stdClass` class entry has not been
    /// registered with PHP yet.
    pub fn new_stdclass() -> ZBox<Self> {
        // SAFETY: This will be `NULL` until it is initialized. `as_ref()` checks for null,
        // so we can panic if it's null.
        Self::new(unsafe {
            zend_standard_class_def
                .as_ref()
                .expect("`stdClass` class instance not initialized yet")
        })
    }

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
    /// * `query` - The type of query to use when attempting to get a property.
    pub fn get_property<'a, T>(&'a self, name: &str) -> Result<T>
    where
        T: FromZval<'a>,
    {
        if !self.has_property(name, PropertyQuery::Exists)? {
            return Err(Error::InvalidProperty);
        }

        let mut name = ZendStr::new(name, false)?;
        let mut rv = Zval::new();

        let zv = unsafe {
            self.handlers()?.read_property.ok_or(Error::InvalidScope)?(
                self.mut_ptr(),
                name.deref_mut(),
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
        let mut name = ZendStr::new(name, false)?;
        let mut value = value.into_zval(false)?;

        unsafe {
            self.handlers()?.write_property.ok_or(Error::InvalidScope)?(
                self,
                name.deref_mut(),
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
        let mut name = ZendStr::new(name, false)?;

        Ok(unsafe {
            self.handlers()?.has_property.ok_or(Error::InvalidScope)?(
                self.mut_ptr(),
                name.deref_mut(),
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

    /// Extracts some type from a Zend object.
    ///
    /// This is a wrapper function around `FromZendObject::extract()`.
    pub fn extract<'a, T>(&'a self) -> Result<T>
    where
        T: FromZendObject<'a>,
    {
        T::from_zend_object(self)
    }
}

unsafe impl ZBoxable for ZendObject {
    fn free(&mut self) {
        unsafe { ext_php_rs_zend_object_release(self) }
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

impl<'a> FromZval<'a> for &'a ZendObject {
    const TYPE: DataType = DataType::Object(None);

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        zval.object()
    }
}

impl IntoZval for ZBox<ZendObject> {
    const TYPE: DataType = DataType::Object(None);

    fn set_zval(mut self, zv: &mut Zval, _: bool) -> Result<()> {
        // We must decrement the refcounter on the object before inserting into the zval,
        // as the reference counter will be incremented on add.
        // NOTE(david): again is this needed, we increment in `set_object`.
        self.dec_count();
        zv.set_object(self.into_raw());
        Ok(())
    }
}

/// `FromZendObject` is implemented by types which can be extracted from a Zend object.
///
/// Normal usage is through the helper method `ZendObject::extract`:
///
/// ```rust,ignore
/// let obj: ZendObject = ...;
/// let repr: String = obj.extract();
/// let props: HashMap = obj.extract();
/// ```
///
/// Should be functionally equivalent to casting an object to another compatable type.
pub trait FromZendObject<'a>: Sized {
    /// Extracts `Self` from the source `ZendObject`.
    fn from_zend_object(obj: &'a ZendObject) -> Result<Self>;
}

/// Implemented on types which can be converted into a Zend object. It is up to the implementation
/// to determine the type of object which is produced.
pub trait IntoZendObject {
    /// Attempts to convert `self` into a Zend object.
    fn into_zend_object(self) -> Result<ZBox<ZendObject>>;
}

impl FromZendObject<'_> for String {
    fn from_zend_object(obj: &ZendObject) -> Result<Self> {
        let mut ret = Zval::new();
        unsafe {
            zend_call_known_function(
                (*obj.ce).__tostring,
                obj as *const _ as *mut _,
                obj.ce,
                &mut ret,
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
        }

        if let Some(err) = ExecutorGlobals::take_exception() {
            // TODO: become an error
            let class_name = obj.get_class_name();
            panic!(
                "Uncaught exception during call to {}::__toString(): {:?}",
                class_name.expect("unable to determine class name"),
                err
            );
        } else if let Some(output) = ret.extract() {
            Ok(output)
        } else {
            // TODO: become an error
            let class_name = obj.get_class_name();
            panic!(
                "{}::__toString() must return a string",
                class_name.expect("unable to determine class name"),
            );
        }
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

impl<T: RegisteredClass> From<ZBox<ZendClassObject<T>>> for ZBox<ZendObject> {
    fn from(obj: ZBox<ZendClassObject<T>>) -> Self {
        let mut this = ManuallyDrop::new(obj);
        unsafe { ZBox::from_raw(&mut this.std) }
    }
}

impl<T: RegisteredClass + Default> Default for ZBox<ZendClassObject<T>> {
    fn default() -> Self {
        ZendClassObject::new(T::default())
    }
}

impl<T: RegisteredClass + Clone> Clone for ZBox<ZendClassObject<T>> {
    fn clone(&self) -> Self {
        // SAFETY: All constructors of `NewClassObject` guarantee that it will contain a valid pointer.
        // The constructor also guarantees that the internal `ZendClassObject` pointer will contain a valid,
        // initialized `obj`, therefore we can dereference both safely.
        unsafe {
            let mut new = ZendClassObject::new((&***self).clone());
            zend_objects_clone_members(&mut new.std, &self.std as *const _ as *mut _);
            new
        }
    }
}

impl<T: RegisteredClass + Debug> Debug for ZendClassObject<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (&**self).fmt(f)
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

/// Object constructor metadata.
pub struct ConstructorMeta<T> {
    /// Constructor function.
    pub constructor: fn(&mut ExecutionData) -> ConstructorResult<T>,
    /// Function called to build the constructor function. Usually adds arguments.
    pub build_fn: fn(FunctionBuilder) -> FunctionBuilder,
}

/// Implemented on Rust types which are exported to PHP. Allows users to get and set PHP properties on
/// the object.
pub trait RegisteredClass: Sized
where
    Self: 'static,
{
    /// PHP class name of the registered class.
    const CLASS_NAME: &'static str;

    /// Optional class constructor.
    const CONSTRUCTOR: Option<ConstructorMeta<Self>> = None;

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
    /// with reference to `self`. The value is a [`Property`].
    fn get_properties<'a>() -> HashMap<&'static str, Property<'a, Self>>;
}

/// Representation of a Zend class object in memory. Usually seen through its owned variant
/// of [`ClassObject`].
#[repr(C)]
pub struct ZendClassObject<T> {
    obj: MaybeUninit<T>,
    init: bool,
    std: zend_object,
}

impl<T: RegisteredClass> ZendClassObject<T> {
    /// Creates a new [`ZendClassObject`] of type `T`, where `T` is a [`RegisteredClass`] in PHP, storing the
    /// given value `val` inside the object.
    ///
    /// # Parameters
    ///
    /// * `val` - The value to store inside the object.
    ///
    /// # Panics
    ///
    /// Panics if memory was unable to be allocated for the new object.
    pub fn new(val: T) -> ZBox<Self> {
        unsafe { Self::internal_new(MaybeUninit::new(val), true) }
    }

    /// Creates a new [`ZendClassObject`] of type `T`, with an uninitialized internal object.
    ///
    /// # Safety
    ///
    /// As the object is uninitialized, the caller must ensure the following until the internal object is
    /// initialized:
    ///
    /// * The [`Drop`] implementation is never called.
    /// * The [`Clone`] implementation is never called.
    /// * The [`Debug`] implementation is never called.
    /// * The object is never dereferenced to `T`.
    ///
    /// If any of these conditions are not met while not initialized, the corresponding function will panic.
    /// Converting the object into its inner pointer with the [`into_raw`] function is valid, however.
    ///
    /// [`into_raw`]: #method.into_raw
    ///
    /// # Panics
    ///
    /// Panics if memory was unable to be allocated for the new object.
    pub unsafe fn new_uninit() -> ZBox<Self> {
        Self::internal_new(MaybeUninit::uninit(), false)
    }

    /// Creates a new [`ZendObject`] of type `T`, storing the given (and potentially uninitialized) `val`
    /// inside the object.
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
    /// Providing an uninitalized variant of [`MaybeUninit<T>`] is unsafe. As the object is uninitialized,
    /// the caller must ensure the following until the internal object is initialized:
    ///
    /// * The [`Drop`] implementation is never called.
    /// * The [`Clone`] implementation is never called.
    /// * The [`Debug`] implementation is never called.
    /// * The object is never dereferenced to `T`.
    ///
    /// If any of these conditions are not met while not initialized, the corresponding function will panic.
    /// Converting the object into its inner with the [`into_raw`] function is valid, however. You can initialize
    /// the object with the [`initialize`] function.
    ///
    /// [`into_raw`]: #method.into_raw
    /// [`initialize`]: #method.initialize
    ///
    /// # Panics
    ///
    /// Panics if memory was unable to be allocated for the new object.
    unsafe fn internal_new(val: MaybeUninit<T>, init: bool) -> ZBox<Self> {
        let size = mem::size_of::<ZendClassObject<T>>();
        let meta = T::get_metadata();
        let ce = meta.ce() as *const _ as *mut _;
        let obj = ext_php_rs_zend_object_alloc(size as _, ce) as *mut ZendClassObject<T>;
        let obj = obj
            .as_mut()
            .expect("Failed to allocate for new Zend object");

        zend_object_std_init(&mut obj.std, ce);
        object_properties_init(&mut obj.std, ce);

        obj.obj = val;
        obj.init = init;
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
    /// Returns the old value in an [`Option`] if the object had already been initialized, [`None`]
    /// otherwise.
    pub fn initialize(&mut self, val: T) -> Option<T> {
        let old = Some(mem::replace(&mut self.obj, MaybeUninit::new(val))).and_then(|v| {
            if self.init {
                Some(unsafe { v.assume_init() })
            } else {
                None
            }
        });
        self.init = true;

        old
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
        // TODO(david): Remove this function
        let ptr = (obj as *const T as *mut Self).as_mut()?;

        if ptr.std.is_instance::<T>() {
            Some(ptr)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the [`ZendClassObject`] of a given zend object `obj`.
    /// Returns [`None`] if the given object is not of the type `T`.
    ///
    /// # Parameters
    ///
    /// * `obj` - The zend object to get the [`ZendClassObject`] for.
    pub fn from_zend_obj(std: &zend_object) -> Option<&Self> {
        Some(Self::_from_zend_obj(std)?)
    }

    /// Returns a mutable reference to the [`ZendClassObject`] of a given zend object `obj`.
    /// Returns [`None`] if the given object is not of the type `T`.
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
        ZendClassObject::from_zend_obj(obj).ok_or(Error::InvalidPointer)
    }
}

unsafe impl<T: RegisteredClass> ZBoxable for ZendClassObject<T> {
    fn free(&mut self) {
        // SAFETY: All constructors guarantee that `self` contains a valid pointer. Further, all constructors
        // guarantee that the `std` field of `ZendClassObject` will be initialized.
        unsafe { ext_php_rs_zend_object_release(&mut self.std) }
    }
}

impl<T> Drop for ZendClassObject<T> {
    fn drop(&mut self) {
        // SAFETY: All constructors guarantee that `obj` is valid.
        unsafe { std::ptr::drop_in_place(self.obj.as_mut_ptr()) };
    }
}

impl<T> Deref for ZendClassObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        if !self.init {
            panic!("Attempted to access uninitialized class object");
        }

        // SAFETY: All constructors guarantee that `obj` is valid.
        unsafe { self.obj.assume_init_ref() }
    }
}

impl<T> DerefMut for ZendClassObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if !self.init {
            panic!("Attempted to access uninitialized class object");
        }

        // SAFETY: All constructors guarantee that `obj` is valid.
        unsafe { self.obj.assume_init_mut() }
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

/// Result returned from a constructor of a class.
pub enum ConstructorResult<T> {
    /// Successfully constructed the class, contains the new class object.
    Ok(T),
    /// An exception occured while constructing the class.
    Exception(PhpException),
    /// Invalid arguments were given to the constructor.
    ArgError,
}

impl<T, E> From<std::result::Result<T, E>> for ConstructorResult<T>
where
    E: Into<PhpException>,
{
    fn from(result: std::result::Result<T, E>) -> Self {
        match result {
            Ok(x) => Self::Ok(x),
            Err(e) => Self::Exception(e.into()),
        }
    }
}

impl<T> From<T> for ConstructorResult<T> {
    fn from(result: T) -> Self {
        Self::Ok(result)
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
        let obj = object
            .as_mut()
            .and_then(|obj| ZendClassObject::<T>::from_zend_obj_mut(obj))
            .expect("Invalid object pointer given for `free_obj`");

        // Manually drop the object as it is wrapped with `MaybeUninit`.
        if obj.init {
            ptr::drop_in_place(obj.obj.as_mut_ptr());
        }

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
        ) -> PhpResult<*mut Zval> {
            let obj = object
                .as_mut()
                .and_then(|obj| ZendClassObject::<T>::from_zend_obj_mut(obj))
                .ok_or("Invalid object pointer given")?;
            let prop_name = member
                .as_ref()
                .ok_or("Invalid property name pointer given")?;
            let self_ = &mut **obj;
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
        ) -> PhpResult<*mut Zval> {
            let obj = object
                .as_mut()
                .and_then(|obj| ZendClassObject::<T>::from_zend_obj_mut(obj))
                .ok_or("Invalid object pointer given")?;
            let prop_name = member
                .as_ref()
                .ok_or("Invalid property name pointer given")?;
            let self_ = &mut **obj;
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
        ) -> PhpResult {
            let obj = object
                .as_mut()
                .and_then(|obj| ZendClassObject::<T>::from_zend_obj_mut(obj))
                .ok_or("Invalid object pointer given")?;
            let self_ = &mut **obj;
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
        ) -> PhpResult<c_int> {
            let obj = object
                .as_mut()
                .and_then(|obj| ZendClassObject::<T>::from_zend_obj_mut(obj))
                .ok_or("Invalid object pointer given")?;
            let prop_name = member
                .as_ref()
                .ok_or("Invalid property name pointer given")?;
            let props = T::get_properties();
            let prop = props.get(prop_name.as_str().ok_or("Invalid property name given")?);
            let self_ = &mut **obj;

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

impl<'a, T: RegisteredClass> FromZendObject<'a> for &'a T {
    fn from_zend_object(obj: &'a ZendObject) -> Result<Self> {
        // TODO(david): Error is kinda wrong, should have something like `WrongObject`
        let cobj = ZendClassObject::<T>::from_zend_obj(obj).ok_or(Error::InvalidPointer)?;
        Ok(&**cobj)
    }
}

impl<'a, T: RegisteredClass> FromZval<'a> for &'a T {
    const TYPE: DataType = DataType::Object(Some(T::CLASS_NAME));

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        Self::from_zend_object(zval.object()?).ok()
    }
}

// TODO(david): Need something like `FromZendObjectMut` and `FromZvalMut`
// impl<'a, T: RegisteredClass> FromZendObject<'a> for &'a mut T {
//     fn from_zend_object(obj: &'a ZendObject) -> Result<Self> {
//         // TODO(david): Error is kinda wrong, should have something like `WrongObject`
//         let cobj = ZendClassObject::<T>::from_zend_obj_mut(obj).ok_or(Error::InvalidPointer)?;
//         Ok(unsafe { cobj.obj.assume_init_mut() })
//     }
// }
//
// impl<'a, T: RegisteredClass> FromZval<'a> for &'a mut T {
//     const TYPE: DataType = DataType::Object(Some(T::CLASS_NAME));

//     fn from_zval(zval: &'a Zval) -> Option<Self> {
//         Self::from_zend_object(zval.object()?).ok()
//     }
// }

impl<T: RegisteredClass> IntoZendObject for T {
    fn into_zend_object(self) -> Result<ZBox<ZendObject>> {
        Ok(ZendClassObject::new(self).into())
    }
}

impl<T: RegisteredClass> IntoZval for T {
    const TYPE: DataType = DataType::Object(Some(T::CLASS_NAME));

    fn set_zval(self, zv: &mut Zval, persistent: bool) -> Result<()> {
        self.into_zend_object()?.set_zval(zv, persistent)
    }
}
