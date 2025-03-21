//! Structures that are used to construct other, more complicated types.
//! Generally zero-cost abstractions.

mod class;
mod function;
mod module;
#[cfg(all(feature = "embed", any(php81, not(php_zts))))]
mod sapi;

pub use class::ClassBuilder;
pub use function::FunctionBuilder;
pub use module::{ModuleBuilder, ModuleStartup};
#[cfg(all(feature = "embed", any(php81, not(php_zts))))]
pub use sapi::SapiBuilder;
