use std::{
    ffi::c_void,
    mem,
    os::raw::{c_char, c_int},
    ptr,
};

use crate::{
    bindings::{
        zend_function_entry, zend_module_entry, zend_result, USING_ZTS, ZEND_DEBUG,
        ZEND_MODULE_API_NO,
    },
    functions::{build_id, c_str},
};

pub type ModuleEntry = zend_module_entry;
pub type FunctionEntry = zend_function_entry;
pub type StartupShutdownFunc = extern "C" fn(type_: c_int, module_number: c_int) -> zend_result;
pub type InfoFunc = extern "C" fn(zend_module: *mut ModuleEntry);

/// Builds a Zend extension. Must be called from within an external function called `get_module`,
/// returning a mutable pointer to a `ModuleEntry`.
///
/// ```
/// #[no_mangle]
/// pub extern "C" fn php_module_info(_module: *mut ModuleEntry) {
///     print_table_start();
///     print_table_row("column 1", "column 2");
///     print_table_end();
/// }
///
/// #[no_mangle]
/// pub extern "C" fn get_module() -> *mut ModuleEntry {
///     ModuleBuilder::new("ext-name", "ext-version")
///         .info_function(php_module_info)
///         .build()
///         .into_raw()
/// }
/// ```
pub struct ModuleBuilder {
    module: ModuleEntry,
    functions: Vec<FunctionEntry>,
}

impl ModuleBuilder {
    /// Creates a new module builder with a given name and version.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the extension.
    /// * `version` - The current version of the extension. TBD: Deprecate in favour of the `Cargo.toml` version?
    pub fn new<N, V>(name: N, version: V) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        Self {
            module: ModuleEntry {
                size: mem::size_of::<ModuleEntry>() as u16,
                zend_api: ZEND_MODULE_API_NO,
                zend_debug: ZEND_DEBUG as u8,
                zts: USING_ZTS as u8,
                ini_entry: ptr::null(),
                deps: ptr::null(),
                name: c_str(name),
                functions: ptr::null(),
                module_startup_func: None,
                module_shutdown_func: None,
                request_startup_func: None,
                request_shutdown_func: None,
                info_func: None,
                version: c_str(version),
                globals_size: 0,
                globals_ptr: ptr::null::<c_void>() as *mut c_void,
                globals_ctor: None,
                globals_dtor: None,
                post_deactivate_func: None,
                module_started: 0,
                type_: 0,
                handle: ptr::null::<c_void>() as *mut c_void,
                module_number: 0,
                build_id: c_str(build_id()),
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
        self.module.module_startup_func = Some(func);
        self
    }

    /// Sets the request shutdown function for the extension.
    ///
    /// # Arguments
    ///
    /// * `func` - The function to be called when shutdown is requested.
    pub fn request_shutdown_function(mut self, func: StartupShutdownFunc) -> Self {
        self.module.module_shutdown_func = Some(func);
        self
    }

    /// Sets the extension information function for the extension.
    ///
    /// # Arguments
    ///
    /// * `func` - The function to be called to retrieve the information about the extension.
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
    pub fn build(mut self) -> ModuleEntry {
        // TODO: move to seperate function
        self.functions.push(FunctionEntry {
            fname: ptr::null() as *const c_char,
            handler: None,
            arg_info: ptr::null(),
            num_args: 0,
            flags: 0,
        });
        self.module.functions =
            Box::into_raw(self.functions.into_boxed_slice()) as *const FunctionEntry;
        self.module
    }
}

impl ModuleEntry {
    /// Converts the module entry into a raw pointer, releasing it to the C world.
    pub fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }
}
