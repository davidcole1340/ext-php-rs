use proc_macro2::{TokenStream};
use quote::quote;
use syn::{ItemFn, Signature};

pub fn parser(input: ItemFn) -> TokenStream {
    let ItemFn { sig, block, .. } = input;
    let Signature { output, inputs, .. } = sig;
    let stmts = &block.stmts;

    quote! {
        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn get_module() -> *mut ::ext_php_rs::zend::ModuleEntry {
            static __EXT_PHP_RS_MODULE_STARTUP: ::ext_php_rs::internal::ModuleStartupMutex =
                ::ext_php_rs::internal::MODULE_STARTUP_INIT;

            extern "C" fn ext_php_rs_startup(ty: i32, mod_num: i32) -> i32 {
                __EXT_PHP_RS_MODULE_STARTUP
                    .lock()
                    .take()
                    .expect("Module startup function has already been called.")
                    .startup(ty, mod_num)
                    .map(|_| 0)
                    .unwrap_or(1)
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

            match builder.build() {
                Ok((entry, startup)) => {
                    __EXT_PHP_RS_MODULE_STARTUP.lock().replace(startup);
                    entry.into_raw()
                },
                Err(e) => panic!("Failed to build PHP module: {:?}", e),
            }
        }
    }
}
