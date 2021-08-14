use crate::{error::Result, STATE};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::DeriveInput;

#[derive(Debug, Default)]
pub struct Class {
    pub methods: Vec<crate::method::Method>,
    pub constants: Vec<crate::constant::Constant>,
}

pub fn parser(input: DeriveInput) -> Result<TokenStream> {
    let name = input.ident;
    let class_name = name.to_string();
    let handlers = Ident::new(
        format!("__{}_OBJECT_HANDLERS", name).as_str(),
        Span::call_site(),
    );

    let output = quote! {
        static mut #handlers: Option<
            *mut ::ext_php_rs::php::types::object::ZendObjectHandlers
        > = None;

        impl ::ext_php_rs::php::types::object::ZendObjectOverride for #name {
            // Allow clippy ptr-deref lint because PHP guarantees that we are passed a valid pointer.
            #[allow(clippy::not_unsafe_ptr_arg_deref)]
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

    let mut state = STATE.lock()?;

    if state.built_module {
        return Err("The `#[php_module]` macro must be called last to ensure functions and classes are registered.".into());
    }

    if state.classes.contains_key(&class_name) {
        return Err(format!(
            "A class has already been registered with the name `{}`.",
            class_name
        )
        .into());
    }

    state.classes.insert(class_name, Default::default());

    Ok(output)
}
