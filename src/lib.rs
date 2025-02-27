#![doc = include_str!("../README.md")]
#![deny(clippy::unwrap_used)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![cfg_attr(docs, feature(doc_cfg))]
#![cfg_attr(windows, feature(abi_vectorcall))]

pub mod alloc;
pub mod args;
pub mod binary;
pub mod binary_slice;
pub mod builders;
pub mod convert;
pub mod error;
pub mod exception;
pub mod ffi;
pub mod flags;
#[macro_use]
pub mod macros;
pub mod boxed;
pub mod class;
#[cfg(any(docs, feature = "closure"))]
#[cfg_attr(docs, doc(cfg(feature = "closure")))]
pub mod closure;
pub mod constant;
pub mod describe;
#[cfg(feature = "embed")]
pub mod embed;
#[doc(hidden)]
pub mod internal;
pub mod props;
pub mod rc;
pub mod types;
pub mod zend;

/// A module typically glob-imported containing the typically required macros
/// and imports.
pub mod prelude {

    pub use crate::builders::ModuleBuilder;
    #[cfg(any(docs, feature = "closure"))]
    #[cfg_attr(docs, doc(cfg(feature = "closure")))]
    pub use crate::closure::Closure;
    pub use crate::exception::{PhpException, PhpResult};
    pub use crate::php_class;
    pub use crate::php_const;
    pub use crate::php_extern;
    pub use crate::php_function;
    pub use crate::php_impl;
    pub use crate::php_module;
    pub use crate::php_print;
    pub use crate::php_println;
    pub use crate::php_startup;
    pub use crate::types::ZendCallable;
    pub use crate::ZvalConvert;
}

/// `ext-php-rs` version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Whether the extension is compiled for PHP debug mode.
pub const PHP_DEBUG: bool = cfg!(php_debug);

/// Whether the extension is compiled for PHP thread-safe mode.
pub const PHP_ZTS: bool = cfg!(php_zts);

pub use ext_php_rs_derive::php_class;
pub use ext_php_rs_derive::php_const;
pub use ext_php_rs_derive::php_extern;
pub use ext_php_rs_derive::php_function;
pub use ext_php_rs_derive::php_impl;
pub use ext_php_rs_derive::php_module;
pub use ext_php_rs_derive::php_startup;
pub use ext_php_rs_derive::zend_fastcall;
pub use ext_php_rs_derive::ZvalConvert;
