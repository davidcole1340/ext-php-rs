use std::collections::HashMap;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{ItemFn, Signature};

use crate::{class::Class, error::Result, STATE};

pub fn parser(input: ItemFn) -> Result<TokenStream> {
    let ItemFn { sig, block, .. } = input;
    let Signature {
        ident,
        // output,
        // inputs,
        ..
    } = sig;
    let stmts = &block.stmts;

    let classes = {
        let mut state = STATE.lock()?;

        state.startup_function = Some(ident.to_string());
        build_classes(&state.classes)
    };

    let func = quote! {
        #[doc(hidden)]
        pub extern "C" fn #ident(ty: i32, module_number: i32) -> i32 {
            fn internal() {
                #(#stmts)*
            }

            #(#classes)*

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

            quote! {
                ::ext_php_rs::php::class::ClassBuilder::new(#name)
                    #(#methods)*
                    .object_override::<#ident>()
                    .build()
                    .unwrap();
            }
        })
        .collect()
}
