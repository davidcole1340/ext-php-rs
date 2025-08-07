use std::{convert::TryFrom, ffi::CString, mem, ptr};

use super::{ClassBuilder, FunctionBuilder};
#[cfg(feature = "enum")]
use crate::{builders::enum_builder::EnumBuilder, enum_::RegisteredEnum};
use crate::{
    class::RegisteredClass,
    constant::IntoConst,
    describe::DocComments,
    error::Result,
    ffi::{ext_php_rs_php_build_id, ZEND_MODULE_API_NO},
    zend::{FunctionEntry, ModuleEntry},
    PHP_DEBUG, PHP_ZTS,
};

/// Builds a Zend module extension to be registered with PHP. Must be called
/// from within an external function called `get_module`, returning a mutable
/// pointer to a `ModuleEntry`.
///
/// ```rust,no_run
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
///         .try_into()
///         .unwrap();
///     entry.into_raw()
/// }
/// ```
#[must_use]
#[derive(Debug, Default)]
pub struct ModuleBuilder<'a> {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) functions: Vec<FunctionBuilder<'a>>,
    pub(crate) constants: Vec<(String, Box<dyn IntoConst + Send>, DocComments)>,
    pub(crate) classes: Vec<fn() -> ClassBuilder>,
    #[cfg(feature = "enum")]
    pub(crate) enums: Vec<fn() -> EnumBuilder>,
    startup_func: Option<StartupShutdownFunc>,
    shutdown_func: Option<StartupShutdownFunc>,
    request_startup_func: Option<StartupShutdownFunc>,
    request_shutdown_func: Option<StartupShutdownFunc>,
    post_deactivate_func: Option<unsafe extern "C" fn() -> i32>,
    info_func: Option<InfoFunc>,
}

impl ModuleBuilder<'_> {
    /// Creates a new module builder with a given name and version.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the extension.
    /// * `version` - The current version of the extension.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            functions: vec![],
            constants: vec![],
            classes: vec![],
            ..Default::default()
        }
    }

    /// Overrides module name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the extension.
    pub fn set_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Overrides module version.
    ///
    /// # Arguments
    ///
    /// * `version` - The current version of the extension.
    pub fn set_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Sets the startup function for the extension.
    ///
    /// # Arguments
    ///
    /// * `func` - The function to be called on startup.
    pub fn startup_function(mut self, func: StartupShutdownFunc) -> Self {
        self.startup_func = Some(func);
        self
    }

    /// Sets the shutdown function for the extension.
    ///
    /// # Arguments
    ///
    /// * `func` - The function to be called on shutdown.
    pub fn shutdown_function(mut self, func: StartupShutdownFunc) -> Self {
        self.shutdown_func = Some(func);
        self
    }

    /// Sets the request startup function for the extension.
    ///
    /// # Arguments
    ///
    /// * `func` - The function to be called when startup is requested.
    pub fn request_startup_function(mut self, func: StartupShutdownFunc) -> Self {
        self.request_startup_func = Some(func);
        self
    }

    /// Sets the request shutdown function for the extension.
    ///
    /// # Arguments
    ///
    /// * `func` - The function to be called when shutdown is requested.
    pub fn request_shutdown_function(mut self, func: StartupShutdownFunc) -> Self {
        self.request_shutdown_func = Some(func);
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
    pub fn post_deactivate_function(mut self, func: unsafe extern "C" fn() -> i32) -> Self {
        self.post_deactivate_func = Some(func);
        self
    }

    /// Sets the extension information function for the extension.
    ///
    /// # Arguments
    ///
    /// * `func` - The function to be called to retrieve the information about
    ///   the extension.
    pub fn info_function(mut self, func: InfoFunc) -> Self {
        self.info_func = Some(func);
        self
    }

    /// Adds a function to the extension.
    ///
    /// # Arguments
    ///
    /// * `func` - The function to be added to the extension.
    pub fn function(mut self, func: FunctionBuilder<'static>) -> Self {
        self.functions.push(func);
        self
    }

    /// Adds a constant to the extension.
    ///
    /// # Arguments
    ///
    /// * `const` - Tuple containing the name, value and doc comments for the
    ///   constant. This is a tuple to support the [`wrap_constant`] macro.
    ///
    /// [`wrap_constant`]: crate::wrap_constant
    pub fn constant(
        mut self,
        r#const: (&str, impl IntoConst + Send + 'static, DocComments),
    ) -> Self {
        let (name, val, docs) = r#const;
        self.constants.push((
            name.into(),
            Box::new(val) as Box<dyn IntoConst + Send>,
            docs,
        ));
        self
    }

    /// Adds a class to the extension.
    ///
    /// # Panics
    ///
    /// * Panics if a constant could not be registered.
    pub fn class<T: RegisteredClass>(mut self) -> Self {
        self.classes.push(|| {
            let mut builder = ClassBuilder::new(T::CLASS_NAME);
            for (method, flags) in T::method_builders() {
                builder = builder.method(method, flags);
            }
            if let Some(parent) = T::EXTENDS {
                builder = builder.extends(parent);
            }
            for interface in T::IMPLEMENTS {
                builder = builder.implements(*interface);
            }
            for (name, value, docs) in T::constants() {
                builder = builder
                    .dyn_constant(*name, *value, docs)
                    .expect("Failed to register constant");
            }
            for (name, prop_info) in T::get_properties() {
                builder = builder.property(name, prop_info.flags, prop_info.docs);
            }
            if let Some(modifier) = T::BUILDER_MODIFIER {
                builder = modifier(builder);
            }

            builder
                .object_override::<T>()
                .registration(|ce| {
                    T::get_metadata().set_ce(ce);
                })
                .docs(T::DOC_COMMENTS)
        });
        self
    }

    /// Adds an enum to the extension.
    #[cfg(feature = "enum")]
    pub fn enumeration<T>(mut self) -> Self
    where
        T: RegisteredClass + RegisteredEnum,
    {
        self.enums.push(|| {
            let mut builder = EnumBuilder::new(T::CLASS_NAME);
            for case in T::CASES {
                builder = builder.case(case);
            }
            for (method, flags) in T::method_builders() {
                builder = builder.method(method, flags);
            }

            builder
                .registration(|ce| {
                    T::get_metadata().set_ce(ce);
                })
                .docs(T::DOC_COMMENTS)
        });

        self
    }
}

