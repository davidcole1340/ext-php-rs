use ext_php_rs::prelude::*;
use std::collections::HashMap;

#[php_interface]
#[php(name = "ExtPhpRs\\Interface\\EmptyObjectInterface")]
pub trait EmptyObjectInterface { }

pub fn build_module(builder: ModuleBuilder) -> ModuleBuilder {
    builder.interface::<PhpInterfaceEmptyObjectInterface>()
}

#[cfg(test)]
mod tests {
    #[test]
    fn interface_work() {
        assert!(crate::integration::test::run_php("interface/interface.php"));
    }
}
