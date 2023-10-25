use crate::{
    builders::ClassBuilder,
    class::RegisteredClass,
    constant::IntoConst,
    error::Result,
    ffi::{ext_php_rs_php_build_id, ZEND_MODULE_API_NO},
    zend::{FunctionEntry, ModuleEntry},
    PHP_DEBUG, PHP_ZTS,
};

use std::{ffi::CString, fmt::Debug, mem, ptr};

/// Builds a Zend module extension to be registered with PHP. Must be called
/// from within an external function called `get_module`, returning a mutable
/// pointer to a `ModuleEntry`.
///
/// ```
/// use ext_php_rs::{
///     builders::ModuleBuilder,
///     zend::ModuleEntry,
///     info_table_start, info_table_end, info_table_row
/// };
///
/// #[no_mangle]
/// pub extern "C" fn php_module_info(_module: *mut ModuleEntry) {
///     info_table_start!();
///     info_table_row!("column 1", "column 2");
///     info_table_end!();
/// }
///
/// #[no_mangle]
/// pub extern "C" fn get_module() -> *mut ModuleEntry {
///     let (entry, _) = ModuleBuilder::new("ext-name", "ext-version")
///         .info_function(php_module_info)
///         .build()
///         .unwrap();
///     entry.into_raw()
/// }
/// ```
#[derive(Debug)]
pub struct ModuleBuilder {
    name: String,
    version: String,
    module: ModuleEntry,
    functions: Vec<FunctionEntry>,
    constants: Vec<(String, Box<dyn IntoConst + Send>)>,
    classes: Vec<fn()>,
}

