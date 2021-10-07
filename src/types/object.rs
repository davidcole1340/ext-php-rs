//! Represents an object in PHP. Allows for overriding the internal object used by classes,
//! allowing users to store Rust data inside a PHP object.

use std::{convert::TryInto, fmt::Debug, ops::DerefMut};

use crate::{
    boxed::{ZBox, ZBoxable},
    class::RegisteredClass,
    convert::{FromZendObject, FromZval, IntoZval},
    error::{Error, Result},
    ffi::{
        ext_php_rs_zend_object_release, zend_call_known_function, zend_object, zend_objects_new,
        zend_standard_class_def, HashTable, ZEND_ISEMPTY, ZEND_PROPERTY_EXISTS,
        ZEND_PROPERTY_ISSET,
    },
    flags::DataType,
    rc::PhpRc,
    types::{ZendClassObject, ZendStr, Zval},
    zend::{ClassEntry, ExecutorGlobals, ZendObjectHandlers},
};

pub type ZendObject = zend_object;

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

    /// Converts the class object into an owned [`ZendObject`]. This removes any possibility of
    /// accessing the underlying attached Rust struct.
    pub fn from_class_object<T: RegisteredClass>(obj: ZBox<ZendClassObject<T>>) -> ZBox<Self> {
        let this = obj.into_raw();
        // SAFETY: Consumed box must produce a well-aligned non-null pointer.
        unsafe { ZBox::from_raw(this.get_mut_zend_obj()) }
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

impl<T: RegisteredClass> From<ZBox<ZendClassObject<T>>> for ZBox<ZendObject> {
    #[inline]
    fn from(obj: ZBox<ZendClassObject<T>>) -> Self {
        ZendObject::from_class_object(obj)
    }
}

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
