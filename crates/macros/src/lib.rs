//! Macros for the `php-ext` crate.
mod class;
mod extern_;
mod fastcall;
mod function;
mod helpers;
mod impl_;
mod module;
mod syn_ext;
mod zval;

use proc_macro::TokenStream;
use syn::{
    parse_macro_input, AttributeArgs, DeriveInput, ItemFn, ItemForeignMod, ItemImpl, ItemStruct,
};

extern crate proc_macro;

#[proc_macro_attribute]
pub fn php_class(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemStruct);

    class::parser(args, input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn php_function(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemFn);

    function::parser(args, input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn php_module(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemFn);

    module::parser(args, input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn php_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemImpl);

    impl_::parser(args, input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn php_extern(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemForeignMod);

    extern_::parser(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_derive(ZvalConvert)]
pub fn zval_convert_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    zval::parser(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro]
pub fn zend_fastcall(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    fastcall::parser(input).into()
}

#[proc_macro]
pub fn wrap_function(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::Path);

    match function::wrap(input) {
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
