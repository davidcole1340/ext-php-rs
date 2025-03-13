use darling::{ast::NestedMeta, FromMeta};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{ItemFn, Signature};

use crate::prelude::*;

#[derive(Debug, Default, FromMeta)]
#[darling(default)]
pub struct ModuleArgs {
    /// Optional function that will be called when the module starts up.
    startup: Option<Ident>,
}

pub fn parser(args: TokenStream, input: ItemFn) -> Result<TokenStream> {
    let meta = NestedMeta::parse_meta_list(args)?;
    let opts = match ModuleArgs::from_list(&meta) {
        Ok(opts) => opts,
        Err(e) => bail!(input => "Failed to parse attribute options: {:?}", e),
    };
    let ItemFn { sig, block, .. } = input;
    let Signature { output, inputs, .. } = sig;
    let stmts = &block.stmts;
    let startup = match opts.startup {
        Some(startup) => quote! { #startup(ty, mod_num) },
        None => quote! { 0i32 },
    };

    Ok(quote! {
        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn get_module() -> *mut ::ext_php_rs::zend::ModuleEntry {
            static __EXT_PHP_RS_MODULE_STARTUP: ::ext_php_rs::internal::ModuleStartupMutex =
                ::ext_php_rs::internal::MODULE_STARTUP_INIT;

            extern "C" fn ext_php_rs_startup(ty: i32, mod_num: i32) -> i32 {
                let a = #startup;
                let b = __EXT_PHP_RS_MODULE_STARTUP
                    .lock()
                    .take()
                    .inspect(|_| ::ext_php_rs::internal::ext_php_rs_startup())
                    .expect("Module startup function has already been called.")
                    .startup(ty, mod_num)
                    .map(|_| 0)
                    .unwrap_or(1);
                a | b
            }

            #[inline]
            fn internal(#inputs) #output {
                #(#stmts)*
            }

            let builder = internal(::ext_php_rs::builders::ModuleBuilder::new(
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            ))
            .startup_function(ext_php_rs_startup);

            match builder.try_into() {
                Ok((entry, startup)) => {
                    __EXT_PHP_RS_MODULE_STARTUP.lock().replace(startup);
                    entry.into_raw()
                },
                Err(e) => panic!("Failed to build PHP module: {:?}", e),
            }
        }

        #[cfg(debug_assertions)]
        #[no_mangle]
        pub extern "C" fn ext_php_rs_describe_module() -> ::ext_php_rs::describe::Description {
            use ::ext_php_rs::describe::*;

            #[inline]
            fn internal(#inputs) #output {
                #(#stmts)*
            }

            let builder = internal(::ext_php_rs::builders::ModuleBuilder::new(
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            ));

            Description::new(builder.into())
        }
    })
}
