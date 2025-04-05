//! Macros for the `php-ext` crate.
mod class;
mod constant;
mod extern_;
mod fastcall;
mod function;
mod helpers;
mod impl_;
mod module;
mod parsing;
mod syn_ext;
mod zval;

use proc_macro::TokenStream;
use syn::{
    parse_macro_input, DeriveInput, ItemConst, ItemFn, ItemForeignMod, ItemImpl, ItemStruct,
};

extern crate proc_macro;

/// # `#[php_class]` Attribute
///
/// Structs can be exported to PHP as classes with the `#[php_class]` attribute
/// macro. This attribute derives the `RegisteredClass` trait on your struct, as
/// well as registering the class to be registered with the `#[php_module]` macro.
///
/// ## Options
///
/// The attribute takes some options to modify the output of the class:
///
/// - `name` - Changes the name of the class when exported to PHP. The Rust struct
///   name is kept the same. If no name is given, the name of the struct is used.
///   Useful for namespacing classes.
///
/// There are also additional macros that modify the class. These macros **must** be
/// placed underneath the `#[php_class]` attribute.
///
/// - `#[extends(ce)]` - Sets the parent class of the class. Can only be used once.
///   `ce` must be a function with the signature `fn() -> &'static ClassEntry`.
/// - `#[implements(ce)]` - Implements the given interface on the class. Can be used
///   multiple times. `ce` must be a valid function with the signature
///   `fn() -> &'static ClassEntry`.
///
/// You may also use the `#[prop]` attribute on a struct field to use the field as a
/// PHP property. By default, the field will be accessible from PHP publicly with
/// the same name as the field. Property types must implement `IntoZval` and
/// `FromZval`.
///
/// You can rename the property with options:
///
/// - `rename` - Allows you to rename the property, e.g.
///   `#[prop(rename = "new_name")]`
///
/// ## Restrictions
///
/// ### No lifetime parameters
///
/// Rust lifetimes are used by the Rust compiler to reason about a program's memory safety.
/// They are a compile-time only concept;
/// there is no way to access Rust lifetimes at runtime from a dynamic language like PHP.
///
/// As soon as Rust data is exposed to PHP,
/// there is no guarantee which the Rust compiler can make on how long the data will live.
/// PHP is a reference-counted language and those references can be held
/// for an arbitrarily long time, which is untraceable by the Rust compiler.
/// The only possible way to express this correctly is to require that any `#[php_class]`
/// does not borrow data for any lifetime shorter than the `'static` lifetime,
/// i.e. the `#[php_class]` cannot have any lifetime parameters.
///
/// When you need to share ownership of data between PHP and Rust,
/// instead of using borrowed references with lifetimes, consider using
/// reference-counted smart pointers such as [Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html).
///
/// ### No generic parameters
///
/// A Rust struct `Foo<T>` with a generic parameter `T` generates new compiled implementations
/// each time it is used with a different concrete type for `T`.
/// These new implementations are generated by the compiler at each usage site.
/// This is incompatible with wrapping `Foo` in PHP,
/// where there needs to be a single compiled implementation of `Foo` which is integrated with the PHP interpreter.
///
/// ## Example
///
/// This example creates a PHP class `Human`, adding a PHP property `address`.
///
/// ```rust,no_run,ignore
/// # #![cfg_attr(windows, feature(abi_vectorcall))]
/// # extern crate ext_php_rs;
/// use ext_php_rs::prelude::*;
///
/// #[php_class]
/// pub struct Human {
///     name: String,
///     age: i32,
///     #[prop]
///     address: String,
/// }
///
/// #[php_module]
/// pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
///     module.class::<Human>()
/// }
/// # fn main() {}
/// ```
///
/// Create a custom exception `RedisException`, which extends `Exception`, and put
/// it in the `Redis\Exception` namespace:
///
/// ```rust,no_run,ignore
/// # #![cfg_attr(windows, feature(abi_vectorcall))]
/// # extern crate ext_php_rs;
/// use ext_php_rs::{
///     prelude::*,
///     exception::PhpException,
///     zend::ce
/// };
///
/// #[php_class(name = "Redis\\Exception\\RedisException")]
/// #[extends(ce::exception)]
/// #[derive(Default)]
/// pub struct RedisException;
///
/// // Throw our newly created exception
/// #[php_function]
/// pub fn throw_exception() -> PhpResult<i32> {
///     Err(PhpException::from_class::<RedisException>("Not good!".into()))
/// }
///
/// #[php_module]
/// pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
///     module
///         .class::<RedisException>()
///         .function(wrap_function!(throw_exception))
/// }
/// # fn main() {}
/// ```
///
/// ## Implementing an Interface
///
/// To implement an interface, use `#[implements(ce)]` where `ce` is an function returning a `ClassEntry`.
/// The following example implements [`ArrayAccess`](https://www.php.net/manual/en/class.arrayaccess.php):
///
/// ````rust,no_run,ignore
/// # #![cfg_attr(windows, feature(abi_vectorcall))]
/// # extern crate ext_php_rs;
/// use ext_php_rs::{
///     prelude::*,
///     exception::PhpResult,
///     types::Zval,
///     zend::ce,
/// };
///
/// #[php_class]
/// #[implements(ce::arrayaccess)]
/// #[derive(Default)]
/// pub struct EvenNumbersArray;
///
/// /// Returns `true` if the array offset is an even number.
/// /// Usage:
/// /// ```php
/// /// $arr = new EvenNumbersArray();
/// /// var_dump($arr[0]); // true
/// /// var_dump($arr[1]); // false
/// /// var_dump($arr[2]); // true
/// /// var_dump($arr[3]); // false
/// /// var_dump($arr[4]); // true
/// /// var_dump($arr[5] = true); // Fatal error:  Uncaught Exception: Setting values is not supported
/// /// ```
/// #[php_impl]
/// impl EvenNumbersArray {
///     pub fn __construct() -> EvenNumbersArray {
///         EvenNumbersArray {}
///     }
///     // We need to use `Zval` because ArrayAccess needs $offset to be a `mixed`
///     pub fn offset_exists(&self, offset: &'_ Zval) -> bool {
///         offset.is_long()
///     }
///     pub fn offset_get(&self, offset: &'_ Zval) -> PhpResult<bool> {
///         let integer_offset = offset.long().ok_or("Expected integer offset")?;
///         Ok(integer_offset % 2 == 0)
///     }
///     pub fn offset_set(&mut self, _offset: &'_ Zval, _value: &'_ Zval) -> PhpResult {
///         Err("Setting values is not supported".into())
///     }
///     pub fn offset_unset(&mut self, _offset: &'_ Zval) -> PhpResult {
///         Err("Setting values is not supported".into())
///     }
/// }
///
/// #[php_module]
/// pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
///     module.class::<EvenNumbersArray>()
/// }
/// # fn main() {}
/// ````
#[proc_macro_attribute]
pub fn php_class(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);

    class::parser(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// # `#[php_function]` Attribute
///
/// Used to annotate functions which should be exported to PHP. Note that this
/// should not be used on class methods - see the `#[php_impl]` macro for that.
///
/// See the [list of types](../types/index.md) that are valid as parameter and
/// return types.
///
/// ## Optional parameters
///
/// Optional parameters can be used by setting the Rust parameter type to a variant
/// of `Option<T>`. The macro will then figure out which parameters are optional by
/// using the last consecutive arguments that are a variant of `Option<T>` or have a
/// default value.
///
/// ```rust,no_run,ignore
/// # #![cfg_attr(windows, feature(abi_vectorcall))]
/// # extern crate ext_php_rs;
/// use ext_php_rs::prelude::*;
///
/// #[php_function]
/// pub fn greet(name: String, age: Option<i32>) -> String {
///     let mut greeting = format!("Hello, {}!", name);
///
///     if let Some(age) = age {
///         greeting += &format!(" You are {} years old.", age);
///     }
///
///     greeting
/// }
///
/// #[php_module]
/// pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
///     module.function(wrap_function!(greet))
/// }
/// # fn main() {}
/// ```
///
/// Default parameter values can also be set for optional parameters. This is done
/// through the `defaults` attribute option. When an optional parameter has a
/// default, it does not need to be a variant of `Option`:
///
/// ```rust,no_run,ignore
/// # #![cfg_attr(windows, feature(abi_vectorcall))]
/// # extern crate ext_php_rs;
/// use ext_php_rs::prelude::*;
///
/// #[php_function(defaults(offset = 0))]
/// pub fn rusty_strpos(haystack: &str, needle: &str, offset: i64) -> Option<usize> {
///     let haystack: String = haystack.chars().skip(offset as usize).collect();
///     haystack.find(needle)
/// }
///
/// #[php_module]
/// pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
///     module.function(wrap_function!(rusty_strpos))
/// }
/// # fn main() {}
/// ```
///
/// Note that if there is a non-optional argument after an argument that is a
/// variant of `Option<T>`, the `Option<T>` argument will be deemed a nullable
/// argument rather than an optional argument.
///
/// ```rust,no_run,ignore
/// # #![cfg_attr(windows, feature(abi_vectorcall))]
/// # extern crate ext_php_rs;
/// use ext_php_rs::prelude::*;
///
/// /// `age` will be deemed required and nullable rather than optional.
/// #[php_function]
/// pub fn greet(name: String, age: Option<i32>, description: String) -> String {
///     let mut greeting = format!("Hello, {}!", name);
///
///     if let Some(age) = age {
///         greeting += &format!(" You are {} years old.", age);
///     }
///
///     greeting += &format!(" {}.", description);
///     greeting
/// }
///
/// #[php_module]
/// pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
///     module.function(wrap_function!(greet))
/// }
/// # fn main() {}
/// ```
///
/// You can also specify the optional arguments if you want to have nullable
/// arguments before optional arguments. This is done through an attribute
/// parameter:
///
/// ```rust,no_run,ignore
/// # #![cfg_attr(windows, feature(abi_vectorcall))]
/// # extern crate ext_php_rs;
/// use ext_php_rs::prelude::*;
///
/// /// `age` will be deemed required and nullable rather than optional,
/// /// while description will be optional.
/// #[php_function(optional = "description")]
/// pub fn greet(name: String, age: Option<i32>, description: Option<String>) -> String {
///     let mut greeting = format!("Hello, {}!", name);
///
///     if let Some(age) = age {
///         greeting += &format!(" You are {} years old.", age);
///     }
///
///     if let Some(description) = description {
///         greeting += &format!(" {}.", description);
///     }
///
///     greeting
/// }
///
/// #[php_module]
/// pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
///     module.function(wrap_function!(greet))
/// }
/// # fn main() {}
/// ```
///
/// ## Variadic Functions
///
/// Variadic functions can be implemented by specifying the last argument in the Rust
/// function to the type `&[&Zval]`. This is the equivalent of a PHP function using
/// the `...$args` syntax.
///
/// ```rust,no_run,ignore
/// # #![cfg_attr(windows, feature(abi_vectorcall))]
/// # extern crate ext_php_rs;
/// use ext_php_rs::{prelude::*, types::Zval};
///
/// /// This can be called from PHP as `add(1, 2, 3, 4, 5)`
/// #[php_function]
/// pub fn add(number: u32, numbers:&[&Zval]) -> u32 {
///     // numbers is a slice of 4 Zvals all of type long
///     number
/// }
///
/// #[php_module]
/// pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
///     module.function(wrap_function!(add))
/// }
/// # fn main() {}
/// ```
///
/// ## Returning `Result<T, E>`
///
/// You can also return a `Result` from the function. The error variant will be
/// translated into an exception and thrown. See the section on
/// [exceptions](../exceptions.md) for more details.
#[proc_macro_attribute]
pub fn php_function(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    function::parser(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// # `#[php_const]` Attribute
///
/// Exports a Rust constant as a global PHP constant. The constant can be any
/// type that implements `IntoConst`.
///
/// The `wrap_constant!()` macro can be used to simplify the registration of
/// constants. It sets the name and doc comments for the constant.
///
/// ## Examples
///
/// ```rust,no_run,ignore
/// # #![cfg_attr(windows, feature(abi_vectorcall))]
/// # extern crate ext_php_rs;
/// use ext_php_rs::prelude::*;
///
/// #[php_const]
/// const TEST_CONSTANT: i32 = 100;
///
/// #[php_const]
/// const ANOTHER_STRING_CONST: &'static str = "Hello world!";
///
/// #[php_module]
/// pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
///     module
///         .constant(wrap_constant!(TEST_CONSTANT))
///         .constant(("MANUAL_CONSTANT", ANOTHER_STRING_CONST, &[]))
/// }
/// # fn main() {}
/// ```
///
/// ## PHP usage
///
/// ```php
/// <?php
///
/// var_dump(TEST_CONSTANT); // int(100)
/// var_dump(MANUAL_CONSTANT); // string(12) "Hello world!"
/// ```
#[proc_macro_attribute]
pub fn php_const(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemConst);

    constant::parser(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// # `#[php_module]` Attribute
///
/// The module macro is used to annotate the `get_module` function, which is
/// used by the PHP interpreter to retrieve information about your extension,
/// including the name, version, functions and extra initialization functions.
/// Regardless if you use this macro, your extension requires a `extern "C" fn
/// get_module()` so that PHP can get this information.
///
/// The function is renamed to `get_module` if you have used another name. The
/// function is passed an instance of `ModuleBuilder` which allows you to
/// register the following (if required):
///
/// - Functions, classes, and constants
/// - Extension and request startup and shutdown functions.
///   - Read more about the PHP extension lifecycle [here](https://www.phpinternalsbook.com/php7/extensions_design/php_lifecycle.html).
/// - PHP extension information function
///   - Used by the `phpinfo()` function to get information about your
///     extension.
///
/// Classes and constants are not registered with PHP in the `get_module`
/// function. These are registered inside the extension startup function.
///
/// ## Usage
///
/// ```rust,no_run,ignore
/// # #![cfg_attr(windows, feature(abi_vectorcall))]
/// # extern crate ext_php_rs;
/// use ext_php_rs::{
///     prelude::*,
///     zend::ModuleEntry,
///     info_table_start,
///     info_table_row,
///     info_table_end
/// };
///
/// #[php_const]
/// pub const MY_CUSTOM_CONST: &'static str = "Hello, world!";
///
/// #[php_class]
/// pub struct Test {
///     a: i32,
///     b: i32
/// }
/// #[php_function]
/// pub fn hello_world() -> &'static str {
///     "Hello, world!"
/// }
///
/// /// Used by the `phpinfo()` function and when you run `php -i`.
/// /// This will probably be simplified with another macro eventually!
/// pub extern "C" fn php_module_info(_module: *mut ModuleEntry) {
///     info_table_start!();
///     info_table_row!("my extension", "enabled");
///     info_table_end!();
/// }
///
/// #[php_module]
/// pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
///     module
///         .constant(wrap_constant!(MY_CUSTOM_CONST))
///         .class::<Test>()
///         .function(wrap_function!(hello_world))
///         .info_function(php_module_info)
/// }
/// # fn main() {}
/// ```
#[proc_macro_attribute]
pub fn php_module(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    module::parser(args.into(), input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// # `#[php_impl]` Attribute
///
/// You can export an entire `impl` block to PHP. This exports all methods as well
/// as constants to PHP on the class that it is implemented on. This requires the
/// `#[php_class]` macro to already be used on the underlying struct. Trait
/// implementations cannot be exported to PHP. Only one `impl` block can be exported
/// per class.
///
/// If you do not want a function exported to PHP, you should place it in a separate
/// `impl` block.
///
/// If you want to use async Rust, use `#[php_async_impl]`, instead: see [here &raquo;](./async_impl.md) for more info.
///
/// ## Methods
///
/// Methods basically follow the same rules as functions, so read about the
/// [`php_function`] macro first. The primary difference between functions and
/// methods is they are bounded by their class object.
///
/// Class methods can take a `&self` or `&mut self` parameter. They cannot take a
/// consuming `self` parameter. Static methods can omit this `self` parameter.
///
/// To access the underlying Zend object, you can take a reference to a
/// `ZendClassObject<T>` in place of the self parameter, where the parameter must
/// be named `self_`. This can also be used to return a reference to `$this`.
///
/// By default, all methods are renamed in PHP to the camel-case variant of the Rust
/// method name. This can be changed on the `#[php_impl]` attribute, by passing one
/// of the following as the `rename_methods` option:
///
/// - `"none"` - does not rename the methods.
/// - `"camelCase"` - renames all methods to camel case (default).
/// - `"snake_case"` - renames all methods to snake case.
///
/// For example, to disable renaming, change the `#[php_impl]` attribute to
/// `#[php_impl(rename_methods = "none")]`.
///
/// The rest of the options are passed as separate attributes:
///
/// - `#[defaults(i = 5, b = "hello")]` - Sets the default value for parameter(s).
/// - `#[optional(i)]` - Sets the first optional parameter. Note that this also sets
///   the remaining parameters as optional, so all optional parameters must be a
///   variant of `Option<T>`.
/// - `#[public]`, `#[protected]` and `#[private]` - Sets the visibility of the
///   method.
/// - `#[rename("method_name")]` - Renames the PHP method to a different identifier,
///   without renaming the Rust method name.
///
/// The `#[defaults]` and `#[optional]` attributes operate the same as the
/// equivalent function attribute parameters.
///
/// ### Constructors
///
/// By default, if a class does not have a constructor, it is not constructable from
/// PHP. It can only be returned from a Rust function to PHP.
///
/// Constructors are Rust methods which can take any amount of parameters and
/// returns either `Self` or `Result<Self, E>`, where `E: Into<PhpException>`. When
/// the error variant of `Result` is encountered, it is thrown as an exception and
/// the class is not constructed.
///
/// Constructors are designated by either naming the method `__construct` or by
/// annotating a method with the `#[constructor]` attribute. Note that when using
/// the attribute, the function is not exported to PHP like a regular method.
///
/// Constructors cannot use the visibility or rename attributes listed above.
///
/// ## Constants
///
/// Constants are defined as regular Rust `impl` constants. Any type that implements
/// `IntoZval` can be used as a constant. Constant visibility is not supported at
/// the moment, and therefore no attributes are valid on constants.
///
/// ## Property getters and setters
///
/// You can add properties to classes which use Rust functions as getters and/or
/// setters. This is done with the `#[getter]` and `#[setter]` attributes. By
/// default, the `get_` or `set_` prefix is trimmed from the start of the function
/// name, and the remainder is used as the property name.
///
/// If you want to use a different name for the property, you can pass a `rename`
/// option to the attribute which will change the property name.
///
/// Properties do not necessarily have to have both a getter and a setter, if the
/// property is immutable the setter can be omitted, and vice versa for getters.
///
/// The `#[getter]` and `#[setter]` attributes are mutually exclusive on methods.
/// Properties cannot have multiple getters or setters, and the property name cannot
/// conflict with field properties defined on the struct.
///
/// As the same as field properties, method property types must implement both
/// `IntoZval` and `FromZval`.
///
/// ## Example
///
/// Continuing on from our `Human` example in the structs section, we will define a
/// constructor, as well as getters for the properties. We will also define a
/// constant for the maximum age of a `Human`.
///
/// ```rust,no_run,ignore
/// # #![cfg_attr(windows, feature(abi_vectorcall))]
/// # extern crate ext_php_rs;
/// use ext_php_rs::{prelude::*, types::ZendClassObject};
///
/// #[php_class]
/// #[derive(Debug, Default)]
/// pub struct Human {
///     name: String,
///     age: i32,
///     #[prop]
///     address: String,
/// }
///
/// #[php_impl]
/// impl Human {
///     const MAX_AGE: i32 = 100;
///
///     // No `#[constructor]` attribute required here - the name is `__construct`.
///     pub fn __construct(name: String, age: i32) -> Self {
///         Self {
///             name,
///             age,
///             address: String::new()
///         }
///     }
///
///     #[getter]
///     pub fn get_name(&self) -> String {
///         self.name.to_string()
///     }
///
///     #[setter]
///     pub fn set_name(&mut self, name: String) {
///         self.name = name;
///     }
///
///     #[getter]
///     pub fn get_age(&self) -> i32 {
///         self.age
///     }
///
///     pub fn introduce(&self) {
///         println!("My name is {} and I am {} years old. I live at {}.", self.name, self.age, self.address);
///     }
///
///     pub fn get_raw_obj(self_: &mut ZendClassObject<Human>) -> &mut ZendClassObject<Human> {
///         dbg!(self_)
///     }
///
///     pub fn get_max_age() -> i32 {
///         Self::MAX_AGE
///     }
/// }
/// #[php_module]
/// pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
///     module.class::<Human>()
/// }
/// # fn main() {}
/// ```
///
/// Using our newly created class in PHP:
///
/// ```php
/// <?php
///
/// $me = new Human('David', 20);
///
/// $me->introduce(); // My name is David and I am 20 years old.
/// var_dump(Human::get_max_age()); // int(100)
/// var_dump(Human::MAX_AGE); // int(100)
/// ```
///
/// [`php_async_impl`]: ./async_impl.md
#[proc_macro_attribute]
pub fn php_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemImpl);

    impl_::parser(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// # `#[php_extern]` Attribute
///
/// Attribute used to annotate `extern` blocks which are deemed as PHP
/// functions.
///
/// This allows you to 'import' PHP functions into Rust so that they can be
/// called like regular Rust functions. Parameters can be any type that
/// implements [`IntoZval`], and the return type can be anything that implements
/// [`From<Zval>`] (notice how [`Zval`] is consumed rather than borrowed in this
/// case).
///
/// Unlike most other attributes, this does not need to be placed inside a
/// `#[php_module]` block.
///
/// # Panics
///
/// The function can panic when called under a few circumstances:
///
/// * The function could not be found or was not callable.
/// * One of the parameters could not be converted into a [`Zval`].
/// * The actual function call failed internally.
/// * The output [`Zval`] could not be parsed into the output type.
///
/// The last point can be important when interacting with functions that return
/// unions, such as [`strpos`] which can return an integer or a boolean. In this
/// case, a [`Zval`] should be returned as parsing a boolean to an integer is
/// invalid, and vice versa.
///
/// # Example
///
/// This `extern` block imports the [`strpos`] function from PHP. Notice that
/// the string parameters can take either [`String`] or [`&str`], the optional
/// parameter `offset` is an [`Option<i64>`], and the return value is a [`Zval`]
/// as the return type is an integer-boolean union.
///
/// ```rust,no_run,ignore
/// # #![cfg_attr(windows, feature(abi_vectorcall))]
/// # extern crate ext_php_rs;
/// use ext_php_rs::{
///     prelude::*,
///     types::Zval,
/// };
///
/// #[php_extern]
/// extern "C" {
///     fn strpos(haystack: &str, needle: &str, offset: Option<i64>) -> Zval;
/// }
///
/// #[php_function]
/// pub fn my_strpos() {
///     assert_eq!(unsafe { strpos("Hello", "e", None) }.long(), Some(1));
/// }
///
/// #[php_module]
/// pub fn module(module: ModuleBuilder) -> ModuleBuilder {
///     module.function(wrap_function!(my_strpos))
/// }
/// # fn main() {}
/// ```
///
/// [`strpos`]: https://www.php.net/manual/en/function.strpos.php
/// [`IntoZval`]: crate::convert::IntoZval
/// [`Zval`]: crate::types::Zval
#[proc_macro_attribute]
pub fn php_extern(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemForeignMod);

    extern_::parser(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// # `ZvalConvert` Derive Macro
///
/// The `#[derive(ZvalConvert)]` macro derives the `FromZval` and `IntoZval` traits
/// on a struct or enum.
///
/// ## Structs
///
/// When used on a struct, the `FromZendObject` and `IntoZendObject` traits are also
/// implemented, mapping fields to properties in both directions. All fields on the
/// struct must implement `FromZval` as well. Generics are allowed on structs that
/// use the derive macro, however, the implementation will add a `FromZval` bound to
/// all generics types.
///
/// ### Examples
///
/// ```rust,no_run,ignore
/// # #![cfg_attr(windows, feature(abi_vectorcall))]
/// # extern crate ext_php_rs;
/// use ext_php_rs::prelude::*;
///
/// #[derive(ZvalConvert)]
/// pub struct ExampleClass<'a> {
///     a: i32,
///     b: String,
///     c: &'a str
/// }
///
/// #[php_function]
/// pub fn take_object(obj: ExampleClass) {
///     dbg!(obj.a, obj.b, obj.c);
/// }
///
/// #[php_function]
/// pub fn give_object() -> ExampleClass<'static> {
///     ExampleClass {
///         a: 5,
///         b: "String".to_string(),
///         c: "Borrowed",
///     }
/// }
///
/// #[php_module]
/// pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
///     module
///         .function(wrap_function!(take_object))
///         .function(wrap_function!(give_object))
/// }
/// # fn main() {}
/// ```
///
/// Calling from PHP:
///
/// ```php
/// <?php
///
/// $obj = new stdClass;
/// $obj->a = 5;
/// $obj->b = 'Hello, world!';
/// $obj->c = 'another string';
///
/// take_object($obj);
/// var_dump(give_object());
/// ```
///
/// Another example involving generics:
///
/// ```rust,no_run,ignore
/// # #![cfg_attr(windows, feature(abi_vectorcall))]
/// # extern crate ext_php_rs;
/// use ext_php_rs::prelude::*;
///
/// // T must implement both `PartialEq<i32>` and `FromZval`.
/// #[derive(Debug, ZvalConvert)]
/// pub struct CompareVals<T: PartialEq<i32>> {
///     a: T,
///     b: T
/// }
///
/// #[php_function]
/// pub fn take_object(obj: CompareVals<i32>) {
///     dbg!(obj);
/// }
///
/// #[php_module]
/// pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
///     module
///         .function(wrap_function!(take_object))
/// }
/// # fn main() {}
/// ```
///
/// ## Enums
///
/// When used on an enum, the `FromZval` implementation will treat the enum as a
/// tagged union with a mixed datatype. This allows you to accept multiple types in
/// a parameter, for example, a string and an integer.
///
/// The enum variants must not have named fields, and each variant must have exactly
/// one field (the type to extract from the zval). Optionally, the enum may have one
/// default variant with no data contained, which will be used when the rest of the
/// variants could not be extracted from the zval.
///
/// The ordering of the variants in the enum is important, as the `FromZval`
/// implementation will attempt to parse the zval data in order. For example, if you
/// put a `String` variant before an integer variant, the integer would be converted
/// to a string and passed as the string variant.
///
/// ### Examples
///
/// Basic example showing the importance of variant ordering and default field:
///
/// ```rust,no_run,ignore
/// # #![cfg_attr(windows, feature(abi_vectorcall))]
/// # extern crate ext_php_rs;
/// use ext_php_rs::prelude::*;
///
/// #[derive(Debug, ZvalConvert)]
/// pub enum UnionExample<'a> {
///     Long(u64), // Long
///     ProperStr(&'a str), // Actual string - not a converted value
///     ParsedStr(String), // Potentially parsed string, i.e. a double
///     None // Zval did not contain anything that could be parsed above
/// }
///
/// #[php_function]
/// pub fn test_union(val: UnionExample) {
///     dbg!(val);
/// }
///
/// #[php_function]
/// pub fn give_union() -> UnionExample<'static> {
///     UnionExample::Long(5)
/// }
///
/// #[php_module]
/// pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
///     module
///         .function(wrap_function!(test_union))
///         .function(wrap_function!(give_union))
/// }
/// # fn main() {}
/// ```
///
/// Use in PHP:
///
/// ```php
/// test_union(5); // UnionExample::Long(5)
/// test_union("Hello, world!"); // UnionExample::ProperStr("Hello, world!")
/// test_union(5.66666); // UnionExample::ParsedStr("5.6666")
/// test_union(null); // UnionExample::None
/// var_dump(give_union()); // int(5)
/// ```
#[proc_macro_derive(ZvalConvert)]
pub fn zval_convert_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    zval::parser(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Defines an `extern` function with the Zend fastcall convention based on
/// operating system.
///
/// On Windows, Zend fastcall functions use the vector calling convention, while
/// on all other operating systems no fastcall convention is used (just the
/// regular C calling convention).
///
/// This macro wraps a function and applies the correct calling convention.
///
/// ## Examples
///
/// ```rust,ignore
/// # #![cfg_attr(windows, feature(abi_vectorcall))]
/// use ext_php_rs::zend_fastcall;
///
/// zend_fastcall! {
///     pub extern fn test_hello_world(a: i32, b: i32) -> i32 {
///         a + b
///     }
/// }
/// ```
///
/// On Windows, this function will have the signature `pub extern "vectorcall"
/// fn(i32, i32) -> i32`, while on macOS/Linux the function will have the
/// signature `pub extern "C" fn(i32, i32) -> i32`.
///
/// ## Support
///
/// The `vectorcall` ABI is currently only supported on Windows with nightly
/// Rust and the `abi_vectorcall` feature enabled.
#[proc_macro]
pub fn zend_fastcall(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    fastcall::parser(input).into()
}

/// Wraps a function to be used in the [`Module::function`] method.
#[proc_macro]
pub fn wrap_function(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::Path);

    match function::wrap(&input) {
        Ok(parsed) => parsed,
        Err(e) => e.to_compile_error(),
    }
    .into()
}

/// Wraps a constant to be used in the [`ModuleBuilder::constant`] method.
#[proc_macro]
pub fn wrap_constant(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::Path);

    match constant::wrap(&input) {
        Ok(parsed) => parsed,
        Err(e) => e.to_compile_error(),
    }
    .into()
}

macro_rules! err {
    ($span:expr => $($msg:tt)*) => {
        ::syn::Error::new(::syn::spanned::Spanned::span(&$span), format!($($msg)*))
    };
    ($($msg:tt)*) => {
        ::syn::Error::new(::proc_macro2::Span::call_site(), format!($($msg)*))
    };
}

/// Bails out of a function with a syn error.
macro_rules! bail {
    ($span:expr => $($msg:tt)*) => {
        return Err($crate::err!($span => $($msg)*))
    };
    ($($msg:tt)*) => {
        return Err($crate::err!($($msg)*))
    };
}

pub(crate) use bail;
pub(crate) use err;

pub(crate) mod prelude {
    pub(crate) trait OptionTokens {
        fn option_tokens(&self) -> proc_macro2::TokenStream;
    }

    impl<T: quote::ToTokens> OptionTokens for Option<T> {
        fn option_tokens(&self) -> proc_macro2::TokenStream {
            if let Some(token) = self {
                quote::quote! { ::std::option::Option::Some(#token) }
            } else {
                quote::quote! { ::std::option::Option::None }
            }
        }
    }

    pub(crate) use crate::{bail, err};
    pub(crate) type Result<T> = std::result::Result<T, syn::Error>;
}
