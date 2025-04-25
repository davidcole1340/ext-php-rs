use ext_php_rs::prelude::*;

#[php_function]
pub fn test_str(a: &str) -> &str {
    a
}

#[php_function]
pub fn test_string(a: String) -> String {
    a
}

pub fn build_module(builder: ModuleBuilder) -> ModuleBuilder {
    builder
        .function(wrap_function!(test_str))
        .function(wrap_function!(test_string))
}

#[cfg(test)]
mod tests {
    #[test]
    fn string_works() {
        assert!(crate::integration::test::run_php("string/string.php"));
    }
}
