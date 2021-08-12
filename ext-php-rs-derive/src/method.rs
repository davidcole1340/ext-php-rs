use std::collections::HashMap;

use crate::{function, Result};
use darling::FromMeta;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, AttributeArgs, FnArg, ItemFn, Lit, Pat, Signature, Token};

enum Arg {
    Receiver(Option<Token![mut]>),
    Typed(function::Arg),
}

#[derive(Debug, Default, FromMeta)]
#[darling(default)]
struct AttrArgs {
    optional: Option<String>,
    #[darling(rename = "static")]
    _static: bool,
    defaults: HashMap<String, Lit>,
}

pub fn parser(args: AttributeArgs, input: ItemFn) -> Result<TokenStream> {
    let attr_args = match AttrArgs::from_list(&args) {
        Ok(args) => args,
        Err(e) => return Err(format!("Unable to parse attribute arguments: {:?}", e)),
    };

    if attr_args._static {
        return function::parser(args, input);
    }

    let ItemFn { sig, block, .. } = input;
    let Signature {
        ident,
        output,
        inputs,
        ..
    } = sig;
    let stmts = &block.stmts;

    let internal_ident = Ident::new(format!("_internal_{}", ident).as_ref(), Span::call_site());
    let args = build_args(&inputs, &attr_args.defaults)?;
    let arg_definitions = build_arg_definitions(&args);
    let arg_parser = build_arg_parser(args.iter(), &attr_args.optional)?;
    let arg_accessors = build_arg_accessors(&args);
    let return_handler = function::build_return_handler(&output);

    let func = quote! {
        #[doc(hidden)]
        fn #internal_ident(#inputs) #output {
            #(#stmts)*
        }

        pub extern "C" fn #ident(ex: &mut ::ext_php_rs::php::execution_data::ExecutionData, retval: &mut ::ext_php_rs::php::types::zval::Zval) {
            use ::ext_php_rs::php::types::zval::IntoZval;

            #(#arg_definitions)*
            #arg_parser

            let result = this.#internal_ident(#(#arg_accessors, )*);

            #return_handler
        }
    };
    Ok(func)
}

fn build_args(
    inputs: &Punctuated<FnArg, Token![,]>,
    defaults: &HashMap<String, Lit>,
) -> Result<Vec<Arg>> {
    inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Receiver(receiver) => {
                if receiver.reference.is_none() {
                    return Err("`self` parameter must be a reference.".into());
                }
                Ok(Arg::Receiver(receiver.mutability))
            }
            FnArg::Typed(ty) => {
                let name = match &*ty.pat {
                    Pat::Ident(pat) => pat.ident.to_string(),
                    _ => return Err("Invalid parameter type.".into()),
                };
                Ok(Arg::Typed(crate::function::syn_arg_to_arg(
                    &name,
                    &ty.ty,
                    defaults.get(&name),
                )?)) // TODO defaults
            }
        })
        .collect()
}

fn build_arg_definitions(args: &[Arg]) -> Vec<TokenStream> {
    args.iter()
        .map(|ty| match ty {
            Arg::Receiver(mutability) => {
                quote! {
                    let #mutability this = match ::ext_php_rs::php::types::object::ZendClassObject::<Self>::get(ex) {
                        Some(this) => this,
                        None => return ::ext_php_rs::php::exceptions::throw(
                            ::ext_php_rs::php::class::ClassEntry::exception(),
                            "Failed to retrieve reference to object function was called on."
                        ).expect("Failed to throw exception: Failed to retrieve reference to object function was called on."),
                    };
                }
            }
            Arg::Typed(arg) => {
                let ident = arg.get_name_ident();
                let definition = arg.get_arg_definition();
                quote! { 
                    let mut #ident = #definition;
                }
            },
        })
        .collect()
}

fn build_arg_parser<'a>(
    args: impl Iterator<Item = &'a Arg>,
    optional: &Option<String>,
) -> Result<TokenStream> {
    function::build_arg_parser(
        args.filter_map(|arg| match arg {
            Arg::Typed(arg) => Some(arg),
            _ => None,
        }),
        optional,
    )
}

fn build_arg_accessors(args: &[Arg]) -> Vec<TokenStream> {
    args.iter()
        .filter_map(|arg| match arg {
            Arg::Typed(arg) => Some(arg.get_accessor()),
            _ => None,
        })
        .collect()
}
