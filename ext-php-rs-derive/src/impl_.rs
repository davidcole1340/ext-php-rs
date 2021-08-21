use std::collections::HashMap;

use anyhow::{anyhow, bail, Result};
use darling::{FromMeta, ToTokens};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, ItemImpl, Lit, Meta, NestedMeta};

use crate::{constant::Constant, method};

#[derive(Debug, Clone)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

#[derive(Debug)]
pub enum ParsedAttribute {
    Default(HashMap<String, Lit>),
    Optional(String),
    Visibility(Visibility),
}

pub fn parser(input: ItemImpl) -> Result<TokenStream> {
    let ItemImpl { self_ty, items, .. } = input;
    let class_name = self_ty.to_token_stream().to_string();

    if input.trait_.is_some() {
        bail!("This macro cannot be used on trait implementations.");
    }

    let mut state = crate::STATE.lock();

    if state.startup_function.is_some() {
        bail!(
            "Impls must be declared before you declare your startup function and module function."
        );
    }

    let class = state
        .classes
        .get_mut(&class_name)
        .ok_or_else(|| anyhow!("You must use `#[derive(ZendObjectHandler)]` on the struct before using this attribute on the impl."))?;

    let tokens = items
        .into_iter()
        .map(|item| {
            Ok(match item {
                syn::ImplItem::Const(constant) => {
                    class.constants.push(Constant {
                        name: constant.ident.to_string(),
                        // visibility: Visibility::Public,
                        value: constant.expr.to_token_stream().to_string(),
                    });

                    quote! {
                        #[allow(dead_code)]
                        #constant
                    }
                }
                syn::ImplItem::Method(mut method) => {
                    let (sig, method) = method::parser(&mut method)?;
                    class.methods.push(method);
                    sig
                }
                item => item.to_token_stream(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let output = quote! {
        impl #self_ty {
            #(#tokens)*
        }
    };
    Ok(output)
}

pub fn parse_attribute(attr: &Attribute) -> Result<ParsedAttribute> {
    let name = attr.path.to_token_stream().to_string();
    let meta = attr
        .parse_meta()
        .map_err(|_| anyhow!("Unable to parse attribute."))?;

    Ok(match name.as_ref() {
        "defaults" => {
            let defaults = HashMap::from_meta(&meta)
                .map_err(|_| anyhow!("Unable to parse `#[default]` macro."))?;
            ParsedAttribute::Default(defaults)
        }
        "optional" => {
            let name = if let Meta::List(list) = meta {
                if let Some(NestedMeta::Meta(meta)) = list.nested.first() {
                    Some(meta.to_token_stream().to_string())
                } else {
                    None
                }
            } else {
                None
            }
            .ok_or_else(|| anyhow!("Invalid argument given for `#[optional]` macro."))?;

            ParsedAttribute::Optional(name)
        }
        "public" => ParsedAttribute::Visibility(Visibility::Public),
        "protected" => ParsedAttribute::Visibility(Visibility::Protected),
        "private" => ParsedAttribute::Visibility(Visibility::Private),
        attr => bail!("Invalid attribute `#[{}]`.", attr),
    })
}
