#![cfg_attr(windows, feature(abi_vectorcall))]
use ext_php_rs::{
    binary::Binary,
    boxed::ZBox,
    prelude::*,
    types::{ArrayKey, ZendHashTable, ZendObject, Zval},
    zend::ProcessGlobals,
};
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

// GLOBALS
#[php_function]
pub fn test_globals_http_get() -> ZBox<ZendHashTable> {
    ProcessGlobals::get().http_get_vars().to_owned()
}

#[php_function]
pub fn test_globals_http_post() -> ZBox<ZendHashTable> {
    ProcessGlobals::get().http_post_vars().to_owned()
}

#[php_function]
pub fn test_globals_http_cookie() -> ZBox<ZendHashTable> {
    ProcessGlobals::get().http_cookie_vars().to_owned()
}

#[php_function]
pub fn test_globals_http_server() -> ZBox<ZendHashTable> {
    ProcessGlobals::get().http_server_vars().unwrap().to_owned()
}

#[php_function]
pub fn test_globals_http_request() -> ZBox<ZendHashTable> {
    ProcessGlobals::get()
        .http_request_vars()
        .unwrap()
        .to_owned()
}

#[php_function]
pub fn test_globals_http_files() -> ZBox<ZendHashTable> {
    ProcessGlobals::get().http_files_vars().to_owned()
}

#[php_function]
pub fn test_closure() -> Closure {
    Closure::wrap(Box::new(|a| a) as Box<dyn Fn(String) -> String>)
}

#[php_function]
pub fn test_closure_once(a: String) -> Closure {
    Closure::wrap_once(Box::new(move || a) as Box<dyn FnOnce() -> String>)
}

#[php_function]
pub fn test_callable(call: ZendCallable, a: String) -> Zval {
    call.try_call(vec![&a]).expect("Failed to call function")
}

#[php_function]
pub fn iter_next(ht: &ZendHashTable) -> Vec<Zval> {
    ht.iter()
        .flat_map(|(k, v)| [key_to_zval(k), v.shallow_clone()])
        .collect()
}

#[php_function]
pub fn iter_back(ht: &ZendHashTable) -> Vec<Zval> {
    ht.iter()
        .rev()
        .flat_map(|(k, v)| [key_to_zval(k), v.shallow_clone()])
        .collect()
}

#[php_function]
pub fn iter_next_back(ht: &ZendHashTable, modulus: usize) -> Vec<Option<Zval>> {
    let mut result = Vec::with_capacity(ht.len());
    let mut iter = ht.iter();

    for i in 0..ht.len() + modulus {
        let entry = if i % modulus == 0 {
            iter.next_back()
        } else {
            iter.next()
        };

        if let Some((k, v)) = entry {
            result.push(Some(key_to_zval(k)));
            result.push(Some(v.shallow_clone()));
        } else {
            result.push(None);
        }
    }

    result
}

fn key_to_zval(key: ArrayKey) -> Zval {
    match key {
        ArrayKey::String(s) => {
            let mut zval = Zval::new();
            let _ = zval.set_string(s.as_str(), false);
            zval
        }
        ArrayKey::Long(l) => {
            let mut zval = Zval::new();
            zval.set_long(l);
            zval
        }
    }
}

// Rust type &[&Zval] must be converted because to Vec<Zval> because of
// lifetime hell.
#[php_function]
pub fn test_variadic_args(params: &[&Zval]) -> Vec<Zval> {
    params.iter().map(|x| x.shallow_clone()).collect()
}

#[php_function]
pub fn test_variadic_add_required(number: u32, numbers: &[&Zval]) -> u32 {
    number
        + numbers
            .iter()
            .map(|x| x.long().unwrap() as u32)
            .sum::<u32>()
}

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

#[php_class]
pub struct MagicMethod(i64);

#[php_impl]
impl MagicMethod {
    pub fn __construct() -> Self {
        Self(0)
    }

    pub fn __destruct(&self) {}

    pub fn __call(&self, name: String, _arguments: HashMap<String, &Zval>) -> Zval {
        let mut z = Zval::new();
        if name == "callMagicMethod" {
            let s = "Hello".to_string();

            let _ = z.set_string(s.as_str(), false);
            z
        } else {
            z.set_null();
            z
        }
    }

