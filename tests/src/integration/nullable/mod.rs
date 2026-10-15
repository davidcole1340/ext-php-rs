use ext_php_rs::prelude::*;

#[php_function]
pub fn test_nullable(a: Option<String>) -> Option<String> {
    a
}

#[php_function]
pub fn test_nullable_int(a: Option<i64>) -> Option<i64> {
    a
}

#[php_function]
pub fn test_nullable_float(a: Option<f64>) -> Option<f64> {
    a
}

#[php_function]
pub fn test_nullable_ints(values: Vec<Option<i64>>) -> Vec<Option<i64>> {
    values
}

pub fn build_module(builder: ModuleBuilder) -> ModuleBuilder {
    builder
        .function(wrap_function!(test_nullable))
        .function(wrap_function!(test_nullable_int))
        .function(wrap_function!(test_nullable_float))
        .function(wrap_function!(test_nullable_ints))
}

#[cfg(test)]
mod tests {
    #[test]
    fn nullable_works() {
        assert!(crate::integration::test::run_php("nullable/nullable.php"));
    }
}
