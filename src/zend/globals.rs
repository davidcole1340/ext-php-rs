//! Types related to the PHP executor globals.

use std::ops::{Deref, DerefMut};

use parking_lot::{const_rwlock, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::boxed::ZBox;
use crate::ffi::{
    _zend_executor_globals, core_globals, ext_php_rs_executor_globals, php_core_globals,
    TRACK_VARS_COOKIE, TRACK_VARS_ENV, TRACK_VARS_FILES, TRACK_VARS_GET, TRACK_VARS_POST,
    TRACK_VARS_REQUEST, TRACK_VARS_SERVER,
};

use crate::types::{ZendHashTable, ZendObject};

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
        let globals = unsafe { &core_globals };
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
        let globals = unsafe { &mut core_globals };
        let guard = PROCESS_GLOBALS_LOCK.write();
        GlobalWriteGuard { globals, guard }
    }

    /// Get the HTTP Server variables. Equivalent of $_SERVER.
    pub fn http_server_vars(&self) -> Option<&ZendHashTable> {
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

/// Executor globals rwlock.
///
/// PHP provides no indication if the executor globals are being accessed so
/// this is only effective on the Rust side.
static GLOBALS_LOCK: RwLock<()> = const_rwlock(());
static PROCESS_GLOBALS_LOCK: RwLock<()> = const_rwlock(());

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