impl ModuleBuilder {
    /// Creates a new module builder with a given name and version.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the extension.
    /// * `version` - The current version of the extension.
    pub fn new<T: Into<String>, U: Into<String>>(name: T, version: U) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            module: ModuleEntry {
                size: mem::size_of::<ModuleEntry>() as u16,
                zend_api: ZEND_MODULE_API_NO,
                zend_debug: u8::from(PHP_DEBUG),
                zts: u8::from(PHP_ZTS),
                ini_entry: ptr::null(),
                deps: ptr::null(),
                name: ptr::null(),
                functions: ptr::null(),
                module_startup_func: None,
                module_shutdown_func: None,
                request_startup_func: None,
                request_shutdown_func: None,
                info_func: None,
                version: ptr::null(),
                globals_size: 0,
                #[cfg(not(php_zts))]
                globals_ptr: ptr::null_mut(),
                #[cfg(php_zts)]
                globals_id_ptr: ptr::null_mut(),
                globals_ctor: None,
                globals_dtor: None,
                post_deactivate_func: None,
                module_started: 0,
                type_: 0,
                handle: ptr::null_mut(),
                module_number: 0,
                build_id: unsafe { ext_php_rs_php_build_id() },
            },
            functions: vec![],
            constants: vec![],
            classes: vec![],
        }
    }

    /// Sets the startup function for the extension.
    ///
    /// # Arguments
    ///
    /// * `func` - The function to be called on startup.
    pub fn startup_function(mut self, func: StartupShutdownFunc) -> Self {
        self.module.module_startup_func = Some(func);
        self
    }

    /// Sets the shutdown function for the extension.
    ///
    /// # Arguments
    ///
    /// * `func` - The function to be called on shutdown.
    pub fn shutdown_function(mut self, func: StartupShutdownFunc) -> Self {
        self.module.module_shutdown_func = Some(func);
        self
    }

    /// Sets the request startup function for the extension.
    ///
    /// # Arguments
    ///
    /// * `func` - The function to be called when startup is requested.
    pub fn request_startup_function(mut self, func: StartupShutdownFunc) -> Self {
        self.module.request_startup_func = Some(func);
        self
    }

    /// Sets the request shutdown function for the extension.
    ///
    /// # Arguments
    ///
    /// * `func` - The function to be called when shutdown is requested.
    pub fn request_shutdown_function(mut self, func: StartupShutdownFunc) -> Self {
        self.module.request_shutdown_func = Some(func);
        self
    }

    /// Sets the post request shutdown function for the extension.
    ///
    /// This function can be useful if you need to do any final cleanup at the
    /// very end of a request, after all other resources have been released. For
    /// example, if your extension creates any persistent resources that last
    /// beyond a single request, you could use this function to clean those up.
    /// # Arguments
    ///
    /// * `func` - The function to be called when shutdown is requested.
    pub fn post_deactivate_function(mut self, func: extern "C" fn() -> i32) -> Self {
        self.module.post_deactivate_func = Some(func);
        self
    }

    /// Sets the extension information function for the extension.
    ///
    /// # Arguments
    ///
    /// * `func` - The function to be called to retrieve the information about
    ///   the extension.
    pub fn info_function(mut self, func: InfoFunc) -> Self {
        self.module.info_func = Some(func);
        self
    }

    /// Adds a function to the extension.
    ///
    /// # Arguments
    ///
    /// * `func` - The function to be added to the extension.
    pub fn function(mut self, func: FunctionEntry) -> Self {
        self.functions.push(func);
        self
    }

    /// Adds a constant to the extension.
    ///
    /// # Arguments
    ///
    /// * `const` - Tuple containing the name and value of the constant. This is
    ///   a tuple to support the [`wrap_constant`] macro.
    ///
    /// [`wrap_constant`]: crate::wrap_constant
    pub fn constant(mut self, r#const: (&str, impl IntoConst + Send + 'static)) -> Self {
        let (name, val) = r#const;
        self.constants
            .push((name.into(), Box::new(val) as Box<dyn IntoConst + Send>));
        self
    }

    pub fn class<T: RegisteredClass>(mut self) -> Self {
        self.classes.push(|| {
            let mut builder = ClassBuilder::new(T::CLASS_NAME);
            for (method, flags) in T::method_builders() {
                builder = builder.method(method.build().expect("Failed to build method"), flags);
            }
            if let Some(extends) = T::EXTENDS {
                builder = builder.extends(extends());
            }
            for iface in T::IMPLEMENTS {
                builder = builder.implements(iface());
            }
            for (name, value) in T::constants() {
                builder = builder
                    .dyn_constant(*name, *value)
                    .expect("Failed to register constant");
            }
            if let Some(modifier) = T::BUILDER_MODIFIER {
                builder = modifier(builder);
            }
            let ce = builder
                .object_override::<T>()
                .build()
                .expect("Failed to build class");
            T::get_metadata().set_ce(ce);
        });
        self
    }

    /// Builds the extension and returns a `ModuleEntry`.
    ///
    /// Returns a result containing the module entry if successful.
    pub fn build(mut self) -> Result<(ModuleEntry, ModuleStartup)> {
        self.functions.push(FunctionEntry::end());
        self.module.functions =
            Box::into_raw(self.functions.into_boxed_slice()) as *const FunctionEntry;
        self.module.name = CString::new(self.name)?.into_raw();
        self.module.version = CString::new(self.version)?.into_raw();

        let startup = ModuleStartup {
            constants: self.constants,
            classes: self.classes,
        };
        Ok((self.module, startup))
    }
}

/// Artifacts from the [`ModuleBuilder`] that should be revisited inside the
/// extension startup function.
pub struct ModuleStartup {
    constants: Vec<(String, Box<dyn IntoConst + Send>)>,
    classes: Vec<fn()>,
}

impl ModuleStartup {
    /// Completes startup of the module. Should only be called inside the module
    /// startup function.
    pub fn startup(self, _ty: i32, mod_num: i32) -> Result<()> {
        for (name, val) in self.constants {
            val.register_constant(&name, mod_num)?;
        }
        for class in self.classes {
            class()
        }
        Ok(())
    }
}

/// A function to be called when the extension is starting up or shutting down.
pub type StartupShutdownFunc = extern "C" fn(_type: i32, _module_number: i32) -> i32;

/// A function to be called when `phpinfo();` is called.
pub type InfoFunc = extern "C" fn(zend_module: *mut ModuleEntry);
