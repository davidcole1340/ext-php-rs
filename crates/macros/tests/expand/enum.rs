#[macro_use]
extern crate ext_php_rs_derive;

/// Doc comments for MyEnum.
/// This is a basic enum example.
#[php_enum]
enum MyEnum {
    /// Variant1 of MyEnum.
    /// This variant represents the first case.
    Variant1,
    #[php(name = "Variant_2")]
    Variant2,
    /// Variant3 of MyEnum.
    #[php(change_case = "UPPER_CASE")]
    Variant3,
}

#[php_enum]
#[php(name = "MyIntValuesEnum")]
enum MyEnumWithIntValues {
    #[php(value = 1)]
    Variant1,
    #[php(value = 42)]
    Variant2,
}

#[php_enum]
#[php(change_case = "UPPER_CASE")]
enum MyEnumWithStringValues {
    #[php(value = "foo")]
    Variant1,
    #[php(value = "bar")]
    Variant2,
}
