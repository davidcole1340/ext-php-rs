//! # Macros for PHP bindings
mod class;
mod constant;
mod extern_;
mod fastcall;
mod function;
mod helpers;
mod impl_;
mod method;
mod module;
mod module_builder;
mod startup_function;
mod syn_ext;
mod zval;

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{parse_macro_input, DeriveInput, ItemFn, ItemForeignMod, ItemMod};

extern crate proc_macro;

/// Structs can be exported to PHP as classes with the #[php_class] attribute macro. This attribute derives the RegisteredClass trait on your struct, as well as registering the class to be registered with the #[php_module] macro.
#[proc_macro_attribute]
pub fn php_class(_args: TokenStream, _input: TokenStream) -> TokenStream {
    syn::Error::new(
        Span::call_site(),
        "php_class can only be used inside a #[php_module] module",
    )
    .to_compile_error()
    .into()
}

/// Used to annotate functions which should be exported to PHP. Note that this should not be used on class methods - see the #[php_impl] macro for that.
#[proc_macro_attribute]
pub fn php_function(_args: TokenStream, _input: TokenStream) -> TokenStream {
    syn::Error::new(
        Span::call_site(),
        "php_function can only be used inside a #[php_module] module",
    )
    .to_compile_error()
    .into()
}

/// The module macro is used to annotate the get_module function, which is used by the PHP interpreter to retrieve information about your extension, including the name, version, functions and extra initialization functions. Regardless if you use this macro, your extension requires a extern "C" fn get_module() so that PHP can get this information.
#[proc_macro_attribute]
pub fn php_module(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemMod);

    match module::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

/// Used to define the PHP extension startup function. This function is used to register extension classes and constants with the PHP interpreter.
#[proc_macro_attribute]
pub fn php_startup(_args: TokenStream, _input: TokenStream) -> TokenStream {
    syn::Error::new(
        Span::call_site(),
        "php_startup can only be used inside a #[php_module] module",
    )
    .to_compile_error()
    .into()
}

/// You can export an entire impl block to PHP. This exports all methods as well as constants to PHP on the class that it is implemented on. This requires the #[php_class] macro to already be used on the underlying struct. Trait implementations cannot be exported to PHP.
#[proc_macro_attribute]
pub fn php_impl(_args: TokenStream, _input: TokenStream) -> TokenStream {
    syn::Error::new(
        Span::call_site(),
        "php_impl can only be used inside a #[php_module] module",
    )
    .to_compile_error()
    .into()
}

/// Exports a Rust constant as a global PHP constant. The constant can be any type that implements [`IntoConst`].
#[proc_macro_attribute]
pub fn php_const(_args: TokenStream, _input: TokenStream) -> TokenStream {
    syn::Error::new(
        Span::call_site(),
        "php_const can only be used inside a #[php_module] module",
    )
    .to_compile_error()
    .into()
}

/// Attribute used to annotate `extern` blocks which are deemed as PHP
/// functions.
///
/// This allows you to 'import' PHP functions into Rust so that they can be
/// called like regular Rust functions. Parameters can be any type that
/// implements [`IntoZval`], and the return type can be anything that implements
/// [`From<Zval>`] (notice how [`Zval`] is consumed rather than borrowed in this
/// case).
#[proc_macro_attribute]
pub fn php_extern(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemForeignMod);

    match extern_::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

/// Derives the traits required to convert a struct or enum to and from a
/// [`Zval`]. Both [`FromZval`] and [`IntoZval`] are implemented on types which
/// use this macro.
#[proc_macro_derive(ZvalConvert)]
pub fn zval_convert_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match zval::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
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
#[proc_macro]
pub fn zend_fastcall(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    match fastcall::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}
