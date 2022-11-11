use ext_php_rs::{
    args::Arg,
    builders::{ClassBuilder, FunctionBuilder, ModuleStartup},
    class::{ClassMetadata, ConstructorMeta, ConstructorResult, RegisteredClass},
    convert::IntoZval,
    flags::DataType,
    internal::class::{PhpClassImpl, PhpClassImplCollector},
    prelude::*,
    types::Zval,
    zend::ExecuteData,
};

// struct MyClass {
//     a: i32,
//     b: i32,
// }

// static __INTERNAL_MYCLASS_METADATA: ClassMetadata<MyClass> =
// ClassMetadata::new();

// fn test_modifier(c: ClassBuilder) -> ClassBuilder {
//     println!("in test modifier");
//     c.constant("FAKE_CONST", 5).unwrap()
// }

// impl RegisteredClass for MyClass {
//     const CLASS_NAME: &'static str = "MyClass";
//     const BUILDER_MODIFIER: Option<
//         fn(ext_php_rs::builders::ClassBuilder) ->
// ext_php_rs::builders::ClassBuilder,     > = Some(test_modifier);

//     #[inline]
//     fn get_metadata() -> &'static ext_php_rs::class::ClassMetadata<Self> {
//         &__INTERNAL_MYCLASS_METADATA
//     }

//     #[inline]
//     fn get_properties<'a>(
//     ) -> std::collections::HashMap<&'static str,
// ext_php_rs::props::Property<'a, Self>> {         Default::default()
//     }

//     #[inline]
//     fn method_builders() -> Vec<FunctionBuilder<'static>> {
//         PhpClassImplCollector::<Self>::default().get_methods()
//     }

//     #[inline]
//     fn constructor() -> Option<ConstructorMeta<Self>> {
//         PhpClassImplCollector::<Self>::default().get_constructor()
//     }
// }

// impl MyClass {
//     pub fn __construct(a: i32, b: i32) -> Self {
//         Self { a, b }
//     }

//     pub fn calc(&self, c: i32) -> i32 {
//         self.a * self.b * c
//     }
// }

// impl PhpClassImpl<MyClass> for PhpClassImplCollector<MyClass> {
//     fn get_methods(self) ->
// Vec<ext_php_rs::builders::FunctionBuilder<'static>> {         vec![{
//             ext_php_rs::zend_fastcall! {
//                 extern fn handler(ex: &mut ExecuteData, retval: &mut Zval) {
//                     let (parser, this) = ex.parser_method::<MyClass>();
//                     let mut c = Arg::new("c", DataType::Long);
//                     if parser.arg(&mut c).parse().is_err() {
//                         return;
//                     }
//                     let ret = this.unwrap().calc(c.val().unwrap());
//                     ret.set_zval(retval, false).unwrap();
//                 }
//             }
//             FunctionBuilder::new("calc", handler)
//                 .arg(Arg::new("c", DataType::Long))
//                 .returns(<i32 as ext_php_rs::convert::IntoZval>::TYPE, false,
// false)         }]
//     }

//     fn get_method_props<'a>(
//         self,
//     ) -> std::collections::HashMap<&'static str,
// ext_php_rs::props::Property<'a, MyClass>> {         Default::default()
//     }

//     fn get_constructor(self) ->
// Option<ext_php_rs::class::ConstructorMeta<MyClass>> {         fn
// constructor(ex: &mut ExecuteData) -> ConstructorResult<MyClass> {
// let mut a = Arg::new("a", DataType::Long);             let mut b =
// Arg::new("b", DataType::Long);             if ex.parser().arg(&mut
// a).arg(&mut b).parse().is_err() {                 return
// ConstructorResult::ArgError;             }
//             ConstructorResult::Ok(MyClass {
//                 a: a.val().unwrap(),
//                 b: b.val().unwrap(),
//             })
//         }
//         fn build_fn(func: FunctionBuilder) -> FunctionBuilder {
//             func.arg(Arg::new("a", DataType::Long))
//                 .arg(Arg::new("b", DataType::Long))
//         }
//         Some(ConstructorMeta {
//             constructor,
//             build_fn,
//         })
//     }
// }

#[php_class]
pub struct TestClass {
    #[prop]
    a: i32,
    #[prop]
    b: i32,
}

#[php_impl]
impl TestClass {
    #[rename("NEW_CONSTANT_NAME")]
    pub const SOME_CONSTANT: i32 = 5;
    pub const SOME_OTHER_STR: &'static str = "Hello, world!";

    pub fn __construct(a: i32, b: i32) -> Self {
        Self { a: a + 10, b: b + 10 }
    }

    #[optional(test)]
    #[defaults(a = 5, test = 100)]
    pub fn test_camel_case(&self, a: i32, test: i32) {
        println!("a: {} test: {}", a, test);
    }

    fn x(&self) -> i32 {
        5
    }
}

#[php_function]
pub fn new_class() -> TestClass {
    TestClass { a: 1, b: 2 }
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .class::<TestClass>()
        .function(wrap_function!(new_class))
}
