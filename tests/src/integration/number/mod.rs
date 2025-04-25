use ext_php_rs::prelude::*;

#[php_function]
pub fn test_number_signed(a: i32) -> i32 {
    a
}

#[php_function]
pub fn test_number_unsigned(a: u32) -> u32 {
    a
}

#[php_function]
pub fn test_number_float(a: f32) -> f32 {
    a
}

pub fn build_module(builder: ModuleBuilder) -> ModuleBuilder {
    builder
        .function(wrap_function!(test_number_signed))
        .function(wrap_function!(test_number_unsigned))
        .function(wrap_function!(test_number_float))
}

#[cfg(test)]
mod tests {
    #[test]
    fn number_works() {
        assert!(crate::integration::test::run_php("number/number.php"));
    }
}
