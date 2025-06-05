use ext_php_rs::{prelude::*, zend::ce};

#[php_class]
#[php(name = "Test\\TestException")]
#[php(extends(ce = ce::exception, stub = "\\Exception"))]
#[derive(Debug)]
pub struct TestException;

#[php_function]
pub fn throw_custom_exception() -> PhpResult<i32> {
    Err(PhpException::from_class::<TestException>(
        "Not good custom!".into(),
    ))
}

#[php_function]
pub fn throw_default_exception() -> PhpResult<i32> {
    Err(PhpException::default("Not good!".into()))
}

pub fn build_module(builder: ModuleBuilder) -> ModuleBuilder {
    builder
        .class::<TestException>()
        .function(wrap_function!(throw_default_exception))
        .function(wrap_function!(throw_custom_exception))
}

#[cfg(test)]
mod tests {
    #[test]
    fn exception_works() {
        assert!(crate::integration::test::run_php("exception/exception.php"));
    }
}
