//! Module tests
#![cfg_attr(windows, feature(abi_vectorcall))]
#![cfg(feature = "embed")]
#![allow(
    missing_docs,
    clippy::needless_pass_by_value,
    clippy::must_use_candidate
)]
extern crate ext_php_rs;

use cfg_if::cfg_if;

use ext_php_rs::embed::Embed;
use ext_php_rs::ffi::zend_register_module_ex;
use ext_php_rs::prelude::*;

#[test]
fn test_module() {
    Embed::run(|| {
        // Allow to load the module
        cfg_if! {
            if #[cfg(php84)] {
                // Register as temporary (2) module
                unsafe { zend_register_module_ex(get_module(), 2) };
            } else {
                unsafe { zend_register_module_ex(get_module()) };
            }
        }

        let result = Embed::eval("$foo = hello_world('foo');");

        assert!(result.is_ok());

        let zval = result.unwrap();

        assert!(zval.is_string());

        let string = zval.string().unwrap();

        assert_eq!(string.to_string(), "Hello, foo!");
    });
}

/// Gives you a nice greeting!
///
/// @param string $name Your name.
///
/// @return string Nice greeting!
#[php_function]
pub fn hello_world(name: String) -> String {
    format!("Hello, {name}!")
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module.function(wrap_function!(hello_world))
}
