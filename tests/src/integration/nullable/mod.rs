use ext_php_rs::prelude::*;

#[php_function]
pub fn test_nullable(a: Option<String>) -> Option<String> {
    a
}

pub fn build_module(builder: ModuleBuilder) -> ModuleBuilder {
    builder.function(wrap_function!(test_nullable))
}

#[cfg(test)]
mod tests {
    #[test]
    fn nullable_works() {
        assert!(crate::integration::test::run_php("nullable/nullable.php"));
    }
}
