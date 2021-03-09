use php_rs::{
    info_table_end, info_table_row, info_table_start,
    php::{
        args::Arg,
        enums::DataType,
        function::{ExecutionData, FunctionBuilder, Zval},
        module::{ModuleBuilder, ModuleEntry},
    },
};

#[no_mangle]
pub extern "C" fn php_module_info(_module: *mut ModuleEntry) {
    info_table_start!();
    info_table_row!("skeleton extension", "enabled");
    info_table_end!();
}

#[no_mangle]
pub extern "C" fn get_module() -> *mut php_rs::php::module::ModuleEntry {
    let funct = FunctionBuilder::new("skeleton_version", skeleton_version)
        .arg(Arg::new("test", DataType::String))
        .returns(DataType::Long, false, false)
        .build();

    ModuleBuilder::new("ext-skel", "0.1.0")
        .info_function(php_module_info)
        .function(funct)
        .build()
        .into_raw()
}

#[no_mangle]
pub extern "C" fn skeleton_version(_execute_data: *mut ExecutionData, _retval: *mut Zval) {
    panic!("it worked?");
}
