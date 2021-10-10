//! Types and traits used for registering classes with PHP.

use std::{
    collections::HashMap,
    marker::PhantomData,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
};

use crate::{
    builders::FunctionBuilder,
    exception::PhpException,
    props::Property,
    zend::{ClassEntry, ExecuteData, ZendObjectHandlers},
};

/// Implemented on Rust types which are exported to PHP. Allows users to get and
/// set PHP properties on the object.
pub trait RegisteredClass: Sized + 'static {
    /// PHP class name of the registered class.
    const CLASS_NAME: &'static str;

    /// Optional class constructor.
    const CONSTRUCTOR: Option<ConstructorMeta<Self>> = None;

    /// Returns a reference to the class metadata, which stores the class entry
    /// and handlers.
    ///
    /// This must be statically allocated, and is usually done through the
    /// [`macro@php_class`] macro.
    ///
    /// [`macro@php_class`]: crate::php_class
    fn get_metadata() -> &'static ClassMetadata<Self>;

    /// Returns a hash table containing the properties of the class.
    ///
    /// The key should be the name of the property and the value should be a
    /// reference to the property with reference to `self`. The value is a
    /// [`Property`].
    fn get_properties<'a>() -> HashMap<&'static str, Property<'a, Self>>;
}

/// Stores metadata about a classes Rust constructor, including the function
/// pointer and the arguments of the function.
pub struct ConstructorMeta<T> {
    /// Constructor function.
    pub constructor: fn(&mut ExecuteData) -> ConstructorResult<T>,
    /// Function called to build the constructor function. Usually adds
    /// arguments.
    pub build_fn: fn(FunctionBuilder) -> FunctionBuilder,
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

/// Stores the class entry and handlers for a Rust type which has been exported
/// to PHP. Usually allocated statically.
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
    /// Returns an immutable reference to the object handlers contained inside
    /// the class metadata.
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
        // SAFETY: There are only two values that can be stored in the atomic ptr: null
        // or a static reference to a class entry. On the latter case,
        // `as_ref()` will return `None` and the function will panic.
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
    /// Panics if the class entry has already been set in the class metadata.
    /// This function should only be called once.
    pub fn set_ce(&self, ce: &'static mut ClassEntry) {
        if !self.ce.load(Ordering::SeqCst).is_null() {
            panic!("Class entry has already been set.");
        }

        self.ce.store(ce, Ordering::SeqCst);
    }

    /// Checks if the handlers have been initialized, and initializes them if
    /// they are not.
    fn check_handlers(&self) {
        if !self.handlers_init.load(Ordering::SeqCst) {
            // SAFETY: `MaybeUninit` has the same size as the handlers.
            unsafe { ZendObjectHandlers::init::<T>(self.handlers.as_ptr() as *mut _) };
            self.handlers_init.store(true, Ordering::SeqCst);
        }
    }
}
