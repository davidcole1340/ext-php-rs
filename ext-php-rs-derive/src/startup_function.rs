use std::collections::HashMap;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{ItemFn, Signature};

use crate::{class::Class, Result};

pub fn parser(input: ItemFn) -> Result<TokenStream> {
    let ItemFn { sig, block, .. } = input;
    let Signature {
        ident,
        // output,
        // inputs,
        ..
    } = sig;
    let stmts = &block.stmts;

    let classes = crate::STATE.with(|state| {
        let mut state = state
            .lock()
            .expect("Unable to lock `ext-php-rs-derive` state mutex.");

        state.startup_function = Some(ident.to_string());
        build_classes(&state.classes)
    });

    let func = quote! {
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
            quote! {
                ::ext_php_rs::php::class::ClassBuilder::new(#name)
                    .object_override::<#ident>()
                    .build()
                    .unwrap();
            }
        })
        .collect()
}
