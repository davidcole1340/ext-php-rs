extern crate ext_php_rs;
use ext_php_rs::prelude::*;

#[php_function]
pub fn test() {
    println!("hello world");
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
