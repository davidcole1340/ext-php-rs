use crate::STATE;
use anyhow::{bail, Result};
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
        static #handlers: ::ext_php_rs::php::types::object::Handlers<#name> = ::ext_php_rs::php::types::object::Handlers::new();

        impl ::ext_php_rs::php::types::object::ZendObjectOverride for #name {
            unsafe extern "C" fn create_object(
                ce: *mut ::ext_php_rs::php::class::ClassEntry,
            ) -> *mut ::ext_php_rs::php::types::object::ZendObject {
                ::ext_php_rs::php::types::object::ZendClassObject::<#name>::new_ptr(ce, #handlers.get())
                    .expect("Failed to allocate memory for new Zend object.")
            }
        }
    };

    let mut state = STATE.lock();

    if state.built_module {
        bail!("The `#[php_module]` macro must be called last to ensure functions and classes are registered.");
    }

    if state.classes.contains_key(&class_name) {
        bail!(
            "A class has already been registered with the name `{}`.",
            class_name
        );
    }

    state.classes.insert(class_name, Default::default());

    Ok(output)
}
