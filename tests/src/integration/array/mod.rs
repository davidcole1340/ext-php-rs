use std::collections::HashMap;

use ext_php_rs::{php_function, prelude::ModuleBuilder, wrap_function};

#[php_function]
pub fn test_array(a: Vec<String>) -> Vec<String> {
    a
}

#[php_function]
pub fn test_array_assoc(a: HashMap<String, String>) -> HashMap<String, String> {
    a
}

pub fn build_module(builder: ModuleBuilder) -> ModuleBuilder {
    builder
        .function(wrap_function!(test_array))
        .function(wrap_function!(test_array_assoc))
}

#[cfg(test)]
mod tests {
    #[test]
    fn array_works() {
        assert!(crate::integration::test::run_php("array/array.php"));
    }
}
