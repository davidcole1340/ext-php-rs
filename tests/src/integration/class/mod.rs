use ext_php_rs::prelude::*;

#[php_class]
pub struct TestClass {
    string: String,
    number: i32,
    #[php(prop)]
    boolean: bool,
}

#[php_impl]
impl TestClass {
    #[php(getter)]
    pub fn get_string(&self) -> String {
        self.string.to_string()
    }

    #[php(setter)]
    pub fn set_string(&mut self, string: String) {
        self.string = string;
    }

    #[php(getter)]
    pub fn get_number(&self) -> i32 {
        self.number
    }

    #[php(setter)]
    pub fn set_number(&mut self, number: i32) {
        self.number = number;
    }

    pub fn static_call(name: String) -> String {
        format!("Hello {name}")
    }
}

#[php_function]
pub fn test_class(string: String, number: i32) -> TestClass {
    TestClass {
        string,
        number,
        boolean: true,
    }
}

pub fn build_module(builder: ModuleBuilder) -> ModuleBuilder {
    builder
        .class::<TestClass>()
        .function(wrap_function!(test_class))
}

#[cfg(test)]
mod tests {
    #[test]
    fn class_works() {
        assert!(crate::integration::test::run_php("class/class.php"));
    }
}
