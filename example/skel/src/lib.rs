use php_rs::{
    info_table_end, info_table_row, info_table_start,
    php::module::{ModuleBuilder, ModuleEntry},
};

#[no_mangle]
pub extern "C" fn php_module_info(_module: *mut ModuleEntry) {
    info_table_start!();
    info_table_row!("skeleton extension", "enabled");
    info_table_end!();
}

#[no_mangle]
pub extern "C" fn get_module() -> *mut php_rs::php::module::ModuleEntry {
    ModuleBuilder::new("ext-skel", "0.1.0")
        .info_function(php_module_info)
        .build()
        .into_raw()
}
