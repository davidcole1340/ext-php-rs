use ext_php_rs::{prelude::*, types::ZendObject};

#[php_function]
pub fn test_object(a: &mut ZendObject) -> &mut ZendObject {
    a
}

pub fn build_module(builder: ModuleBuilder) -> ModuleBuilder {
    builder.function(wrap_function!(test_object))
}

#[cfg(test)]
mod tests {
    #[test]
    fn object_works() {
        assert!(crate::integration::test::run_php("object/object.php"));
    }
}
