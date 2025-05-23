use ext_php_rs::prelude::*;

#[php_function]
#[php(defaults(a = 42))]
pub fn test_defaults_integer(a: i32) -> i32 {
    a
}

#[php_function]
#[php(defaults(a = None))]
pub fn test_defaults_nullable_string(a: Option<String>) -> Option<String> {
    a
}

pub fn build_module(builder: ModuleBuilder) -> ModuleBuilder {
    builder
        .function(wrap_function!(test_defaults_integer))
        .function(wrap_function!(test_defaults_nullable_string))
}

#[cfg(test)]
mod tests {
    #[test]
    fn defaults_works() {
        assert!(crate::integration::test::run_php("defaults/defaults.php"));
    }
}
