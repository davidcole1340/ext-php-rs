extern crate ext_php_rs;
use ext_php_rs::prelude::*;
use std::collections::HashMap;

#[php_function]
pub fn test_array(vec: Vec<String>) -> Vec<String> {
    vec
}

#[php_function]
pub fn test_assoc_array(arr: HashMap<String, i32>) -> HashMap<String, i32> {
    arr
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
