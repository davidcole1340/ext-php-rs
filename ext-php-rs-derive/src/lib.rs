mod function;
mod method;
mod module;

use std::sync::Mutex;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DeriveInput, ItemFn};

extern crate proc_macro;

type Result<T> = std::result::Result<T, String>;

#[derive(Default, Debug)]
struct State {
    functions: Vec<module::Function>,
    built_module: bool,
}

thread_local! {
    pub(crate) static STATE: Mutex<State> = Mutex::new(Default::default());
}

/// Derives the implementation of `ZendObjectOverride` for the given structure.
#[proc_macro_derive(ZendObjectHandler)]
pub fn object_handler_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let handlers = Ident::new(
        format!("__{}_OBJECT_HANDLERS", name).as_str(),
        Span::call_site(),
    );

    let output = quote! {
        static mut #handlers: Option<
            *mut ::ext_php_rs::php::types::object::ZendObjectHandlers
        > = None;

        impl ::ext_php_rs::php::types::object::ZendObjectOverride for #name {
            extern "C" fn create_object(
                ce: *mut ::ext_php_rs::php::class::ClassEntry,
            ) -> *mut ::ext_php_rs::php::types::object::ZendObject {
                // SAFETY: The handlers are only modified once, when they are first accessed.
                // At the moment we only support single-threaded PHP installations therefore the pointer contained
                // inside the option can be passed around.
                unsafe {
                    if #handlers.is_none() {
                        #handlers = Some(::ext_php_rs::php::types::object::ZendObjectHandlers::init::<#name>());
                    }

                    // The handlers unwrap can never fail - we check that it is none above.
                    // Unwrapping the result from `new_ptr` is nessacary as C cannot handle results.
                    ::ext_php_rs::php::types::object::ZendClassObject::<#name>::new_ptr(
                        ce,
                        #handlers.unwrap()
                    ).expect("Failed to allocate memory for new Zend object.")
                }
            }
        }
    };

    TokenStream::from(output)
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
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

/// Attribute used to declare a function as a PHP method.
///
/// Although this attribute exports the function to PHP-land, you *must* still register the
/// function using the `ClassBuilder` and `FunctionBuilder` in your `startup_function`.
///
/// This attribute should be used on functions implemented on `struct`s that also implements the
/// `ZendClassObject`:trait.
///
/// See the documentation for the [`macro@php_function`] attribute for a deeper dive into the valid
/// parameter and return types.
///
/// Behind the scenes, the function is renamed, and an `extern "C"` function is generated in its
/// place, which is called from PHP. The first example below would be represented like so (only the
/// `get_num` function is shown):
///
/// ```ignore
/// impl TestClass {
///     pub extern "C" fn get_num(ex: &mut ExecutionData, retval: &mut Zval) {
///         let this = match Self::get() {
///             Some(this) => this,
///             None => {
///                 throw(
///                     ClassEntry::exception(),
///                     "Failed to retrieve reference to object function was called on."
///                 )
///                 .unwrap();
///                 return;
///             }
///         };
///
///         let result = this._internal_get_num();
///         match result.set_zval(retval, false) {
///             Ok(_) => {},
///             Err(e) => {
///                 throw(
///                     ClassEntry::exception(),
///                     e.to_string().as_ref(),
///                 )
///                 .unwrap();
///             }
///         };
///     }
///
///     fn _internal_get_num(&self) -> i32 {
///         self.num
///     }
/// }
/// ```
///
/// # Examples
///
/// ```ignore
/// #[derive(Debug, Default, ZendObjectHandler)]
/// struct TestClass {
///     num: i32
/// }
///
/// impl TestClass {
///     #[php_method]
///     pub fn get_num(&self) -> i32 {
///         self.num
///     }
///
///     #[php_method]
///     pub fn set_num(&mut self, num: i32) {
///         self.num = num
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn php_method(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemFn);

    match method::parser(args, input) {
        Ok(parsed) => parsed,
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
