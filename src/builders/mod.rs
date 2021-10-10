//! Structures that are used to construct other, more complicated types.
//! Generally zero-cost abstractions.

mod class;
#[cfg(any(docs, feature = "php81"))]
#[cfg_attr(docs, doc(cfg(feature = "php81")))]
mod enums;
mod function;
mod module;

pub use class::ClassBuilder;
#[cfg(any(docs, feature = "php81"))]
#[cfg_attr(docs, doc(cfg(feature = "php81")))]
pub use enums::{BackedEnumBuilder, UnbackedEnumBuilder};
pub use function::FunctionBuilder;
pub use module::ModuleBuilder;
