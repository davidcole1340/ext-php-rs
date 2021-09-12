use ext_php_rs::prelude::*;

#[php_class]
struct TestClass {
    #[prop(flags = PropertyFlags::Private, rename = "Hello")]
    a: i32,
    #[prop]
    b: i64,
    #[prop]
    c: String,
}

impl Default for TestClass {
    fn default() -> Self {
        Self {
            a: 100,
            b: 123,
            c: "Hello, world!".into(),
        }
    }
}

#[php_impl]
impl TestClass {
    fn set_c(&mut self, c: String) {
        self.c = c;
    }
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
