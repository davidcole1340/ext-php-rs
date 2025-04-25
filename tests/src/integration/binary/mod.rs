use ext_php_rs::{binary::Binary, prelude::*};

#[php_function]
pub fn test_binary(a: Binary<u32>) -> Binary<u32> {
    a
}

pub fn build_module(builder: ModuleBuilder) -> ModuleBuilder {
    builder.function(wrap_function!(test_binary))
}

#[cfg(test)]
mod tests {
    #[test]
    fn binary_works() {
        assert!(crate::integration::test::run_php("binary/binary.php"));
    }
}
