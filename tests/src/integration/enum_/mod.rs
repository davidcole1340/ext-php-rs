use ext_php_rs::{error::Result, php_enum, php_function, prelude::ModuleBuilder, wrap_function};

#[php_enum]
#[php(allow_native_discriminants)]
pub enum TestEnum {
    // #[php(discriminant = 2)]
    Variant1,
    // #[php(discriminant = 1)]
    Variant2 = 1,
}

#[php_enum]
pub enum IntBackedEnum {
    #[php(discriminant = 1)]
    Variant1,
    #[php(discriminant = 2)]
    Variant2,
}

#[php_enum]
pub enum StringBackedEnum {
    #[php(discriminant = "foo")]
    Variant1,
    #[php(discriminant = "bar")]
    Variant2,
}

#[php_function]
pub fn test_enum(a: TestEnum) -> Result<StringBackedEnum> {
    let str: &str = StringBackedEnum::Variant2.into();
    match a {
        TestEnum::Variant1 => str.try_into(),
        TestEnum::Variant2 => Ok(StringBackedEnum::Variant1),
    }
}

pub fn build_module(builder: ModuleBuilder) -> ModuleBuilder {
    builder
        .r#enum::<TestEnum>()
        .r#enum::<IntBackedEnum>()
        .r#enum::<StringBackedEnum>()
        .function(wrap_function!(test_enum))
}

#[cfg(test)]
mod tests {
    #[test]
    fn enum_works() {
        assert!(crate::integration::test::run_php("enum_/enum.php"));
    }
}
