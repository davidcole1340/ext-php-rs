use ext_php_rs::{prelude::*, types::Zval};

#[php_function]
pub fn test_callable(call: ZendCallable, a: String) -> Zval {
    call.try_call(vec![&a]).expect("Failed to call function")
}

pub fn build_module(builder: ModuleBuilder) -> ModuleBuilder {
    builder.function(wrap_function!(test_callable))
}

#[cfg(test)]
mod tests {
    #[test]
    fn callable_works() {
        assert!(crate::integration::test::run_php("callable/callable.php"));
    }
}
