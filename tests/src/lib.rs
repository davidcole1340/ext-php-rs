use ext_php_rs::{binary::Binary, prelude::*, types::ZendObject, types::Zval};
use std::collections::HashMap;

#[php_function]
pub fn test_str(a: &str) -> &str {
    a
}

#[php_function]
pub fn test_string(a: String) -> String {
    a
}

#[php_function]
pub fn test_bool(a: bool) -> bool {
    a
}

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

#[php_function]
pub fn test_array(a: Vec<String>) -> Vec<String> {
    a
}

#[php_function]
pub fn test_array_assoc(a: HashMap<String, String>) -> HashMap<String, String> {
    a
}

#[php_function]
pub fn test_binary(a: Binary<u32>) -> Binary<u32> {
    a
}

#[php_function]
pub fn test_nullable(a: Option<String>) -> Option<String> {
    a
}

#[php_function]
pub fn test_object(a: &mut ZendObject) -> &mut ZendObject {
    a
}

#[php_function]
pub fn test_closure() -> Closure {
    Closure::wrap(Box::new(|a| a) as Box<dyn Fn(String) -> String>)
}

#[php_function]
pub fn test_closure_once(a: String) -> Closure {
    let example = a.clone();
    Closure::wrap_once(Box::new(move || example) as Box<dyn FnOnce() -> String>)
}

#[php_function]
pub fn test_callable(call: ZendCallable, a: String) -> Zval {
    call.try_call(vec![&a]).expect("Failed to call function")
}

#[php_class]
pub struct TestClass {
    foo: String,
    bar: String,
}

#[php_impl]
impl TestClass {
    pub fn test(&self) -> String {
        format!("{} {}", self.foo, self.bar)
    }
}

#[php_function]
pub fn test_class(foo: String, bar: String) -> TestClass {
    TestClass { foo, bar }
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
}

#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::sync::Once;

    static BUILD: Once = Once::new();

    #[cfg(target_os = "linux")]
    const EXTENSION: &str = "so";
    #[cfg(target_os = "macos")]
    const EXTENSION: &str = "dylib";
    #[cfg(target_os = "windows")]
    const EXTENSION: &str = "dll";

    fn setup() {
        BUILD.call_once(|| {
            Command::new("cargo")
                .arg("build")
                .output()
                .expect("failed to execute process")
                .status
                .success();
        });
    }

    fn run_php(file: &str) -> String {
        setup();
        String::from_utf8(
            Command::new("php")
                .arg(format!(
                    "-dextension=../target/debug/libtests.{}",
                    EXTENSION
                ))
                .arg(format!("integration/{}", file))
                .output()
                .expect("failed to run php file")
                .stdout,
        )
        .unwrap()
    }

    #[test]
    fn str_works() {
        assert_eq!(run_php("str.php"), "str works");
    }

    #[test]
    fn string_works() {
        assert_eq!(run_php("string.php"), "string works");
    }

    #[test]
    fn bool_works() {
        assert_eq!(run_php("bool.php"), "true false");
    }

    #[test]
    fn number_signed_works() {
        assert_eq!(run_php("number_signed.php"), "-12 0 12");
    }

    #[test]
    fn number_unsigned_works() {
        assert_eq!(run_php("number_unsigned.php"), "0 12 invalid");
    }

    #[test]
    fn number_float_works() {
        let output = run_php("number_float.php");
        let floats: Vec<f32> = output
            .split_whitespace()
            .map(|a| a.parse::<f32>().unwrap())
            .collect();
        assert_eq!(floats, vec![-1.2, 0.0, 1.2]);
    }

    #[test]
    fn array_works() {
        assert_eq!(run_php("array.php"), "a b c");
    }

    #[test]
    fn array_assoc_works() {
        let output = run_php("array_assoc.php");
        assert_eq!(output.contains("first=1"), true);
        assert_eq!(output.contains("second=2"), true);
        assert_eq!(output.contains("third=3"), true);
    }

    #[test]
    fn binary_works() {
        assert_eq!(run_php("binary.php"), "1 2 3 4 5");
    }

    #[test]
    fn nullable_works() {
        assert_eq!(run_php("nullable.php"), "null not_null");
    }

    #[test]
    fn object_works() {
        let output = run_php("object.php");
        assert_eq!(output.contains("first=1"), true);
        assert_eq!(output.contains("second=2"), true);
        assert_eq!(output.contains("third=3"), true);
    }

    #[test]
    fn closure_works() {
        assert_eq!(run_php("closure.php"), "closure works");
    }

    #[test]
    fn closure_once_works() {
        assert_eq!(run_php("closure_once.php"), "closure works once");
    }

    #[test]
    fn callable_works() {
        assert_eq!(run_php("callable.php"), "callable works");
    }

    #[test]
    fn class_works() {
        assert_eq!(run_php("class.php"), "class works");
    }
}
