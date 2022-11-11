//! Internal, public functions that are called from downstream extensions.

use parking_lot::{const_mutex, Mutex};

use crate::builders::ModuleStartup;

pub mod class;
pub mod function;

/// A mutex type that contains a [`ModuleStartup`] instance.
pub type ModuleStartupMutex = Mutex<Option<ModuleStartup>>;

/// The initialisation value for [`ModuleStartupMutex`]. By default the mutex
/// contains [`None`].
pub const MODULE_STARTUP_INIT: ModuleStartupMutex = const_mutex(None);

/// Called by startup functions registered with the [`#[php_startup]`] macro.
/// Initializes all classes that are defined by ext-php-rs (i.e. `Closure`).
///
/// [`#[php_startup]`]: crate::php_startup
#[inline(always)]
pub fn ext_php_rs_startup() {
    #[cfg(feature = "closure")]
    crate::closure::Closure::build();
}
