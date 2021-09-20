extern crate ext_php_rs;
use ext_php_rs::prelude::*;

#[php_function]
pub fn hello_world() -> String {
    "Hello world".into()
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
