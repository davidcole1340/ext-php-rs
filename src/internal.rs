/// Called by startup functions registered with the `#[php_startup]` macro.
/// Initializes all classes that are defined by ext-php-rs (i.e. [`Closure`]).
///
/// [`Closure`]: ext_php_rs::php::types::closure::Closure
#[inline(always)]
pub fn ext_php_rs_startup() {
    #[cfg(feature = "closure")]
    crate::closure::Closure::build();
}
