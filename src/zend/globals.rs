//! Types related to the PHP executor, sapi and process globals.
use std::ffi::CStr;
use std::ops::{Deref, DerefMut};
use std::slice;
use std::str;

use parking_lot::{const_rwlock, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::boxed::ZBox;
use crate::ffi::{
    _zend_executor_globals, ext_php_rs_executor_globals, ext_php_rs_process_globals,
    ext_php_rs_sapi_globals, php_core_globals, sapi_globals_struct, sapi_header_struct,
    sapi_headers_struct, sapi_request_info, zend_is_auto_global, TRACK_VARS_COOKIE, TRACK_VARS_ENV,
    TRACK_VARS_FILES, TRACK_VARS_GET, TRACK_VARS_POST, TRACK_VARS_REQUEST, TRACK_VARS_SERVER,
};

use crate::types::{ZendHashTable, ZendObject, ZendStr};

use super::linked_list::ZendLinkedListIterator;

/// Stores global variables used in the PHP executor.
pub type ExecutorGlobals = _zend_executor_globals;

impl ExecutorGlobals {
    /// Returns a reference to the PHP executor globals.
    ///
    /// The executor globals are guarded by a RwLock. There can be multiple
    /// immutable references at one time but only ever one mutable reference.
    /// Attempting to retrieve the globals while already holding the global
    /// guard will lead to a deadlock. Dropping the globals guard will release
    /// the lock.
    pub fn get() -> GlobalReadGuard<Self> {
        // SAFETY: PHP executor globals are statically declared therefore should never
        // return an invalid pointer.
        let globals = unsafe { ext_php_rs_executor_globals().as_ref() }
            .expect("Static executor globals were invalid");
        let guard = GLOBALS_LOCK.read();
        GlobalReadGuard { globals, guard }
    }

    /// Returns a mutable reference to the PHP executor globals.
    ///
    /// The executor globals are guarded by a RwLock. There can be multiple
    /// immutable references at one time but only ever one mutable reference.
    /// Attempting to retrieve the globals while already holding the global
    /// guard will lead to a deadlock. Dropping the globals guard will release
    /// the lock.
    pub fn get_mut() -> GlobalWriteGuard<Self> {
        // SAFETY: PHP executor globals are statically declared therefore should never
        // return an invalid pointer.
        let globals = unsafe { ext_php_rs_executor_globals().as_mut() }
            .expect("Static executor globals were invalid");
        let guard = GLOBALS_LOCK.write();
        GlobalWriteGuard { globals, guard }
    }

    /// Attempts to retrieve the global class hash table.
    pub fn class_table(&self) -> Option<&ZendHashTable> {
        unsafe { self.class_table.as_ref() }
    }

    /// Attempts to extract the last PHP exception captured by the interpreter.
    /// Returned inside a [`ZBox`].
    ///
    /// This function requires the executor globals to be mutably held, which
    /// could lead to a deadlock if the globals are already borrowed immutably
    /// or mutably.
    pub fn take_exception() -> Option<ZBox<ZendObject>> {
        let mut globals = Self::get_mut();

        let mut exception_ptr = std::ptr::null_mut();
        std::mem::swap(&mut exception_ptr, &mut globals.exception);

        // SAFETY: `as_mut` checks for null.
        Some(unsafe { ZBox::from_raw(exception_ptr.as_mut()?) })
    }
}

/// Stores global variables used in the PHP executor.
pub type ProcessGlobals = php_core_globals;

impl ProcessGlobals {
    /// Returns a reference to the PHP process globals.
    ///
    /// The process globals are guarded by a RwLock. There can be multiple
    /// immutable references at one time but only ever one mutable reference.
    /// Attempting to retrieve the globals while already holding the global
    /// guard will lead to a deadlock. Dropping the globals guard will release
    /// the lock.
    pub fn get() -> GlobalReadGuard<Self> {
        // SAFETY: PHP executor globals are statically declared therefore should never
        // return an invalid pointer.
        let globals = unsafe { &*ext_php_rs_process_globals() };
        let guard = PROCESS_GLOBALS_LOCK.read();
        GlobalReadGuard { globals, guard }
    }

    /// Returns a mutable reference to the PHP executor globals.
    ///
    /// The executor globals are guarded by a RwLock. There can be multiple
    /// immutable references at one time but only ever one mutable reference.
    /// Attempting to retrieve the globals while already holding the global
    /// guard will lead to a deadlock. Dropping the globals guard will release
    /// the lock.
    pub fn get_mut() -> GlobalWriteGuard<Self> {
        // SAFETY: PHP executor globals are statically declared therefore should never
        // return an invalid pointer.
        let globals = unsafe { &mut *ext_php_rs_process_globals() };
        let guard = PROCESS_GLOBALS_LOCK.write();
        GlobalWriteGuard { globals, guard }
    }

    /// Get the HTTP Server variables. Equivalent of $_SERVER.
    pub fn http_server_vars(&self) -> Option<&ZendHashTable> {
        // $_SERVER is lazy-initted, we need to call zend_is_auto_global
        // if it's not already populated.
        if !self.http_globals[TRACK_VARS_SERVER as usize].is_array() {
            let name = ZendStr::new("_SERVER", false).as_mut_ptr();
            unsafe { zend_is_auto_global(name) };
        }
        if self.http_globals[TRACK_VARS_SERVER as usize].is_array() {
            self.http_globals[TRACK_VARS_SERVER as usize].array()
        } else {
            None
        }
    }

    /// Get the HTTP POST variables. Equivalent of $_POST.
    pub fn http_post_vars(&self) -> &ZendHashTable {
        self.http_globals[TRACK_VARS_POST as usize]
            .array()
            .expect("Type is not a ZendArray")
    }

    /// Get the HTTP GET variables. Equivalent of $_GET.
    pub fn http_get_vars(&self) -> &ZendHashTable {
        self.http_globals[TRACK_VARS_GET as usize]
            .array()
            .expect("Type is not a ZendArray")
    }

    /// Get the HTTP Cookie variables. Equivalent of $_COOKIE.
    pub fn http_cookie_vars(&self) -> &ZendHashTable {
        self.http_globals[TRACK_VARS_COOKIE as usize]
            .array()
            .expect("Type is not a ZendArray")
    }

    /// Get the HTTP Request variables. Equivalent of $_REQUEST.
    pub fn http_request_vars(&self) -> &ZendHashTable {
        self.http_globals[TRACK_VARS_REQUEST as usize]
            .array()
            .expect("Type is not a ZendArray")
    }

    /// Get the HTTP Environment variables. Equivalent of $_ENV.
    pub fn http_env_vars(&self) -> &ZendHashTable {
        self.http_globals[TRACK_VARS_ENV as usize]
            .array()
            .expect("Type is not a ZendArray")
    }

    /// Get the HTTP Files variables. Equivalent of $_FILES.
    pub fn http_files_vars(&self) -> &ZendHashTable {
        self.http_globals[TRACK_VARS_FILES as usize]
            .array()
            .expect("Type is not a ZendArray")
    }
}

/// Stores global variables used in the SAPI.
pub type SapiGlobals = sapi_globals_struct;

impl SapiGlobals {
    /// Returns a reference to the PHP process globals.
    ///
    /// The process globals are guarded by a RwLock. There can be multiple
    /// immutable references at one time but only ever one mutable reference.
    /// Attempting to retrieve the globals while already holding the global
    /// guard will lead to a deadlock. Dropping the globals guard will release
    /// the lock.
    pub fn get() -> GlobalReadGuard<Self> {
        // SAFETY: PHP executor globals are statically declared therefore should never
        // return an invalid pointer.
        let globals = unsafe { &*ext_php_rs_sapi_globals() };
        let guard = SAPI_GLOBALS_LOCK.read();
        GlobalReadGuard { globals, guard }
    }

    /// Returns a mutable reference to the PHP executor globals.
    ///
    /// The executor globals are guarded by a RwLock. There can be multiple
    /// immutable references at one time but only ever one mutable reference.
    /// Attempting to retrieve the globals while already holding the global
    /// guard will lead to a deadlock. Dropping the globals guard will release
    /// the lock.
    pub fn get_mut() -> GlobalWriteGuard<Self> {
        // SAFETY: PHP executor globals are statically declared therefore should never
        // return an invalid pointer.
        let globals = unsafe { &mut *ext_php_rs_sapi_globals() };
        let guard = SAPI_GLOBALS_LOCK.write();
        GlobalWriteGuard { globals, guard }
    }
    // Get the request info for the Sapi.
    pub fn request_info(&self) -> &SapiRequestInfo {
        &self.request_info
    }

    pub fn sapi_headers(&self) -> &SapiHeaders {
        &self.sapi_headers
    }
}

pub type SapiHeaders = sapi_headers_struct;

impl<'a> SapiHeaders {
    pub fn headers(&'a mut self) -> ZendLinkedListIterator<'a, SapiHeader> {
        self.headers.iter()
    }
}

pub type SapiHeader = sapi_header_struct;

impl<'a> SapiHeader {
    pub fn as_str(&'a self) -> &'a str {
        unsafe {
            let slice = slice::from_raw_parts(self.header as *const u8, self.header_len);
            str::from_utf8(slice).expect("Invalid header string")
        }
    }

    pub fn name(&'a self) -> &'a str {
        self.as_str().split(':').next().unwrap_or("").trim()
    }

    pub fn value(&'a self) -> Option<&'a str> {
        self.as_str().split(':').nth(1).map(|s| s.trim())
    }
}

pub type SapiRequestInfo = sapi_request_info;

impl SapiRequestInfo {
    pub fn request_method(&self) -> Option<&str> {
        if self.request_method.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(self.request_method).to_str().ok() }
    }

    pub fn query_string(&self) -> Option<&str> {
        if self.query_string.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(self.query_string).to_str().ok() }
    }

    pub fn cookie_data(&self) -> Option<&str> {
        if self.cookie_data.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(self.cookie_data).to_str().ok() }
    }

    pub fn content_length(&self) -> i64 {
        self.content_length
    }

    pub fn path_translated(&self) -> Option<&str> {
        if self.path_translated.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(self.path_translated).to_str().ok() }
    }

    pub fn request_uri(&self) -> Option<&str> {
        if self.request_uri.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(self.request_uri).to_str().ok() }
    }

    // Todo: request_body _php_stream

    pub fn content_type(&self) -> Option<&str> {
        if self.content_type.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(self.content_type).to_str().ok() }
    }

    pub fn headers_only(&self) -> bool {
        self.headers_only
    }

    pub fn no_headers(&self) -> bool {
        self.no_headers
    }

    pub fn headers_read(&self) -> bool {
        self.headers_read
    }

    // Todo: post_entry sapi_post_entry

    pub fn auth_user(&self) -> Option<&str> {
        if self.auth_user.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(self.auth_user).to_str().ok() }
    }

    pub fn auth_password(&self) -> Option<&str> {
        if self.auth_password.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(self.auth_password).to_str().ok() }
    }

    pub fn auth_digest(&self) -> Option<&str> {
        if self.auth_digest.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(self.auth_digest).to_str().ok() }
    }

    pub fn argv0(&self) -> Option<&str> {
        if self.argv0.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(self.argv0).to_str().ok() }
    }

    pub fn current_user(&self) -> Option<&str> {
        if self.current_user.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(self.current_user).to_str().ok() }
    }

    pub fn current_user_length(&self) -> i32 {
        self.current_user_length
    }

    pub fn argvc(&self) -> i32 {
        self.argc
    }

    pub fn argv(&self) -> Option<&str> {
        if self.argv.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(*self.argv).to_str().ok() }
    }

    pub fn proto_num(&self) -> i32 {
        self.proto_num
    }
}

/// Executor globals rwlock.
///
/// PHP provides no indication if the executor globals are being accessed so
/// this is only effective on the Rust side.
static GLOBALS_LOCK: RwLock<()> = const_rwlock(());
static PROCESS_GLOBALS_LOCK: RwLock<()> = const_rwlock(());
static SAPI_GLOBALS_LOCK: RwLock<()> = const_rwlock(());

/// Wrapper guard that contains a reference to a given type `T`. Dropping a
/// guard releases the lock on the relevant rwlock.
pub struct GlobalReadGuard<T: 'static> {
    globals: &'static T,
    #[allow(dead_code)]
    guard: RwLockReadGuard<'static, ()>,
}

impl<T> Deref for GlobalReadGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.globals
    }
}

/// Wrapper guard that contains a mutable reference to a given type `T`.
/// Dropping a guard releases the lock on the relevant rwlock.
pub struct GlobalWriteGuard<T: 'static> {
    globals: &'static mut T,
    #[allow(dead_code)]
    guard: RwLockWriteGuard<'static, ()>,
}

impl<T> Deref for GlobalWriteGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.globals
    }
}

impl<T> DerefMut for GlobalWriteGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.globals
    }
}
