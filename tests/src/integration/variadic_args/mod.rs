//! Rust type &[&Zval] must be converted to Vec<Zval> because of
//! lifetime hell.

use ext_php_rs::{prelude::*, types::Zval};

#[php_function]
pub fn test_variadic_args(params: &[&Zval]) -> Vec<Zval> {
    params.iter().map(|x| x.shallow_clone()).collect()
}

#[php_function]
pub fn test_variadic_add_required(number: u32, numbers: &[&Zval]) -> u32 {
    number
        + numbers
            .iter()
            .map(|x| u32::try_from(x.long().unwrap()).unwrap())
            .sum::<u32>()
}

pub fn build_module(builder: ModuleBuilder) -> ModuleBuilder {
    builder
        .function(wrap_function!(test_variadic_args))
        .function(wrap_function!(test_variadic_add_required))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_variadic_args() {
        assert!(crate::integration::test::run_php(
            "variadic_args/variadic_args.php"
        ));
    }
}
