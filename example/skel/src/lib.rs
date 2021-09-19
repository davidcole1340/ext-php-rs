mod allocator;

use ext_php_rs::{
    parse_args,
    php::{
        args::Arg,
        enums::DataType,
        exceptions::PhpException,
        execution_data::ExecutionData,
        function::FunctionBuilder,
        types::{
            callable::Callable,
            closure::Closure,
            object::{ClassObject, ClassRef},
            zval::Zval,
        },
    },
    php_class,
    prelude::*,
};

#[php_class]
#[property(test = 0)]
#[property(another = "Hello world")]
#[derive(Default, Debug, Clone)]
pub struct Test {
    pub test: String,
}

#[php_function]
pub fn take_test(test: &Test) -> String {
    test.test.clone()
}

#[php_class]
#[derive(Default)]
struct PhpFuture {
    then: Option<Callable<'static>>,
}

#[php_impl]
impl PhpFuture {
    pub fn then(&mut self, then: Callable<'static>) {
        self.then = Some(then);
    }

    pub fn now(&self) -> Result<(), PhpException> {
        if let Some(then) = &self.then {
            then.try_call(vec![&"Hello"]).unwrap();
            Ok(())
        } else {
            Err(PhpException::default("No `then`".into()))
        }
    }

    pub fn obj(&self) -> ClassObject<Test> {
        ClassObject::new(Test {
            test: "Hello world from class entry :)".into(),
        })
    }

    pub fn return_self(&self) -> ClassRef<PhpFuture> {
        ClassRef::from_ref(self).unwrap()
    }
}

#[php_impl]
impl Test {
    pub fn set_str(&mut self, str: String) {
        self.test = str;
    }

    pub fn get_str(&self) -> String {
        self.test.clone()
    }
}

#[php_function]
pub fn get_closure() -> Closure {
    let mut x = 100;
    Closure::wrap(Box::new(move || {
        x += 5;
        format!("x: {}", x)
    }) as Box<dyn FnMut() -> String>)
}

#[php_function]
pub fn fn_once() -> Closure {
    let x = "Hello".to_string();
    Closure::wrap_once(Box::new(move || {
        println!("val here: {}", &x);
        x
    }) as Box<dyn FnOnce() -> String>)
}

#[php_function]
pub fn closure_get_string() -> Closure {
    // Return a closure which takes two integers and returns a string
    Closure::wrap(Box::new(|a, b| format!("A: {} B: {}", a, b)) as Box<dyn Fn(i32, i32) -> String>)
}

#[php_function]
pub fn closure_count() -> Closure {
    let mut count = 0i32;

    Closure::wrap(Box::new(move |a: i32| {
        count += a;
        count
    }) as Box<dyn FnMut(i32) -> i32>)
}

// #[php_function]
// pub fn test_zval(t: Zval) {
// let mut z = Zval::new();
// z.set_long(5);
// z.set_double(100.5);
// z.set_bool(false);
// z.set_null();
// z.set_string("Hello world", false).unwrap();
// z.set_array(ZendHashTable::try_from(vec![1, 2, 3, 4, 5]).unwrap());
// drop(dbg!(z));

// let mut x = ZendHashTable::new();
// dbg!(x.insert("test", "Hello world"));
// dbg!(x.insert("test", 1234));
// dbg!(x.insert("test", "ok test"));
// }

extern "C" fn test_zval(ex: &mut ExecutionData, retval: &mut Zval) {
    let mut a = Arg::new("test", DataType::Array);

    parse_args!(ex, a);
    let zv = a.zval().unwrap();
    let arr = zv.array().unwrap();
    let mut new = arr.clone();

    new.insert("hello", "a new fvalue");
    dbg!(&arr);
    dbg!(&new);
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module.function(
        FunctionBuilder::new("test_zval", test_zval)
            .arg(Arg::new("test", DataType::Array))
            .build()
            .unwrap(),
    )
}
