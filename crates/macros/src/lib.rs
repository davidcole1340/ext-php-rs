//! Macros for the `php-ext` crate.
mod class;
mod constant;
mod extern_;
mod fastcall;
mod function;
mod helpers;
mod impl_;
mod module;
mod syn_ext;
mod zval;

use lsp_doc_stable::lsp_doc;
use proc_macro::TokenStream;
use syn::{
    parse_macro_input, AttributeArgs, DeriveInput, ItemConst, ItemFn, ItemForeignMod, ItemImpl,
    ItemStruct,
};

extern crate proc_macro;

// Not included in the doc tests, as they depend on `ext-php-rs` being available.
// The guide tests will cover these macros.
#[cfg(not(doctest))]
#[lsp_doc("guide/src/macros/classes.md")]
#[proc_macro_attribute]
pub fn php_class(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemStruct);

    class::parser(args, input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

// Not included in the doc tests, as they depend on `ext-php-rs` being available.
// The guide tests will cover these macros.
#[cfg(not(doctest))]
#[lsp_doc("guide/src/macros/function.md")]
#[proc_macro_attribute]
pub fn php_function(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemFn);

    function::parser(args, input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

// Not included in the doc tests, as they depend on `ext-php-rs` being available.
// The guide tests will cover these macros.
#[cfg(not(doctest))]
#[lsp_doc("guide/src/macros/constant.md")]
#[proc_macro_attribute]
pub fn php_const(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemConst);

    constant::parser(input).into()
}

// Not included in the doc tests, as they depend on `ext-php-rs` being available.
// The guide tests will cover these macros.
#[cfg(not(doctest))]
#[lsp_doc("guide/src/macros/module.md")]
#[proc_macro_attribute]
pub fn php_module(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemFn);

    module::parser(args, input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

// Not included in the doc tests, as they depend on `ext-php-rs` being available.
// The guide tests will cover these macros.
#[cfg(not(doctest))]
#[lsp_doc("guide/src/macros/impl.md")]
#[proc_macro_attribute]
pub fn php_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemImpl);

    impl_::parser(args, input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

// Not included in the doc tests, as they depend on `ext-php-rs` being available.
// The guide tests will cover these macros.
#[cfg(not(doctest))]
#[lsp_doc("guide/src/macros/extern.md")]
#[proc_macro_attribute]
pub fn php_extern(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemForeignMod);

    extern_::parser(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

// Not included in the doc tests, as they depend on `ext-php-rs` being available.
// The guide tests will cover these macros.
#[cfg(not(doctest))]
#[lsp_doc("guide/src/macros/zval_convert.md")]
#[proc_macro_derive(ZvalConvert)]
pub fn zval_convert_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    zval::parser(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[cfg(not(doctest))]
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
/// ```
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

    match function::wrap(input) {
        Ok(parsed) => parsed,
        Err(e) => e.to_compile_error(),
    }
    .into()
}

/// Wraps a constant to be used in the [`ModuleBuilder::constant`] method.
#[proc_macro]
pub fn wrap_constant(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::Path);

    match constant::wrap(input) {
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
            match self {
                Some(token) => quote::quote! { ::std::option::Option::Some(#token) },
                None => quote::quote! { ::std::option::Option::None },
            }
        }
    }

    pub(crate) use crate::{bail, err};
    pub(crate) type Result<T> = std::result::Result<T, syn::Error>;
}
