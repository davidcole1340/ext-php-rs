use std::collections::HashMap;

use ext_php_rs::{
    call_user_func, info_table_end, info_table_row, info_table_start, parse_args,
    php::{
        args::{Arg, ArgParser},
        class::{ClassBuilder, ClassEntry},
        constants::IntoConst,
        enums::DataType,
        exceptions::throw,
        execution_data::ExecutionData,
        flags::MethodFlags,
        function::FunctionBuilder,
        module::{ModuleBuilder, ModuleEntry},
        types::{
            array::ZendHashTable, long::ZendLong, object::ZendClassObject, string::ZendString,
            zval::Zval,
        },
    },
    ZendObjectHandler,
};

#[no_mangle]
pub extern "C" fn php_module_info(_module: *mut ModuleEntry) {
    info_table_start!();
    info_table_row!("skeleton extension", "enabled");
    info_table_end!();
}

#[derive(Debug, ZendObjectHandler)]
struct Test {
    a: u32,
    b: u32,
}

#[derive(Debug, Default, ZendObjectHandler)]
struct AnotherTest {
    x: u32,
}

impl Test {
    pub extern "C" fn constructor(execute_data: &mut ExecutionData, _retval: &mut Zval) {
        println!("Inside constructor");
        let x = ZendClassObject::<Test>::get(execute_data);
        if x.is_none() {
            eprintln!("Object was none");
        } else {
            // let obj = x.unwrap();
            println!("Object not none");
        }
    }

    pub extern "C" fn set(execute_data: &mut ExecutionData, retval: &mut Zval) {
        let x = ZendClassObject::<Test>::get(execute_data).unwrap();
        x.a = 100;
    }

    pub extern "C" fn get(execute_data: &mut ExecutionData, _retval: &mut Zval) {
        let x = ZendClassObject::<Test>::get(execute_data).unwrap();
        dbg!(x.a);
    }

    pub extern "C" fn call(execute_data: &mut ExecutionData, _retval: &mut Zval) {
        let mut _fn = Arg::new("fn", DataType::Callable);
        let result = ArgParser::new(execute_data).arg(&mut _fn).parse();

        if result.is_err() {
            return;
        }

        let result = call_user_func!(_fn, "Hello", 5);

        if let Some(r) = result {
            println!("{}", r.string().unwrap());
        }

        println!("Ready for call!");
    }
}

impl Default for Test {
    fn default() -> Self {
        Self { a: 1, b: 2 }
    }
}

#[no_mangle]
pub extern "C" fn module_init(_type: i32, module_number: i32) -> i32 {
    // object_handlers_init!(Test);

    ClassBuilder::new("TestClass")
        .method(
            FunctionBuilder::constructor(Test::constructor).build(),
            MethodFlags::Public,
        )
        .method(
            FunctionBuilder::new("set", Test::set).build(),
            MethodFlags::Public,
        )
        .method(
            FunctionBuilder::new("get", Test::get).build(),
            MethodFlags::Public,
        )
        .method(
            FunctionBuilder::new("call", Test::call)
                .arg(Arg::new("fn", DataType::Callable))
                .build(),
            MethodFlags::Public,
        )
        // .property("value", "world", PropertyFlags::Protected)
        .constant("TEST", "Hello world")
        .object_override::<Test>()
        .build();

    "Test constant".register_constant("SKEL_TEST_CONST", module_number);
    1234.register_constant("SKEL_TEST_LONG_CONST", module_number);

    0
}

#[no_mangle]
pub extern "C" fn get_module() -> *mut ext_php_rs::php::module::ModuleEntry {
    let funct = FunctionBuilder::new("skeleton_version", skeleton_version)
        .arg(Arg::new("a", DataType::Array))
        .arg(Arg::new("b", DataType::Double))
        .not_required()
        .arg(Arg::new("c", DataType::Double))
        .returns(DataType::String, false, false)
        .build();

    let array = FunctionBuilder::new("skel_array", skeleton_array)
        .arg(Arg::new("arr", DataType::Array))
        .build();

    let t = FunctionBuilder::new("test_array", test_array)
        .returns(DataType::Array, false, false)
        .build();

    let iter = FunctionBuilder::new("skel_unpack", skel_unpack)
        .arg(Arg::new("arr", DataType::String))
        .returns(DataType::String, false, false)
        .build();

    ModuleBuilder::new("ext-skel", "0.1.0")
        .info_function(php_module_info)
        .startup_function(module_init)
        .function(funct)
        .function(array)
        .function(t)
        .function(iter)
        .build()
        .into_raw()
}

#[no_mangle]
pub extern "C" fn skeleton_version(execute_data: &mut ExecutionData, retval: &mut Zval) {
    let mut x = Arg::new("x", DataType::Array);
    let mut y = Arg::new("y", DataType::Double);
    let mut z = Arg::new("z", DataType::Double);

    parse_args!(execute_data, x, y; z);
    dbg!(x);
    retval.set_string("Hello");
}

#[no_mangle]
pub extern "C" fn skeleton_array(execute_data: &mut ExecutionData, _retval: &mut Zval) {
    let mut arr = Arg::new("arr", DataType::Array);
    let mut x = Arg::new("x", DataType::Long);
    let mut y = Arg::new("y", DataType::Double);
    let mut z = Arg::new("z", DataType::Double);

    parse_args!(execute_data, arr, x, y; z);

    let ht: ZendHashTable = arr.val().unwrap();

    for (k, x, y) in ht.into_iter() {
        println!("{:?} {:?} {:?}", k, x, y.string());
    }

    let mut new = ZendHashTable::new();
    new.insert("Hello", "WOrld");
    let _ = _retval.set_array(new);
}

#[no_mangle]
pub extern "C" fn test_array(_execute_data: &mut ExecutionData, retval: &mut Zval) {
    retval.set_array(vec![1, 2, 3, 4]);
}

pub extern "C" fn skel_unpack(execute_data: &mut ExecutionData, retval: &mut Zval) {
    let mut packed = Arg::new("arr", DataType::String);
    parse_args!(execute_data, packed);

    let zv = packed.zval().unwrap();
    let val = unsafe { zv.binary::<f32>() };
    dbg!(val);
    let v = vec![1i32, 2, 4, 8];
    retval.set_binary(v);
}
