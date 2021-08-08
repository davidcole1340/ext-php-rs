use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

extern crate proc_macro;

/// Derives the implementation of `ZendObjectOverride` for the given structure.
#[proc_macro_derive(ZendObjectHandler)]
pub fn object_handler_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let handlers = Ident::new(
        format!("__{}_OBJECT_HANDLERS", name).as_str(),
        Span::call_site(),
    );

    let output = quote! {
        static mut #handlers: Option<
            *mut ::ext_php_rs::php::types::object::ZendObjectHandlers
        > = None;

        impl ::ext_php_rs::php::types::object::ZendObjectOverride for #name {
            extern "C" fn create_object(
                ce: *mut ::ext_php_rs::php::class::ClassEntry,
            ) -> *mut ::ext_php_rs::php::types::object::ZendObject {
                // SAFETY: The handlers are only modified once, when they are first accessed.
                // At the moment we only support single-threaded PHP installations therefore the pointer contained
                // inside the option can be passed around.
                unsafe {
                    if #handlers.is_none() {
                        #handlers = Some(::ext_php_rs::php::types::object::ZendObjectHandlers::init::<#name>());
                    }

                    // The handlers unwrap can never fail - we check that it is none above.
                    // Unwrapping the result from `new_ptr` is nessacary as C cannot handle results.
                    ::ext_php_rs::php::types::object::ZendClassObject::<#name>::new_ptr(
                        ce,
                        #handlers.unwrap()
                    ).expect("Failed to allocate memory for new Zend object.")
                }
            }
        }
    };

    TokenStream::from(output)
}
