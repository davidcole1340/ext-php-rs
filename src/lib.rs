#![deny(clippy::unwrap_used)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[macro_use]
pub mod macros;
pub mod bindings;
pub mod errors;
pub mod functions;
pub mod php;

pub use ext_php_rs_derive::php_function;
pub use ext_php_rs_derive::php_method;
pub use ext_php_rs_derive::php_module;
pub use ext_php_rs_derive::ZendObjectHandler;
