#[macro_use]
extern crate ext_php_rs_derive;

use ext_php_rs::types::ZendClassObject;
use ext_php_rs::php_interface;
use ext_php_rs::zend::ce;

#[php_interface]
#[php(extends(ce = ce::throwable, stub = "\\Throwable"))]
#[php(name = "ExtPhpRs\\Interface\\EmptyObjectInterface")]
pub trait EmptyObjectTrait
{
    const HELLO: &'static str = "HELLO";

    const ONE: u64 = 12;

    fn void();

    fn non_static(&self, data: String) -> String;

    fn ref_to_like_this_class(
        &self,
        data: String,
        other: &ZendClassObject<PhpInterfaceEmptyObjectTrait>
    ) -> String;
}
