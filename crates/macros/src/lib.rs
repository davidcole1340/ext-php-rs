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
mod startup_function;
mod syn_ext;
mod zval;

use std::collections::HashMap;

use constant::Constant;
use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, spanned::Spanned, Attribute, AttributeArgs, DeriveInput, Item, ItemConst,
    ItemFn, ItemForeignMod, ItemImpl, ItemMod, ItemStruct, NestedMeta,
};

extern crate proc_macro;

#[derive(Default, Debug)]
struct State {
    functions: Vec<function::Function>,
    classes: HashMap<String, class::Class>,
    constants: Vec<Constant>,
    startup_function: Option<String>,
}

impl State {
    fn parse_item(&mut self, item: &Item) -> TokenStream2 {
        let mut unparsed_attr = vec![];
        let tokens: TokenStream2 = match item {
            Item::Fn(func) => {
                for attr in &func.attrs {
                    if attr.style == syn::AttrStyle::Outer && attr.path.is_ident("php_function") {
                        match Self::parse_attr(attr) {
                            Ok(attr_args) => {
                                let (tokens, func) = function::parser(attr_args, func)
                                    .expect("Failed to parse function");
                                self.functions.push(func);
                                return tokens;
                            }
                            Err(e) => return e,
                        }
                    } else if attr.style == syn::AttrStyle::Outer
                        && attr.path.is_ident("php_startup")
                    {
                        match Self::parse_attr(attr) {
                            Ok(attr_args) => {
                                let (tokens, func) =
                                    startup_function::parser(Some(attr_args), func)
                                        .expect("Failed to parse startup function");
                                self.startup_function = Some(func.to_string());
                                return tokens;
                            }
                            Err(e) => return e,
                        }
                    } else {
                        unparsed_attr.push(attr.into_token_stream());
                    }

                    unparsed_attr.push(attr.into_token_stream());
                }
                return quote! { #func };
            }
            _ => quote! { #item },
        };

        tokens
    }

    fn parse_attr<T>(attr: &Attribute) -> Result<T, TokenStream2>
    where
        T: FromMeta,
    {
        let meta = Self::parse_metadata(attr);

        Self::parse_from_meta(&meta, Some(attr.span()))
    }

    fn parse_metadata(attr: &Attribute) -> Vec<NestedMeta> {
        if let Ok(args) = attr.parse_args::<TokenStream2>().map(|args| args.into()) {
            syn::parse_macro_input::parse::<AttributeArgs>(args).unwrap_or(vec![])
        } else {
            vec![]
        }
    }

    fn parse_from_meta<T>(
        meta: &Vec<NestedMeta>,
        call_site: Option<Span>,
    ) -> Result<T, TokenStream2>
    where
        T: FromMeta,
    {
        T::from_list(&meta).map_err(|e| {
            syn::Error::new(
                call_site.unwrap_or_else(Span::call_site),
                format!("Unable to parse attribute arguments: {:?}", e),
            )
            .to_compile_error()
            .into()
        })
    }
}

#[proc_macro_attribute]
pub fn php_class(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemStruct);

    match class::parser(args, input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn php_function(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemFn);
    let attr_args = match State::parse_from_meta(&args, None) {
        Ok(attr_args) => attr_args,
        Err(e) => return e.into(),
    };

    match function::parser(attr_args, &input) {
        Ok((parsed, _)) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn php_module(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemMod);

    match module::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn php_startup(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemFn);
    let attr_args = match State::parse_from_meta(&args, None) {
        Ok(attr_args) => attr_args,
        Err(e) => return e.into(),
    };

    match startup_function::parser(Some(attr_args), &input) {
        Ok((parsed, _)) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn php_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemImpl);

    match impl_::parser(args, input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn php_const(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemConst);

    match constant::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn php_extern(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemForeignMod);

    match extern_::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

#[proc_macro_derive(ZvalConvert)]
pub fn zval_convert_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match zval::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

#[proc_macro]
pub fn zend_fastcall(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    match fastcall::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}
