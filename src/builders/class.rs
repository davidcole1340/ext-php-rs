use std::{ffi::CString, mem::MaybeUninit, rc::Rc};

use crate::{
    builders::FunctionBuilder,
    class::{ConstructorMeta, ConstructorResult, RegisteredClass},
    convert::{IntoZval, IntoZvalDyn},
    describe::DocComments,
    error::{Error, Result},
    exception::PhpException,
    ffi::{
        zend_declare_class_constant, zend_declare_property, zend_do_implement_interface,
        zend_register_internal_class_ex,
    },
    flags::{ClassFlags, MethodFlags, PropertyFlags},
    types::{ZendClassObject, ZendObject, ZendStr, Zval},
    zend::{ClassEntry, ExecuteData, FunctionEntry},
    zend_fastcall,
};

type ConstantEntry = (String, Box<dyn FnOnce() -> Result<Zval>>, DocComments);

/// Builder for registering a class in PHP.
pub struct ClassBuilder {
    pub(crate) name: String,
    ce: ClassEntry,
    extends: Option<&'static ClassEntry>,
    interfaces: Vec<&'static ClassEntry>,
    pub(crate) methods: Vec<(FunctionBuilder<'static>, MethodFlags)>,
    object_override: Option<unsafe extern "C" fn(class_type: *mut ClassEntry) -> *mut ZendObject>,
    pub(crate) properties: Vec<(String, PropertyFlags, DocComments)>,
    pub(crate) constants: Vec<ConstantEntry>,
    register: Option<fn(&'static mut ClassEntry)>,
    pub(crate) docs: DocComments,
}

impl ClassBuilder {
    /// Creates a new class builder, used to build classes
    /// to be exported to PHP.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the class.
    pub fn new<T: Into<String>>(name: T) -> Self {
        Self {
            name: name.into(),
            // SAFETY: A zeroed class entry is in an initialized state, as it is a raw C type
            // whose fields do not have a drop implementation.
            ce: unsafe { MaybeUninit::zeroed().assume_init() },
            extends: None,
            interfaces: vec![],
            methods: vec![],
            object_override: None,
            properties: vec![],
            constants: vec![],
            register: None,
            docs: &[],
        }
    }

    /// Sets the class builder to extend another class.
    ///
    /// # Parameters
    ///
    /// * `parent` - The parent class to extend.
    pub fn extends(mut self, parent: &'static ClassEntry) -> Self {
        self.extends = Some(parent);
        self
    }

    /// Implements an interface on the class.
    ///
    /// # Parameters
    ///
    /// * `interface` - Interface to implement on the class.
    ///
    /// # Panics
    ///
    /// Panics when the given class entry `interface` is not an interface.
    pub fn implements(mut self, interface: &'static ClassEntry) -> Self {
        assert!(
            interface.is_interface(),
            "Given class entry was not an interface."
        );
        self.interfaces.push(interface);
        self
    }

    /// Adds a method to the class.
    ///
    /// # Parameters
    ///
    /// * `func` - The function builder to add to the class.
    /// * `flags` - Flags relating to the function. See [`MethodFlags`].
    pub fn method(mut self, func: FunctionBuilder<'static>, flags: MethodFlags) -> Self {
        self.methods.push((func, flags));
        self
    }

    /// Adds a property to the class. The initial type of the property is given
    /// by the type of the given default. Note that the user can change the
    /// type.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the property to add to the class.
    /// * `default` - The default value of the property.
    /// * `flags` - Flags relating to the property. See [`PropertyFlags`].
    /// * `docs` - Documentation comments for the property.
    ///
    /// # Panics
    ///
    /// Function will panic if the given `default` cannot be converted into a
    /// [`Zval`].
    pub fn property<T: Into<String>>(
        mut self,
        name: T,
        flags: PropertyFlags,
        docs: DocComments,
    ) -> Self {
        self.properties.push((name.into(), flags, docs));
        self
    }

    /// Adds a constant to the class. The type of the constant is defined by the
    /// type of the given default.
    ///
    /// Returns a result containing the class builder if the constant was
    /// successfully added.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the constant to add to the class.
    /// * `value` - The value of the constant.
    /// * `docs` - Documentation comments for the constant.
    pub fn constant<T: Into<String>>(
        mut self,
        name: T,
        value: impl IntoZval + 'static,
        docs: DocComments,
    ) -> Result<Self> {
        self.constants
            .push((name.into(), Box::new(|| value.into_zval(true)), docs));
        Ok(self)
    }

    /// Adds a constant to the class from a `dyn` object. The type of the
    /// constant is defined by the type of the value.
    ///
    /// Returns a result containing the class builder if the constant was
    /// successfully added.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the constant to add to the class.
    /// * `value` - The value of the constant.
    /// * `docs` - Documentation comments for the constant.
    pub fn dyn_constant<T: Into<String>>(
        mut self,
        name: T,
        value: &'static dyn IntoZvalDyn,
        docs: DocComments,
    ) -> Result<Self> {
        let value = Rc::new(value);
        self.constants
            .push((name.into(), Box::new(move || value.as_zval(true)), docs));
        Ok(self)
    }

    /// Sets the flags for the class.
    ///
    /// # Parameters
    ///
    /// * `flags` - Flags relating to the class. See [`ClassFlags`].
    pub fn flags(mut self, flags: ClassFlags) -> Self {
        self.ce.ce_flags = flags.bits();
        self
    }

    /// Overrides the creation of the Zend object which will represent an
    /// instance of this class.
    ///
    /// # Parameters
    ///
    /// * `T` - The type which will override the Zend object. Must implement
    ///   [`RegisteredClass`] which can be derived using the
    ///   [`php_class`](crate::php_class) attribute macro.
    ///
    /// # Panics
    ///
    /// Panics if the class name associated with `T` is not the same as the
    /// class name specified when creating the builder.
    pub fn object_override<T: RegisteredClass>(mut self) -> Self {
        extern "C" fn create_object<T: RegisteredClass>(ce: *mut ClassEntry) -> *mut ZendObject {
            // SAFETY: After calling this function, PHP will always call the constructor
            // defined below, which assumes that the object is uninitialized.
            let obj = unsafe { ZendClassObject::<T>::new_uninit(ce.as_ref()) };
            obj.into_raw().get_mut_zend_obj()
        }

        zend_fastcall! {
            extern fn constructor<T: RegisteredClass>(ex: &mut ExecuteData, _: &mut Zval) {
                let ConstructorMeta { constructor, .. } = match T::constructor() {
                    Some(c) => c,
                    None => {
                        PhpException::default("You cannot instantiate this class from PHP.".into())
                            .throw()
                            .expect("Failed to throw exception when constructing class");
                        return;
                    }
                };

                let this = match constructor(ex) {
                    ConstructorResult::Ok(this) => this,
                    ConstructorResult::Exception(e) => {
                        e.throw()
                            .expect("Failed to throw exception while constructing class");
                        return;
                    }
                    ConstructorResult::ArgError => return,
                };
                let this_obj = match ex.get_object::<T>() {
                    Some(obj) => obj,
                    None => {
                        PhpException::default("Failed to retrieve reference to `this` object.".into())
                            .throw()
                            .expect("Failed to throw exception while constructing class");
                        return;
                    }
                };
                this_obj.initialize(this);
            }
        }

        debug_assert_eq!(
            self.name.as_str(),
            T::CLASS_NAME,
            "Class name in builder does not match class name in `impl RegisteredClass`."
        );
        self.object_override = Some(create_object::<T>);
        self.method(
            {
                let mut func = FunctionBuilder::new("__construct", constructor::<T>);
                if let Some(ConstructorMeta { build_fn, .. }) = T::constructor() {
                    func = build_fn(func);
                }
                func
            },
            MethodFlags::Public,
        )
    }

    /// Function to register the class with PHP. This function is called after
    /// the class is built.
    ///
    /// # Parameters
    ///
    /// * `register` - The function to call to register the class.
    pub fn registration(mut self, register: fn(&'static mut ClassEntry)) -> Self {
        self.register = Some(register);
        self
    }

    /// Sets the documentation for the class.
    ///
    /// # Parameters
    ///
    /// * `docs` - The documentation comments for the class.
    pub fn docs(mut self, docs: DocComments) -> Self {
        self.docs = docs;
        self
    }

    /// Builds and registers the class.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] variant if the class could not be registered.
    pub fn register(mut self) -> Result<()> {
        self.ce.name = ZendStr::new_interned(&self.name, true).into_raw();

        let mut methods = self
            .methods
            .into_iter()
            .map(|(method, flags)| {
                method.build().map(|mut method| {
                    method.flags |= flags.bits();
                    method
                })
            })
            .collect::<Result<Vec<_>>>()?;

        methods.push(FunctionEntry::end());
        let func = Box::into_raw(methods.into_boxed_slice()) as *const FunctionEntry;
        self.ce.info.internal.builtin_functions = func;

        let class = unsafe {
            zend_register_internal_class_ex(
                &mut self.ce,
                match self.extends {
                    Some(ptr) => (ptr as *const _) as *mut _,
                    None => std::ptr::null_mut(),
                },
            )
            .as_mut()
            .ok_or(Error::InvalidPointer)?
        };

        // disable serialization if the class has an associated object
        if self.object_override.is_some() {
            cfg_if::cfg_if! {
                if #[cfg(php81)] {
                    class.ce_flags |= ClassFlags::NotSerializable.bits();
                } else {
                    class.serialize = Some(crate::ffi::zend_class_serialize_deny);
                    class.unserialize = Some(crate::ffi::zend_class_unserialize_deny);
                }
            }
        }

        for iface in self.interfaces {
            unsafe {
                zend_do_implement_interface(
                    class,
                    iface as *const crate::ffi::_zend_class_entry
                        as *mut crate::ffi::_zend_class_entry,
                )
            };
        }

        for (name, flags, _) in self.properties {
            unsafe {
                zend_declare_property(
                    class,
                    CString::new(name.as_str())?.as_ptr(),
                    name.len() as _,
                    &mut Zval::new(),
                    flags.bits() as _,
                );
            }
        }

        for (name, value, _) in self.constants {
            let value = Box::into_raw(Box::new(value()?));
            unsafe {
                zend_declare_class_constant(
                    class,
                    CString::new(name.as_str())?.as_ptr(),
                    name.len(),
                    value,
                )
            };
        }

        if let Some(object_override) = self.object_override {
            class.__bindgen_anon_2.create_object = Some(object_override);
        }

        if let Some(register) = self.register {
            register(class);
        } else {
            panic!("Class {} was not registered.", self.name);
        }

        Ok(())
    }
}
