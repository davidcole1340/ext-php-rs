//! Structures that are used to construct other, more complicated types.
//! Generally zero-cost abstractions.

mod class;
mod function;
#[cfg(php82)]
mod ini;
mod module;
#[cfg(feature = "embed")]
mod sapi;

pub use class::ClassBuilder;
pub use function::FunctionBuilder;
#[cfg(php82)]
pub use ini::IniBuilder;
pub use module::{ModuleBuilder, ModuleStartup};
#[cfg(feature = "embed")]
pub use sapi::SapiBuilder;
