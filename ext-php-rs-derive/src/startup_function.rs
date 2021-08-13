use std::collections::HashMap;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{ItemFn, Signature};

use crate::{class::Class, constant::Constant, error::Result, STATE};

pub fn parser(input: ItemFn) -> Result<TokenStream> {
    let ItemFn { sig, block, .. } = input;
    let Signature {
        ident,
        // output,
        // inputs,
        ..
    } = sig;
    let stmts = &block.stmts;

    let mut state = STATE.lock()?;
    state.startup_function = Some(ident.to_string());

    let classes = build_classes(&state.classes);
    let constants = build_constants(&state.constants);

    let func = quote! {
        #[doc(hidden)]
        pub extern "C" fn #ident(ty: i32, module_number: i32) -> i32 {
            pub use ::ext_php_rs::php::constants::IntoConst;

            fn internal() {
                #(#stmts)*
            }

            #(#classes)*
            #(#constants)*

            // TODO return result?
            internal();

            0
        }
    };

    Ok(func)
}

/// Returns a vector of `ClassBuilder`s for each class.
fn build_classes(classes: &HashMap<String, Class>) -> Vec<TokenStream> {
    classes
        .iter()
        .map(|(name, class)| {
            let ident = Ident::new(name, Span::call_site());
            let methods = class.methods.iter().map(|method| {
                let builder = method.get_builder(&ident);
                let flags = method.get_flags();
                quote! { .method(#builder.unwrap(), #flags) }
            });
            let constants = class.constants.iter().map(|constant| {
                let name = &constant.name;
                let val = constant.val_tokens();
                quote! { .constant(#name, #val).unwrap() }
            });

            quote! {
                ::ext_php_rs::php::class::ClassBuilder::new(#name)
                    #(#methods)*
                    #(#constants)*
                    .object_override::<#ident>()
                    .build()
                    .unwrap();
            }
        })
        .collect()
}

fn build_constants(constants: &[Constant]) -> Vec<TokenStream> {
    constants
        .iter()
        .map(|constant| {
            let name = &constant.name;
            let val = constant.val_tokens();
            quote! {
                #val.register_constant(#name, module_number).unwrap();
            }
        })
        .collect()
}
