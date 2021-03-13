use std::{os::raw::c_int, thread, time::Duration};

use php_rs::{
    info_table_end, info_table_row, info_table_start,
    php::{
        args::{Arg, ArgParser},
        class::{ClassBuilder, ClassEntry},
        enums::DataType,
        execution_data::ExecutionData,
        flags::{ClassFlags, MethodFlags, PropertyFlags},
        function::FunctionBuilder,
        module::{ModuleBuilder, ModuleEntry},
        types::{
            array::ZendHashTable,
            long::ZendLong,
            zval::{SetZval, Zval},
        },
    },
};

#[no_mangle]
pub extern "C" fn php_module_info(_module: *mut ModuleEntry) {
    info_table_start!();
    info_table_row!("skeleton extension", "enabled");
    info_table_end!();
}

#[no_mangle]
pub extern "C" fn module_init(_type: i32, _module_number: i32) -> i32 {
    let func = FunctionBuilder::new("test", skeleton_version)
        .returns(DataType::Long, false, false)
        .build();

    ClassBuilder::new("TestClass")
        .function(func, MethodFlags::Public)
        .property("hello", "doc", 10, PropertyFlags::Public)
        .build();

    0
}

#[no_mangle]
pub extern "C" fn get_module() -> *mut php_rs::php::module::ModuleEntry {
    let funct = FunctionBuilder::new("skeleton_version", skeleton_version)
        .arg(Arg::new("test", DataType::Long))
        .returns(DataType::Long, false, false)
        .build();

    let array = FunctionBuilder::new("skel_array", skeleton_array)
        .arg(Arg::new("arr", DataType::Array))
        .build();

    ModuleBuilder::new("ext-skel", "0.1.0")
        .info_function(php_module_info)
        .startup_function(module_init)
        .function(funct)
        .function(array)
        .build()
        .into_raw()
}

#[no_mangle]
pub extern "C" fn skeleton_version(execute_data: *mut ExecutionData, mut _retval: *mut Zval) {
    let mut x = Arg::new("x", DataType::Long);
    let mut y = Arg::new("y", DataType::Double);
    let mut z = Arg::new("z", DataType::Double);

    let result = ArgParser::new(execute_data)
        .arg(&mut x)
        .arg(&mut y)
        .not_required()
        .arg(&mut z)
        .parse();

    if result.is_err() {
        return;
    }

    let result = format!(
        "x: {}, y: {}, z: {}",
        x.val::<ZendLong>().unwrap_or_default(),
        y.val::<f64>().unwrap_or_default(),
        z.val::<f64>().unwrap_or_default()
    );

    _retval.set_string(result).unwrap();
}

#[no_mangle]
pub extern "C" fn skeleton_array(execute_data: *mut ExecutionData, mut _retval: *mut Zval) {
    let mut arr = Arg::new("arr", DataType::Array);

    let result = ArgParser::new(execute_data).arg(&mut arr).parse();
    if result.is_err() {
        return;
    }

    let ht: ZendHashTable = arr.val().unwrap();

    for (k, x, y) in ht {
        println!("{:?} {:?} {:?}", k, x, y.string());
    }

    let mut new = ZendHashTable::new();
    new.insert("Hello", "WOrld");
    let _ = _retval.set_array(new);
}
