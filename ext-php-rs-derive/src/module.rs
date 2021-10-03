use anyhow::{anyhow, bail, Result};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{ItemFn, Signature};

use crate::{class::Class, startup_function, STATE};

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
    let registered_classes_impls = state
        .classes
        .iter()
        .map(|(_, class)| generate_registered_class_impl(class))
        .collect::<Result<Vec<_>>>()?;

    let result = quote! {
        #(#registered_classes_impls)*

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

/// Generates an implementation for `RegisteredClass` on the given class.
pub fn generate_registered_class_impl(class: &Class) -> Result<TokenStream> {
    let self_ty = Ident::new(&class.struct_path, Span::call_site());
    let class_name = &class.class_name;
    let meta = Ident::new(&format!("_{}_META", &class.struct_path), Span::call_site());
    let prop_tuples = class
        .properties
        .iter()
        .map(|(name, prop)| prop.as_prop_tuple(name));
    let constructor = if let Some(constructor) = &class.constructor {
        let func = Ident::new(&constructor.ident, Span::call_site());
        let args = constructor.get_arg_definitions();
        quote! {
            Some(::ext_php_rs::php::types::object::ConstructorMeta {
                constructor: Self::#func,
                build_fn: {
                    use ext_php_rs::php::function::FunctionBuilder;
                    fn build_fn(func: FunctionBuilder) -> FunctionBuilder {
                        func
                        #(#args)*
                    }
                    build_fn
                }
            })
        }
    } else {
        quote! { None }
    };

    Ok(quote! {
        static #meta: ::ext_php_rs::php::types::object::ClassMetadata<#self_ty> = ::ext_php_rs::php::types::object::ClassMetadata::new();

        impl ::ext_php_rs::php::types::object::RegisteredClass for #self_ty {
            const CLASS_NAME: &'static str = #class_name;
            const CONSTRUCTOR: ::std::option::Option<
                ::ext_php_rs::php::types::object::ConstructorMeta<Self>
            > = #constructor;

            fn get_metadata() -> &'static ::ext_php_rs::php::types::object::ClassMetadata<Self> {
                &#meta
            }

            fn get_properties<'a>() -> ::std::collections::HashMap<&'static str, ::ext_php_rs::php::types::props::Property<'a, Self>> {
                use ::std::iter::FromIterator;

                ::std::collections::HashMap::from_iter([
                    #(#prop_tuples)*
                ])
            }
        }
    })
}
