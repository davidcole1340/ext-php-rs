use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{ItemFn, Signature};

use crate::{error::Result, STATE};

pub fn parser(input: ItemFn) -> Result<TokenStream> {
    let ItemFn { sig, block, .. } = input;
    let Signature { output, inputs, .. } = sig;
    let stmts = &block.stmts;

    let (functions, startup) = {
        let mut state = STATE.lock()?;

        if state.built_module {
            return Err(
                "You may only define a module with the `#[php_module]` attribute once.".into(),
            );
        }

        state.built_module = true;

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

        (functions, startup)
    };

    let result = quote! {
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