/// Artifacts from the [`ModuleBuilder`] that should be revisited inside the
/// extension startup function.
pub struct ModuleStartup {
    constants: Vec<(String, Box<dyn IntoConst + Send>)>,
    classes: Vec<fn() -> ClassBuilder>,
    #[cfg(feature = "enum")]
    enums: Vec<fn() -> EnumBuilder>,
}

impl ModuleStartup {
    /// Completes startup of the module. Should only be called inside the module
    /// startup function.
    ///
    /// # Errors
    ///
    /// * Returns an error if a constant could not be registered.
    ///
    /// # Panics
    ///
    /// * Panics if a class could not be registered.
    pub fn startup(self, _ty: i32, mod_num: i32) -> Result<()> {
        for (name, val) in self.constants {
            val.register_constant(&name, mod_num)?;
        }

        self.classes.into_iter().map(|c| c()).for_each(|c| {
            c.register().expect("Failed to build class");
        });

        #[cfg(feature = "enum")]
        self.enums
            .into_iter()
            .map(|builder| builder())
            .for_each(|e| {
                e.register().expect("Failed to build enum");
            });

        Ok(())
    }
}

/// A function to be called when the extension is starting up or shutting down.
pub type StartupShutdownFunc = unsafe extern "C" fn(_type: i32, _module_number: i32) -> i32;

/// A function to be called when `phpinfo();` is called.
pub type InfoFunc = unsafe extern "C" fn(zend_module: *mut ModuleEntry);

