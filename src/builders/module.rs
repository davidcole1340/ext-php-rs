use crate::{
    error::Result,
    ffi::{ext_php_rs_php_build_id, ZEND_MODULE_API_NO},
    zend::{FunctionEntry, ModuleEntry},
    PHP_DEBUG, PHP_ZTS,
};

use std::{ffi::CString, mem, ptr};

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
///     ModuleBuilder::new("ext-name", "ext-version")
///         .info_function(php_module_info)
///         .build()
///         .unwrap()
///         .into_raw()
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ModuleBuilder {
    name: String,
    version: String,
    module: ModuleEntry,
    functions: Vec<FunctionEntry>,
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
    /// beyond a single request, you could use this function to clean those up. # Arguments
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

    /// Builds the extension and returns a `ModuleEntry`.
    ///
    /// Returns a result containing the module entry if successful.
    pub fn build(mut self) -> Result<ModuleEntry> {
        self.functions.push(FunctionEntry::end());
        self.module.functions =
            Box::into_raw(self.functions.into_boxed_slice()) as *const FunctionEntry;
        self.module.name = CString::new(self.name)?.into_raw();
        self.module.version = CString::new(self.version)?.into_raw();

        Ok(self.module)
    }
}

/// A function to be called when the extension is starting up or shutting down.
pub type StartupShutdownFunc = extern "C" fn(_type: i32, _module_number: i32) -> i32;

/// A function to be called when `phpinfo();` is called.
pub type InfoFunc = extern "C" fn(zend_module: *mut ModuleEntry);
