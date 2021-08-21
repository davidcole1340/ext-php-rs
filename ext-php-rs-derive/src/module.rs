use anyhow::{anyhow, bail, Result};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{ItemFn, Signature};

use crate::{startup_function, STATE};

pub fn parser(input: ItemFn) -> Result<TokenStream> {
    let ItemFn { sig, block, .. } = input;
    let Signature { output, inputs, .. } = sig;
    let stmts = &block.stmts;

    let mut state = STATE.lock();

    if state.built_module {
        bail!("You may only define a module with the `#[php_module]` attribute once.");
    }

    state.built_module = true;

    // Generate startup function if one hasn't already been tagged with the macro.
    let startup_fn = if (!state.classes.is_empty() || !state.constants.is_empty())
        && state.startup_function.is_none()
    {
        drop(state);

        let parsed = syn::parse2(quote! {
            fn php_module_startup() {}
        })
        .map_err(|_| anyhow!("Unable to generate PHP module startup function."))?;
        let startup = startup_function::parser(parsed)?;

        state = STATE.lock();
        Some(startup)
    } else {
        None
    };

    let functions = state
        .functions
        .iter()
        .map(|func| func.get_builder())
        .collect::<Vec<_>>();
    let startup = state.startup_function.as_ref().map(|ident| {
        let ident = Ident::new(ident, Span::call_site());
        quote! {
            .startup_function(#ident)
        }
    });

    let result = quote! {
        #startup_fn

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn get_module() -> *mut ::ext_php_rs::php::module::ModuleEntry {
            fn internal(#inputs) #output {
                #(#stmts)*
            }

            let mut builder = ::ext_php_rs::php::module::ModuleBuilder::new(
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            )
            #startup
            #(.function(#functions.unwrap()))*
            ;

            // TODO allow result return types
            let builder = internal(builder);

            match builder.build() {
                Ok(module) => module.into_raw(),
                Err(e) => panic!("Failed to build PHP module: {:?}", e),
            }
        }
    };
    Ok(result)
}
