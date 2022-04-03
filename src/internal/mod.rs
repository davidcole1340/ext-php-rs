//! Internal, public functions that are called from downstream extensions.

pub mod class;
pub mod function;

/// Called by startup functions registered with the [`#[php_startup]`] macro.
/// Initializes all classes that are defined by ext-php-rs (i.e. `Closure`).
///
/// [`#[php_startup]`]: crate::php_startup
#[inline(always)]
pub fn ext_php_rs_startup() {
    #[cfg(feature = "closure")]
    crate::closure::Closure::build();
}