/// Builds a [`ModuleEntry`] and [`ModuleStartup`] from a [`ModuleBuilder`].
/// This is the entry point for the module to be registered with PHP.
impl TryFrom<ModuleBuilder<'_>> for (ModuleEntry, ModuleStartup) {
    type Error = crate::error::Error;

    fn try_from(builder: ModuleBuilder) -> Result<Self, Self::Error> {
        let mut functions = builder
            .functions
            .into_iter()
            .map(FunctionBuilder::build)
            .collect::<Result<Vec<_>>>()?;
        functions.push(FunctionEntry::end());
        let functions = Box::into_raw(functions.into_boxed_slice()) as *const FunctionEntry;

        let name = CString::new(builder.name)?.into_raw();
        let version = CString::new(builder.version)?.into_raw();

        let startup = ModuleStartup {
            constants: builder
                .constants
                .into_iter()
                .map(|(n, v, _)| (n, v))
                .collect(),
            classes: builder.classes,
            #[cfg(feature = "enum")]
            enums: builder.enums,
        };

        Ok((
            ModuleEntry {
                size: mem::size_of::<ModuleEntry>().try_into()?,
                zend_api: ZEND_MODULE_API_NO,
                zend_debug: u8::from(PHP_DEBUG),
                zts: u8::from(PHP_ZTS),
                ini_entry: ptr::null(),
                deps: ptr::null(),
                name,
                functions,
                module_startup_func: builder.startup_func,
                module_shutdown_func: builder.shutdown_func,
                request_startup_func: builder.request_startup_func,
                request_shutdown_func: builder.request_shutdown_func,
                info_func: builder.info_func,
                version,
                globals_size: 0,
                #[cfg(not(php_zts))]
                globals_ptr: ptr::null_mut(),
                #[cfg(php_zts)]
                globals_id_ptr: ptr::null_mut(),
                globals_ctor: None,
                globals_dtor: None,
                post_deactivate_func: builder.post_deactivate_func,
                module_started: 0,
                type_: 0,
                handle: ptr::null_mut(),
                module_number: 0,
                build_id: unsafe { ext_php_rs_php_build_id() },
            },
            startup,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::test::{
        test_deactivate_function, test_function, test_info_function, test_startup_shutdown_function,
    };

    use super::*;

    #[test]
    fn test_new() {
        let builder = ModuleBuilder::new("test", "1.0");
        assert_eq!(builder.name, "test");
        assert_eq!(builder.version, "1.0");
        assert!(builder.functions.is_empty());
        assert!(builder.constants.is_empty());
        assert!(builder.classes.is_empty());
        assert!(builder.startup_func.is_none());
        assert!(builder.shutdown_func.is_none());
        assert!(builder.request_startup_func.is_none());
        assert!(builder.request_shutdown_func.is_none());
        assert!(builder.post_deactivate_func.is_none());
        assert!(builder.info_func.is_none());
        #[cfg(feature = "enum")]
        assert!(builder.enums.is_empty());
    }

    #[test]
    fn test_startup_function() {
        let builder =
            ModuleBuilder::new("test", "1.0").startup_function(test_startup_shutdown_function);
        assert!(builder.startup_func.is_some());
    }

    #[test]
    fn test_shutdown_function() {
        let builder =
            ModuleBuilder::new("test", "1.0").shutdown_function(test_startup_shutdown_function);
        assert!(builder.shutdown_func.is_some());
    }

    #[test]
    fn test_request_startup_function() {
        let builder = ModuleBuilder::new("test", "1.0")
            .request_startup_function(test_startup_shutdown_function);
        assert!(builder.request_startup_func.is_some());
    }

    #[test]
    fn test_request_shutdown_function() {
        let builder = ModuleBuilder::new("test", "1.0")
            .request_shutdown_function(test_startup_shutdown_function);
        assert!(builder.request_shutdown_func.is_some());
    }

    #[test]
    fn test_set_post_deactivate_function() {
        let builder =
            ModuleBuilder::new("test", "1.0").post_deactivate_function(test_deactivate_function);
        assert!(builder.post_deactivate_func.is_some());
    }

    #[test]
    fn test_set_info_function() {
        let builder = ModuleBuilder::new("test", "1.0").info_function(test_info_function);
        assert!(builder.info_func.is_some());
    }

    #[test]
    fn test_add_function() {
        let builder =
            ModuleBuilder::new("test", "1.0").function(FunctionBuilder::new("test", test_function));
        assert_eq!(builder.functions.len(), 1);
    }

    #[test]
    #[cfg(feature = "embed")]
    fn test_add_constant() {
        let builder =
            ModuleBuilder::new("test", "1.0").constant(("TEST_CONST", 42, DocComments::default()));
        assert_eq!(builder.constants.len(), 1);
        assert_eq!(builder.constants[0].0, "TEST_CONST");
        // TODO: Check if the value is 42
        assert_eq!(builder.constants[0].2, DocComments::default());
    }
}
