mod class;
mod error;
mod function;
mod impl_;
mod method;
mod module;
mod startup_function;

use std::{collections::HashMap, sync::Mutex};

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{parse_macro_input, AttributeArgs, DeriveInput, ItemFn, ItemImpl};

extern crate proc_macro;

#[derive(Default, Debug)]
struct State {
    functions: Vec<function::Function>,
    classes: HashMap<String, class::Class>,
    startup_function: Option<String>,
    built_module: bool,
}

lazy_static::lazy_static! {
    pub(crate) static ref STATE: Mutex<State> = Mutex::new(Default::default());
}

/// Derives the implementation of `ZendObjectOverride` for the given structure.
#[proc_macro_derive(ZendObjectHandler)]
pub fn object_handler_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match class::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

/// Attribute used to declare a function as a PHP function.
///
/// Although this attribute exports the function to PHP-land, you *must* still register the
/// function using the `FunctionBuilder` in your `get_module` function.
///
/// Only types which can be converted to and from a `Zval` can be used as parameter and return
/// types. These include but are not limited to:
///
/// - Most primitive integers ([`i8`], [`i16`], [`i32`], [`i64`], [`u8`], [`u16`], [`u32`])
/// - [`bool`]
/// - [`String`]
/// - [`Vec<T>`] and [`HashMap<String, T>`](std::collections::HashMap) containing the above types.
/// - `Binary<T>` for passing binary data as a string.
/// - `Callable` for receiving PHP callables, not applicable for return.
/// - [`Option<T>`] containing the above types. When used as a parameter, the parameter will be
/// deemed nullable, and will contain [`None`] when `null` is passed. Optional parameters *must* be
/// of the type [`Option<T>`]. Returning [`None`] is the same as returning `null`.
///
/// Additionally, you are able to return a variant of [`Result<T, E>`]. `T` must be one of the
/// above types and `E` must implement [`ToString`] (which is implemented through
/// [`Display`](std::fmt::Display) in most cases). If an error variant is returned, a PHP
/// `Exception` is thrown with the error type converted into a string.
///
/// Parameters may be deemed optional by passing the parameter name into the attribute options.
/// Note that all parameters that are optional (which includes the given optional parameter as well
/// as all parameters after) *must* be of the type [`Option<T>`], where `T` is a valid type.
///
/// Generics are *not* supported.
///
/// Behind the scenes, the function is 'wrapped' with an `extern "C"` function which is actually
/// called by PHP. The first example function below would be converted into a function which looks
/// like so:
///
/// ```ignore
/// pub extern "C" fn hello(ex: &mut ExecutionData, retval: &mut Zval) {
///     fn internal(name: String) -> String {
///         format!("Hello, {}!", name)
///     }
///
///     let mut name = Arg::new("name", DataType::String);
///     let parser = ArgParser::new(ex)
///         .arg(&mut name)
///         .parse();
///
///     if parser.is_err() {
///         return;
///     }
///
///     let result = internal(match name.val() {
///         Some(val) => val,
///         None => {
///             throw(
///                 ClassEntry::exception(),
///                 "Invalid value given for argument `name`."
///             )
///             .expect("Failed to throw exception: Invalid value given for argument `name`.");
///             return;
///         }
///     });
///
///     match result.set_zval(retval, false) {
///         Ok(_) => {},
///         Err(e) => {
///             throw(
///                 ClassEntry::exception(),
///                 e.to_string().as_ref()
///             ).expect("Failed to throw exception: Failed to set return value.");
///         }
///     };
/// }
/// ```
///
/// # Examples
///
/// Creating a simple function which will return a string. The function still must be declared in
/// the PHP module to be able to call.
///
/// ```ignore
/// #[php_function]
/// pub fn hello(name: String) -> String {
///     format!("Hello, {}!", name)
/// }
/// ```
///
/// Parameters can also be deemed optional by passing the parameter name in the attribute options.
/// This function takes one required parameter (`hello`) and two optional parameters (`description`
/// and `age`).
///
/// ```ignore
/// #[php_function(optional = "description")]
/// pub fn hello(name: String, description: Option<String>, age: Option<i32>) -> String {
///     let mut response = format!("Hello, {}!", name);
///
///     if let Some(description) = description {
///         response.push_str(format!(" {}.", description).as_ref());
///     }
///
///     if let Some(age) = age {
///         response.push_str(format!(" I am {} year(s) old.", age).as_ref());
///     }
///
///     response
/// }
/// ```
///
/// [`Result<T, E>`]: std::result::Result
#[proc_macro_attribute]
pub fn php_function(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemFn);

    match function::parser(args, input) {
        Ok((parsed, _)) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

/// Annotates a function that will be used by PHP to retrieve information about the module. In the
/// process, the function is wrapped by an `extern "C"` function which is called from PHP, which
/// then calls the given function.
///
/// As well as wrapping the function, the `ModuleBuilder` is initialized ans functions which have
/// already been declared with the [`macro@php_function`] attribute will be registered with the
/// module, so ideally you won't have to do anything inside the function.
///
/// The attribute must be called on a function *last*, i.e. the last proc-macro to be compiled, as
/// the attribute relies on all other PHP attributes being compiled before the module. If another
/// PHP attribute is compiled after the module attribute, an error will be thrown.
///
/// Note that if the function is not called `get_module`, it will be renamed.
///
/// # Example
///
/// The `get_module` function is required in every PHP extension. This is a bare minimum example,
/// since the function is declared above the module it will automatically be registered when the
/// module attribute is called.
///
/// ```ignore
/// #[php_function]
/// pub fn hello(name: String) -> String {
///     format!("Hello, {}!", name)
/// }
///
/// #[php_module]
/// pub fn module(module: ModuleBuilder) -> ModuleBuilder {
///     module
/// }
/// ```
#[proc_macro_attribute]
pub fn php_module(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    match module::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn php_startup(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    match startup_function::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn php_impl(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemImpl);

    match impl_::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}
