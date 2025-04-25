#![allow(clippy::unused_self)]
use ext_php_rs::{prelude::*, types::Zval};
use std::collections::HashMap;

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
            self.0 = val.long().unwrap();
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

pub fn build_module(builder: ModuleBuilder) -> ModuleBuilder {
    builder.class::<MagicMethod>()
}

#[cfg(test)]
mod tests {
    #[test]
    fn magic_method() {
        assert!(crate::integration::test::run_php(
            "magic_method/magic_method.php"
        ));
    }
}
