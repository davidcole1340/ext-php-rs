//! Types related to the PHP executor globals.

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use parking_lot::{const_rwlock, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::boxed::ZBox;
#[cfg(php82)]
use crate::ffi::zend_atomic_bool_store;
use crate::ffi::{
    _sapi_globals_struct, _sapi_module_struct, _zend_executor_globals, ext_php_rs_executor_globals,
    ext_php_rs_sapi_globals, ext_php_rs_sapi_module, zend_ini_entry,
};
use crate::types::{ZendHashTable, ZendObject};

/// Stores global variables used in the PHP executor.
pub type ExecutorGlobals = _zend_executor_globals;

/// Stores global SAPI variables used in the PHP executor.
pub type SapiGlobals = _sapi_globals_struct;

/// Stores the SAPI module used in the PHP executor.
pub type SapiModule = _sapi_module_struct;

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

    /// Retrieves the ini values for all ini directives in the current executor
    /// context..
    pub fn ini_values(&self) -> HashMap<String, Option<String>> {
        let hash_table = unsafe { &*self.ini_directives };
        let mut ini_hash_map: HashMap<String, Option<String>> = HashMap::new();
        for (_index, key, value) in hash_table.iter() {
            if let Some(key) = key {
                ini_hash_map.insert(key, unsafe {
                    let ini_entry = &*value.ptr::<zend_ini_entry>().expect("Invalid ini entry");
                    if ini_entry.value.is_null() {
                        None
                    } else {
                        Some(
                            (*ini_entry.value)
                                .as_str()
                                .expect("Ini value is not a string")
                                .to_owned(),
                        )
                    }
                });
            }
        }
        ini_hash_map
    }

    /// Attempts to retrieve the global constants table.
    pub fn constants(&self) -> Option<&ZendHashTable> {
        unsafe { self.zend_constants.as_ref() }
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

    /// Request an interrupt of the PHP VM. This will call the registered
    /// interrupt handler function.
    /// set with [`crate::ffi::zend_interrupt_function`].
    pub fn request_interrupt(&mut self) {
        cfg_if::cfg_if! {
            if #[cfg(php82)] {
                unsafe {
                    zend_atomic_bool_store(&mut self.vm_interrupt, true);
                }
            } else {
                self.vm_interrupt = true;
            }
        }
    }

    /// Cancel a requested an interrupt of the PHP VM.
    pub fn cancel_interrupt(&mut self) {
        cfg_if::cfg_if! {
            if #[cfg(php82)] {
                unsafe {
                    zend_atomic_bool_store(&mut self.vm_interrupt, false);
                }
            } else {
                self.vm_interrupt = true;
            }
        }
    }
}

impl SapiGlobals {
    /// Returns a reference to the PHP SAPI globals.
    ///
    /// The executor globals are guarded by a RwLock. There can be multiple
    /// immutable references at one time but only ever one mutable reference.
    /// Attempting to retrieve the globals while already holding the global
    /// guard will lead to a deadlock. Dropping the globals guard will release
    /// the lock.
    pub fn get() -> GlobalReadGuard<Self> {
        // SAFETY: PHP executor globals are statically declared therefore should never
        // return an invalid pointer.
        let globals = unsafe { ext_php_rs_sapi_globals().as_ref() }
            .expect("Static executor globals were invalid");
        let guard = SAPI_LOCK.read();
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
        let globals = unsafe { ext_php_rs_sapi_globals().as_mut() }
            .expect("Static executor globals were invalid");
        let guard = SAPI_LOCK.write();
        GlobalWriteGuard { globals, guard }
    }
}

impl SapiModule {
    /// Returns a reference to the PHP SAPI module.
    ///
    /// The executor globals are guarded by a RwLock. There can be multiple
    /// immutable references at one time but only ever one mutable reference.
    /// Attempting to retrieve the globals while already holding the global
    /// guard will lead to a deadlock. Dropping the globals guard will release
    /// the lock.
    pub fn get() -> GlobalReadGuard<Self> {
        // SAFETY: PHP executor globals are statically declared therefore should never
        // return an invalid pointer.
        let globals = unsafe { ext_php_rs_sapi_module().as_ref() }
            .expect("Static executor globals were invalid");
        let guard = SAPI_MODULE_LOCK.read();
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
        let globals = unsafe { ext_php_rs_sapi_module().as_mut() }
            .expect("Static executor globals were invalid");
        let guard = SAPI_MODULE_LOCK.write();
        GlobalWriteGuard { globals, guard }
    }
}

/// Executor globals rwlock.
///
/// PHP provides no indication if the executor globals are being accessed so
/// this is only effective on the Rust side.
static GLOBALS_LOCK: RwLock<()> = const_rwlock(());

/// SAPI globals rwlock.
///
/// PHP provides no indication if the executor globals are being accessed so
/// this is only effective on the Rust side.
static SAPI_LOCK: RwLock<()> = const_rwlock(());

/// SAPI globals rwlock.
///
/// PHP provides no indication if the executor globals are being accessed so
/// this is only effective on the Rust side.
static SAPI_MODULE_LOCK: RwLock<()> = const_rwlock(());

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
