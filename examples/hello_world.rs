use ext_php_rs::{
    args::Arg,
    builders::{ClassBuilder, FunctionBuilder, ModuleStartup},
    class::{ClassMetadata, ConstructorMeta, ConstructorResult, RegisteredClass},
    constant::IntoConst,
    convert::IntoZval,
    flags::DataType,
    internal::class::{PhpClassImpl, PhpClassImplCollector},
    prelude::*,
    types::Zval,
    zend::ExecuteData,
};

#[php_class]
pub struct TestClass {
    #[prop]
    a: i32,
    #[prop]
    b: i32,
}

#[php_impl]
impl TestClass {
    #[rename("NEW_CONSTANT_NAME")]
    pub const SOME_CONSTANT: i32 = 5;
    pub const SOME_OTHER_STR: &'static str = "Hello, world!";

    pub fn __construct(a: i32, b: i32) -> Self {
        Self {
            a: a + 10,
            b: b + 10,
        }
    }

    #[optional(test)]
    #[defaults(a = 5, test = 100)]
    pub fn test_camel_case(&self, a: i32, test: i32) {
        println!("a: {} test: {}", a, test);
    }

    fn x(&self) -> i32 {
        5
    }
}

#[php_function]
pub fn new_class() -> TestClass {
    TestClass { a: 1, b: 2 }
}

#[php_function]
pub fn hello_world() -> &'static str {
    "Hello, world!"
}

pub const HELLO_WORLD: i32 = 100;

#[php_extern]
extern "C" {
    fn phpinfo() -> bool;
}

#[derive(Debug, ZvalConvert)]
pub struct TestZvalConvert<'a> {
    a: i32,
    b: i32,
    c: &'a str,
}

#[php_function]
pub fn get_zval_convert<'a>(z: TestZvalConvert<'a>) -> i32 {
    dbg!(z);
    5
}

fn startup(ty: i32, mod_num: i32) -> i32 {
    5.register_constant("SOME_CONST", mod_num).unwrap();
    0
}

#[php_module(startup = "startup")]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .class::<TestClass>()
        .function(wrap_function!(hello_world))
        .function(wrap_function!(new_class))
        .function(wrap_function!(get_zval_convert))
        .constant(wrap_constant!(HELLO_WORLD))
        .constant(("CONST_NAME", HELLO_WORLD))
}
