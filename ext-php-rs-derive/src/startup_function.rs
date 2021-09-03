use std::collections::HashMap;

use anyhow::{anyhow, Result};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Expr, ItemFn, Signature};

use crate::{class::Class, constant::Constant, STATE};

pub fn parser(input: ItemFn) -> Result<TokenStream> {
    let ItemFn { sig, block, .. } = input;
    let Signature { ident, .. } = sig;
    let stmts = &block.stmts;

    let mut state = STATE.lock();
    state.startup_function = Some(ident.to_string());

    let classes = build_classes(&state.classes)?;
    let constants = build_constants(&state.constants);

    let func = quote! {
        #[doc(hidden)]
        pub extern "C" fn #ident(ty: i32, module_number: i32) -> i32 {
            use ::ext_php_rs::php::constants::IntoConst;
            use ::ext_php_rs::php::flags::PropertyFlags;

            fn internal() {
                #(#stmts)*
            }

            #(#classes)*
            #(#constants)*

            // TODO return result?
            internal();

            0
        }
    };

    Ok(func)
}

/// Returns a vector of `ClassBuilder`s for each class.
fn build_classes(classes: &HashMap<String, Class>) -> Result<Vec<TokenStream>> {
    classes
        .iter()
        .map(|(name, class)| {
            let Class { class_name, .. } = &class;
            let ident = Ident::new(name, Span::call_site());
            let meta = Ident::new(&format!("_{}_META", name), Span::call_site());
            let methods = class.methods.iter().map(|method| {
                let builder = method.get_builder(&ident);
                let flags = method.get_flags();
                quote! { .method(#builder.unwrap(), #flags) }
            });
            let constants = class.constants.iter().map(|constant| {
                let name = &constant.name;
                let val = constant.val_tokens();
                quote! { .constant(#name, #val).unwrap() }
            });
            let parent = {
                if let Some(parent) = &class.parent {
                    let expr: Expr = syn::parse_str(parent).map_err(|_| {
                        anyhow!("Invalid expression given for `{}` parent", class_name)
                    })?;
                    Some(quote! { .extends(#expr) })
                } else {
                    None
                }
            };
            let interfaces = class
                .interfaces
                .iter()
                .map(|interface| {
                    let expr: Expr = syn::parse_str(interface).map_err(|_| {
                        anyhow!(
                            "Invalid expression given for `{}` interface: `{}`",
                            class_name,
                            interface
                        )
                    })?;
                    Ok(quote! { .implements(#expr) })
                })
                .collect::<Result<Vec<_>>>()?;
            let properties = class
                .properties
                .iter()
                .map(|(name, (default, flags))| {
                    let default_expr: Expr = syn::parse_str(default).map_err(|_| {
                        anyhow!(
                            "Invalid default value given for property `{}` type: `{}`",
                            name,
                            default
                        )
                    })?;
                    let flags_expr: Expr = syn::parse_str(
                        flags
                            .as_ref()
                            .map(|flags| flags.as_str())
                            .unwrap_or("PropertyFlags::Public"),
                    )
                    .map_err(|_| {
                        anyhow!(
                            "Invalid default value given for property `{}` type: `{}`",
                            name,
                            default
                        )
                    })?;

                    Ok(quote! { .property(#name, #default_expr, #flags_expr) })
                })
                .collect::<Result<Vec<_>>>()?;

            Ok(quote! {{
                let class = ::ext_php_rs::php::class::ClassBuilder::new(#class_name)
                    #(#methods)*
                    #(#constants)*
                    #(#interfaces)*
                    #(#properties)*
                    #parent
                    .object_override::<#ident>()
                    .build()
                    .expect(concat!("Unable to build class `", #class_name, "`"));

                #meta.set_ce(class);
            }})
        })
        .collect::<Result<Vec<_>>>()
}

fn build_constants(constants: &[Constant]) -> Vec<TokenStream> {
    constants
        .iter()
        .map(|constant| {
            let name = &constant.name;
            let val = constant.val_tokens();
            quote! {
                #val.register_constant(#name, module_number).unwrap();
            }
        })
        .collect()
}
