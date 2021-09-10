mod allocator;

use std::{collections::HashMap, iter::FromIterator, sync::Mutex};

use allocator::PhpAllocator;
use ext_php_rs::{
    parse_args,
    php::{
        args::Arg,
        class::ClassBuilder,
        enums::DataType,
        exceptions::PhpException,
        execution_data::ExecutionData,
        flags::{MethodFlags, PropertyFlags},
        function::FunctionBuilder,
        types::{
            array::OwnedHashTable,
            callable::Callable,
            closure::Closure,
            object::{ClassMetadata, ClassObject, ClassRef, Prop, RegisteredClass},
            zval::Zval,
        },
    },
    php_class,
    prelude::*,
};

struct TestClass {
    a: i32,
    b: i64,
    c: String,
}

pub extern "C" fn test_class_change(ex: &mut ExecutionData, retval: &mut Zval) {
    let mut new = Arg::new("c", DataType::String);

    parse_args!(ex, new);

    let mut obj = unsafe { ex.get_object::<TestClass>() }.unwrap();
    obj.c = new.val().unwrap();
    retval.set_null();
}

impl Default for TestClass {
    fn default() -> Self {
        Self {
            a: 100,
            b: 123,
            c: "Hello, world!".into(),
        }
    }
}

static TEST_CLASS_META: ClassMetadata<TestClass> = ClassMetadata::new();

impl RegisteredClass for TestClass {
    const CLASS_NAME: &'static str = "TestClass";

    fn get_metadata() -> &'static ext_php_rs::php::types::object::ClassMetadata<Self> {
        &TEST_CLASS_META
    }

    fn get_properties(&mut self) -> HashMap<&'static str, &mut dyn Prop> {
        HashMap::from_iter([
            ("a", &mut self.a as &mut dyn Prop),
            ("b", &mut self.b as &mut dyn Prop),
            ("c", &mut self.c as &mut dyn Prop),
        ])
    }
}

#[php_startup]
pub fn startup() {
    let ce = ClassBuilder::new("TestClass")
        .method(
            FunctionBuilder::new("set_c", test_class_change)
                .arg(Arg::new("c", DataType::String))
                .build()
                .unwrap(),
            MethodFlags::Public,
        )
        .property("test", 5, PropertyFlags::Public)
        .object_override::<TestClass>()
        .build()
        .unwrap();
    TEST_CLASS_META.set_ce(ce);
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
