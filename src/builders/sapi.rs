use crate::ffi::sapi_header_struct;
use crate::{embed::SapiModule, error::Result};

use std::ffi::{c_char, c_void};
use std::{ffi::CString, ptr};

/// Builder for `SapiModule`s
#[must_use]
pub struct SapiBuilder {
    name: String,
    pretty_name: String,
    module: SapiModule,
}

impl SapiBuilder {
    /// Creates a new [`SapiBuilder`] instance
    pub fn new<T: Into<String>, U: Into<String>>(name: T, pretty_name: U) -> Self {
        Self {
            name: name.into(),
            pretty_name: pretty_name.into(),
            module: SapiModule {
                name: ptr::null_mut(),
                pretty_name: ptr::null_mut(),
                startup: None,
                shutdown: None,
                activate: None,
                deactivate: None,
                ub_write: None,
                flush: None,
                get_stat: None,
                getenv: None,
                sapi_error: None,
                header_handler: None,
                send_headers: None,
                send_header: None,
                read_post: None,
                read_cookies: None,
                register_server_variables: None,
                log_message: None,
                get_request_time: None,
                terminate_process: None,
                php_ini_path_override: ptr::null_mut(),
                default_post_reader: None,
                treat_data: None,
                executable_location: ptr::null_mut(),
                php_ini_ignore: 0,
                php_ini_ignore_cwd: 0,
                get_fd: None,
                force_http_10: None,
                get_target_uid: None,
                get_target_gid: None,
                input_filter: None,
                ini_defaults: None,
                phpinfo_as_text: 0,
                ini_entries: ptr::null_mut(),
                additional_functions: ptr::null(),
                input_filter_init: None,
            },
        }
    }

    /// Sets the `ub_write` function for this SAPI
    pub fn ub_write_function(mut self, func: SapiUbWriteFunc) -> Self {
        self.module.ub_write = Some(func);
        self
    }

    /// Sets the send header function for this SAPI
    ///
    /// # Arguments
    ///
    /// * `func` - The function to be called on shutdown.
    pub fn send_header_function(mut self, func: SapiSendHeaderFunc) -> Self {
        self.module.send_header = Some(func);
        self
    }

    /// Builds the extension and returns a `SapiModule`.
    ///
    /// Returns a result containing the sapi module if successful.
    ///
    /// # Errors
    ///
    /// * If name or property name contain null bytes
    pub fn build(mut self) -> Result<SapiModule> {
        self.module.name = CString::new(self.name)?.into_raw();
        self.module.pretty_name = CString::new(self.pretty_name)?.into_raw();

        if self.module.send_header.is_none() {
            self.module.send_header = Some(dummy_send_header);
        }

        Ok(self.module)
    }
}

/// A function to be called when PHP send a header
pub type SapiSendHeaderFunc =
    extern "C" fn(header: *mut sapi_header_struct, server_context: *mut c_void);

/// A function to be called when PHP write to the output buffer
pub type SapiUbWriteFunc = extern "C" fn(str: *const c_char, str_length: usize) -> usize;

extern "C" fn dummy_send_header(_header: *mut sapi_header_struct, _server_context: *mut c_void) {}
