use ext_php_rs::prelude::*;

#[php_class]
struct TestClass {
    #[prop(rename = "Hello")]
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
    #[getter]
    fn get_test_name(&self) -> String {
        self.c.clone()
    }

    #[setter]
    fn set_test_name(&mut self, c: String) {
        self.c = c;
    }
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
