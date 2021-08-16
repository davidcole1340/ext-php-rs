use ext_php_rs::{
    call_user_func, info_table_end, info_table_row, info_table_start,
    php::{
        exceptions::PhpException,
        module::ModuleEntry,
        types::{array::ZendHashTable, callable::Callable},
    },
    prelude::*,
};

#[derive(Debug, Default, ZendObjectHandler)]
struct Human {
    name: String,
    age: i32,
}

#[php_impl]
impl Human {
    const AGE_LIMIT: i32 = 100;

    #[optional(age)]
    #[defaults(age = 10)]
    pub fn __construct(&mut self, name: String, age: i32) {
        self.name = name;
        self.age = age;
    }

    pub fn get_name(&self) -> String {
        self.name.to_string()
    }

    pub fn get_age(&self) -> i32 {
        self.age
    }

    pub fn get_age_limit() -> i32 {
        Self::AGE_LIMIT
    }
}

#[derive(Debug, ZendObjectHandler)]
struct Test {
    a: u32,
    b: u32,
}

#[php_impl]
impl Test {
    const TEST: &'static str = "Hello, world!";

    pub fn __construct(&self) {
        dbg!(self);
        println!("Inside constructor");
    }

    pub fn set(&mut self, a: u32) {
        self.a = a;
        dbg!(self.get());
    }

    pub fn get(&self) -> u32 {
        self.a
    }

    pub fn call(&self, func: Callable) {
        let result = call_user_func!(func);

        if let Ok(r) = result {
            dbg!(r);
        }

        println!("Ready for call!");
    }
}

impl Default for Test {
    fn default() -> Self {
        Self { a: 1, b: 2 }
    }
}

#[php_const]
const SKEL_TEST_CONST: &str = "Test constant";
#[php_const]
const SKEL_TEST_LONG_CONST: i32 = 1234;

#[php_function(optional = "z")]
pub fn skeleton_version(x: ZendHashTable, y: f64, z: Option<f64>) -> String {
    dbg!(x, y, z);
    "Hello".into()
}

#[php_function(optional = "z")]
pub fn skeleton_array(
    arr: ZendHashTable,
    x: i32,
    y: f64,
    z: Option<f64>,
) -> Result<ZendHashTable, String> {
    for (k, x, y) in arr.iter() {
        println!("{:?} {:?} {:?}", k, x, y.string());
    }

    dbg!(x, y, z);

    let mut new = ZendHashTable::new();
    new.insert("Hello", &"World")
        .map_err(|_| "Couldn't insert into hashtable")?;
    Ok(new)
}

#[php_function]
pub fn test_array() -> Vec<i32> {
    vec![1, 2, 3, 4]
}

#[php_function]
pub fn skel_unpack<'a>(mut arr: ZendHashTable) -> Result<ZendHashTable, PhpException<'a>> {
    arr.insert("hello", &"not world");
    Ok(arr)
}

#[no_mangle]
pub extern "C" fn php_module_info(_module: *mut ModuleEntry) {
    info_table_start!();
    info_table_row!("skeleton extension", "enabled");
    info_table_end!();
}

#[php_startup]
pub fn startup() {}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module.info_function(php_module_info)
}
