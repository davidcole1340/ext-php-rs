extern crate ext_php_rs;
use ext_php_rs::prelude::*;

#[php_function]
pub fn test_string(str: String) -> String {
    str
}

#[php_function]
pub fn test_str(str: &str) -> &str {
    str
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
