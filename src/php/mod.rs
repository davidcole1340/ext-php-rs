//! Objects relating to PHP and the Zend engine.

#[cfg(any(docs, feature = "alloc"))]
#[cfg_attr(docs, doc(cfg(feature = "alloc")))]
pub mod alloc;

pub mod args;
pub mod class;
pub mod constants;
pub mod enums;
pub mod exceptions;
pub mod execution_data;
pub mod flags;
pub mod function;
pub mod globals;
pub mod module;
pub mod pack;
pub mod types;
