#![cfg_attr(windows, feature(abi_vectorcall))]
#![cfg(feature = "embed")]
extern crate ext_php_rs;

use ext_php_rs::embed::Embed;
use ext_php_rs::ffi::zend_register_module_ex;
use ext_php_rs::prelude::*;

#[test]
fn test_module() {
    Embed::run(|| {
        // Allow to load the module
        unsafe { zend_register_module_ex(get_module()) };

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
    format!("Hello, {}!", name)
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module.function(wrap_function!(hello_world))
}