    pub fn __call_static(name: String, arguments: HashMap<String, &Zval>) -> Zval {
        let mut zval = Zval::new();
        if name == "callStaticSomeMagic" {
            let concat_args = format!(
                "Hello from static call {}",
                arguments
                    .iter()
                    .filter(|(_, v)| v.is_long())
                    .map(|(_, s)| s.long().unwrap())
                    .collect::<Vec<_>>()
                    .iter()
                    .sum::<i64>()
            );

            let _ = zval.set_string(&concat_args, false);
            zval
        } else {
            zval.set_null();
            zval
        }
    }

    pub fn __get(&self, name: String) -> Zval {
        let mut v = Zval::new();
        v.set_null();
        if name == "count" {
            v.set_long(self.0);
        }

        v
    }

    pub fn __set(&mut self, prop_name: String, val: &Zval) {
        if val.is_long() && prop_name == "count" {
            self.0 = val.long().unwrap()
        }
    }

    pub fn __isset(&self, prop_name: String) -> bool {
        "count" == prop_name
    }

    pub fn __unset(&mut self, prop_name: String) {
        if prop_name == "count" {
            self.0 = 0;
        }
    }

    pub fn __to_string(&self) -> String {
        self.0.to_string()
    }

    pub fn __invoke(&self, n: i64) -> i64 {
        self.0 + n
    }

    pub fn __debug_info(&self) -> HashMap<String, Zval> {
        let mut h: HashMap<String, Zval> = HashMap::new();
        let mut z = Zval::new();
        z.set_long(self.0);
        h.insert("count".to_string(), z);

        h
    }
}

#[php_module]
pub fn build_module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .class::<TestClass>()
        .class::<MagicMethod>()
        .function(wrap_function!(test_str))
        .function(wrap_function!(test_string))
        .function(wrap_function!(test_bool))
        .function(wrap_function!(test_number_signed))
        .function(wrap_function!(test_number_unsigned))
        .function(wrap_function!(test_number_float))
        .function(wrap_function!(test_array))
        .function(wrap_function!(test_array_assoc))
        .function(wrap_function!(test_binary))
        .function(wrap_function!(test_nullable))
        .function(wrap_function!(test_object))
        .function(wrap_function!(test_globals_http_get))
        .function(wrap_function!(test_globals_http_post))
        .function(wrap_function!(test_globals_http_cookie))
        .function(wrap_function!(test_globals_http_server))
        .function(wrap_function!(test_globals_http_request))
        .function(wrap_function!(test_globals_http_files))
        .function(wrap_function!(test_closure))
        .function(wrap_function!(test_closure_once))
        .function(wrap_function!(test_callable))
        .function(wrap_function!(iter_next))
        .function(wrap_function!(iter_back))
        .function(wrap_function!(iter_next_back))
        .function(wrap_function!(test_class))
        .function(wrap_function!(test_variadic_args))
        .function(wrap_function!(test_variadic_add_required))
}

#[cfg(test)]
mod integration {
    use std::env;

    use std::process::Command;
    use std::sync::Once;

    static BUILD: Once = Once::new();

    fn setup() {
        BUILD.call_once(|| {
            assert!(Command::new("cargo")
                .arg("build")
                .output()
                .expect("failed to build extension")
                .status
                .success());
        });
    }

    pub fn run_php(file: &str) -> bool {
        setup();
        let mut path = env::current_dir().expect("Could not get cwd");
        path.pop();
        path.push("target");
        path.push("debug");
        path.push(if std::env::consts::DLL_EXTENSION == "dll" {
            "tests"
        } else {
            "libtests"
        });
        path.set_extension(std::env::consts::DLL_EXTENSION);
        let output = Command::new("php")
            .arg(format!("-dextension={}", path.to_str().unwrap()))
            .arg("-dassert.active=1")
            .arg("-dassert.exception=1")
            .arg("-dzend.assertions=1")
            .arg(format!("src/integration/{}", file))
            .output()
            .expect("failed to run php file");
        if output.status.success() {
            true
        } else {
            panic!(
                "
                status: {}
                stdout: {}
                stderr: {}
                ",
                output.status,
                String::from_utf8(output.stdout).unwrap(),
                String::from_utf8(output.stderr).unwrap()
            );
        }
    }

    mod array;
    mod binary;
    mod bool;
    mod callable;
    mod class;
    mod closure;
    mod globals;
    mod iterator;
    mod magic_method;
    mod nullable;
    mod number;
    mod object;
    mod string;
    mod types;
    mod variadic_args;
}
