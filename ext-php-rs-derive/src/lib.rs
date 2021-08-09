mod function;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DeriveInput, ItemFn};

extern crate proc_macro;

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

/// Function attribute used to modify the footprint of a tagged function to export it as a PHP
/// function.
///
/// Only types which can be converted to and from a `Zval` can be used as parameter and return
/// types. These include but are not limited to:
///
/// - Most primitive integers ([`i8`], [`i16`], [`i32`], [`i64`], [`u8`], [`u16`], [`u32`])
/// - [`bool`]
/// - [`String`]
/// - [`Vec<T>`] and [`HashMap<String, T>`](std::collections::HashMap) containing the above types.
/// - [`Option<T>`] containing the above types. When used as a parameter, the parameter will be
/// deemed nullable, and will contain [`None`] when `null` is passed. Optional parameters *must* be
/// of the type [`Option<T>`]. Returning [`None`] is the same as returning `null`.
///
/// Additionally, you are able to return a variant of [`Result<T, E>`]. `T` must be one of the
/// above types and `E` must implement [`ToString`] (which is implemented through
/// [`Display`](std::fmt::Display) in most cases).
///
/// Parameters may be deemed optional by passing the parameter name into the attribute options.
/// Note that all parameters that are optional (which includes the given optional parameter as well
/// as all parameters after) *must* be of the type [`Option<T>`], where `T` is a valid type.
///
/// Generics are *not* supported.
///
/// # Example
///
/// Creating a simple function which will return a string. The function still must be declared in
/// the PHP module to be able to call.
///
/// ```ignore
/// # use ext_php_rs::php_function;
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
/// # use ext_php_rs::php_function;
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
