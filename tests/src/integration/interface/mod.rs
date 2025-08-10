use ext_php_rs::php_interface;
use ext_php_rs::prelude::ModuleBuilder;
use ext_php_rs::types::ZendClassObject;
use ext_php_rs::zend::ce;

#[php_interface]
#[php(extends(ce = ce::throwable, stub = "\\Throwable"))]
#[php(name = "ExtPhpRs\\Interface\\EmptyObjectInterface")]
pub trait EmptyObjectTrait {
    const STRING_CONST: &'static str = "STRING_CONST";

    const USIZE_CONST: u64 = 200;

    fn void();

    fn non_static(&self, data: String) -> String;

    fn ref_to_like_this_class(
        &self,
        data: String,
        other: &ZendClassObject<PhpInterfaceEmptyObjectTrait>,
    ) -> String;

    #[php(defaults(value = 0))]
    fn set_value(&mut self, value: i32);
}

pub fn build_module(builder: ModuleBuilder) -> ModuleBuilder {
    builder.interface::<PhpInterfaceEmptyObjectTrait>()
}

#[cfg(test)]
mod tests {
    #[test]
    fn interface_work() {
        assert!(crate::integration::test::run_php("interface/interface.php"));
    }
}
