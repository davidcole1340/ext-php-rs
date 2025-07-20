use std::collections::HashMap;

use ext_php_rs::types::ZendClassObject;
use ext_php_rs::php_interface;
use ext_php_rs::{php_module, prelude::ModuleBuilder};
use ext_php_rs::zend::ce;

#[php_interface]
#[php(extends(ce = ce::throwable, stub = "\\Throwable"))]
#[php(name = "ExtPhpRs\\Interface\\EmptyObjectInterface")]
pub trait EmptyObjectTrait
{
    const HELLO: &'static str = "HELLO";

    fn void();

    fn non_static(&self, data: String) -> String;

    fn ref_to_like_this_class(
        &self,
        data: String,
        other: &ZendClassObject<PhpInterfaceEmptyObjectTrait>
    ) -> String;
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .interface::<PhpInterfaceEmptyObjectTrait>()
}

