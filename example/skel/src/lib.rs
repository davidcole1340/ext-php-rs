mod allocator;

use std::{collections::HashMap, convert::TryFrom, mem::MaybeUninit};

use allocator::PhpAllocator;
use ext_php_rs::{
    call_user_func, info_table_end, info_table_row, info_table_start,
    php::{
        class::ClassEntry,
        exceptions::PhpException,
        module::ModuleEntry,
        types::{array::ZendHashTable, callable::Callable, zval::Zval},
    },
    php_class,
    prelude::*,
};

#[global_allocator]
static GLOBAL: PhpAllocator = PhpAllocator::new();

// #[php_function]
// pub fn hello_world() -> String {
//     let call = Callable::try_from_name("strpos").unwrap();

//     eprintln!("im callin");
//     let val = call.try_call(vec![&"hello world", &"w"]);
//     dbg!(val);
//     "Ok".into()
// }

// #[php_const]
// const SKEL_TEST_CONST: &str = "Test constant";
// #[php_const]
// const SKEL_TEST_LONG_CONST: i32 = 1234;

// #[php_function(optional = "z")]
// pub fn skeleton_version(x: ZendHashTable, y: f64, z: Option<f64>) -> String {
//     dbg!(x, y, z);
//     "Hello".into()
// }

// #[php_function(optional = "z")]
// pub fn skeleton_array(
//     arr: ZendHashTable,
//     x: i32,
//     y: f64,
//     z: Option<f64>,
// ) -> Result<ZendHashTable, String> {
//     for (k, x, y) in arr.iter() {
//         println!("{:?} {:?} {:?}", k, x, y.string());
//     }

//     dbg!(x, y, z);

//     let mut new = ZendHashTable::new();
//     new.insert("Hello", &"World")
//         .map_err(|_| "Couldn't insert into hashtable")?;
//     Ok(new)
// }

// #[php_function(optional = "i", defaults(i = 5))]
// pub fn test_array(i: i32, b: Option<i32>) -> Vec<i32> {
//     dbg!(i, b);
//     vec![1, 2, 3, 4]
// }

// #[php_function(optional = "offset", defaults(offset = 0))]
// pub fn rust_strpos(haystack: &str, needle: &str, offset: i64) -> Option<usize> {
//     let haystack = haystack.chars().skip(offset as usize).collect::<String>();
//     haystack.find(needle)
// }

// #[php_function]
// pub fn example_exception() -> Result<i32, &'static str> {
//     Err("Bad here")
// }

// #[php_function]
// pub fn skel_unpack<'a>(
//     mut arr: HashMap<String, String>,
// ) -> Result<HashMap<String, String>, PhpException<'a>> {
//     arr.insert("hello".into(), "not world".into());
//     Ok(arr)
// }

// #[php_function]
// pub fn test_extern() -> i32 {
//     // let y = unsafe { strpos("hello", "e", None) };
//     // dbg!(y);
//     // let x = unsafe { test_func() };
//     // dbg!(x.try_call(vec![]));
//     0
// }

// #[php_function]
// pub fn test_lifetimes<'a>() -> ZendHashTable<'a> {
//     ZendHashTable::try_from(&HashMap::<String, String>::new()).unwrap()
// }

// #[php_function]
// pub fn test_str(input: &str) -> &str {
//     input
// }

// #[no_mangle]
// pub extern "C" fn php_module_info(_module: *mut ModuleEntry) {
//     info_table_start!();
//     info_table_row!("skeleton extension", "enabled");
//     info_table_end!();
// }

#[php_class(name = "Redis\\Exception\\RedisClientException")]
#[extends(ClassEntry::exception())]
#[derive(Default)]
struct RedisException;

#[php_function]
pub fn test_exception() -> Result<i32, PhpException<'static>> {
    Err(PhpException::from_class::<RedisException>(
        "Hello world".into(),
    ))
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    // module.info_function(php_module_info)
    module
}
