use ext_php_rs::prelude::*;

#[php_function]
pub fn test_closure() -> Closure {
    Closure::wrap(Box::new(|a| a) as Box<dyn Fn(String) -> String>)
}

#[php_function]
pub fn test_closure_once(a: String) -> Closure {
    Closure::wrap_once(Box::new(move || a) as Box<dyn FnOnce() -> String>)
}

pub fn build_module(builder: ModuleBuilder) -> ModuleBuilder {
    builder
        .function(wrap_function!(test_closure))
        .function(wrap_function!(test_closure_once))
}

#[cfg(test)]
mod tests {
    #[test]
    fn closure_works() {
        assert!(crate::integration::test::run_php("closure/closure.php"));
    }
}
