use std::collections::{hash_map::Entry, HashMap};

use anyhow::{anyhow, bail, Result};
use darling::{FromMeta, ToTokens};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, AttributeArgs, ItemImpl, Lit, Meta, NestedMeta};

use crate::{
    class::{Property, PropertyAttr},
    constant::Constant,
    method,
};

#[derive(Debug, Clone)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

#[derive(Debug, Copy, Clone, FromMeta)]
pub enum RenameRule {
    #[darling(rename = "none")]
    None,
    #[darling(rename = "camelCase")]
    Camel,
    #[darling(rename = "snake_case")]
    Snake,
}

impl Default for RenameRule {
    fn default() -> Self {
        RenameRule::Camel
    }
}

impl RenameRule {
    /// Change case of an identifier.
    ///
    /// Magic methods are handled specially to make sure they're always cased
    /// correctly.
    pub fn rename(&self, name: impl AsRef<str>) -> String {
        let name = name.as_ref();
        match self {
            RenameRule::None => name.to_string(),
            rule => match name {
                "__construct" => "__construct".to_string(),
                "__destruct" => "__destruct".to_string(),
                "__call" => "__call".to_string(),
                "__call_static" => "__callStatic".to_string(),
                "__get" => "__get".to_string(),
                "__set" => "__set".to_string(),
                "__isset" => "__isset".to_string(),
                "__unset" => "__unset".to_string(),
                "__sleep" => "__sleep".to_string(),
                "__wakeup" => "__wakeup".to_string(),
                "__serialize" => "__serialize".to_string(),
                "__unserialize" => "__unserialize".to_string(),
                "__to_string" => "__toString".to_string(),
                "__invoke" => "__invoke".to_string(),
                "__set_state" => "__set_state".to_string(),
                "__clone" => "__clone".to_string(),
                "__debug_info" => "__debugInfo".to_string(),
                field => match rule {
                    Self::Camel => ident_case::RenameRule::CamelCase.apply_to_field(field),
                    Self::Snake => ident_case::RenameRule::SnakeCase.apply_to_field(field),
                    Self::None => unreachable!(),
                },
            },
        }
    }
}

#[derive(Debug)]
pub enum ParsedAttribute {
    Default(HashMap<String, Lit>),
    Optional(String),
    Visibility(Visibility),
    Rename(String),
    Property {
        prop_name: Option<String>,
        ty: PropAttrTy,
    },
    Constructor,
}

#[derive(Default, Debug, FromMeta)]
#[darling(default)]
pub struct AttrArgs {
    rename_methods: Option<RenameRule>,
}

#[derive(Debug)]
pub enum PropAttrTy {
    Getter,
    Setter,
}

pub fn parser(args: AttributeArgs, input: ItemImpl) -> Result<TokenStream> {
    let args = AttrArgs::from_list(&args)
        .map_err(|e| anyhow!("Unable to parse attribute arguments: {:?}", e))?;

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

    let class = state.classes.get_mut(&class_name).ok_or_else(|| {
        anyhow!(
            "You must use `#[php_class]` on the struct before using this attribute on the impl."
        )
    })?;

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
                    let parsed_method =
                        method::parser(&mut method, args.rename_methods.unwrap_or_default())?;
                    if let Some((prop, ty)) = parsed_method.property {
                        let prop = match class.properties.entry(prop) {
                            Entry::Occupied(entry) => entry.into_mut(),
                            Entry::Vacant(vacant) => vacant.insert(Property::method(None)),
                        };
                        let ident = parsed_method.method.orig_ident.clone();
                        match ty {
                            PropAttrTy::Getter => prop.add_getter(ident)?,
                            PropAttrTy::Setter => prop.add_setter(ident)?,
                        }
                    }
                    if parsed_method.constructor {
                        if class.constructor.is_some() {
                            bail!("You cannot have two constructors on the same class.");
                        }
                        class.constructor = Some(parsed_method.method);
                    } else {
                        class.methods.push(parsed_method.method);
                    }
                    parsed_method.tokens
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
        "rename" => {
            let ident = if let Meta::List(list) = meta {
                if let Some(NestedMeta::Lit(lit)) = list.nested.first() {
                    String::from_value(lit).ok()
                } else {
                    None
                }
            } else {
                None
            }
            .ok_or_else(|| anyhow!("Invalid argument given for `#[rename] macro."))?;

            ParsedAttribute::Rename(ident)
        }
        "getter" => {
            let prop_name = if attr.tokens.is_empty() {
                None
            } else {
                let parsed: PropertyAttr = attr
                    .parse_args()
                    .map_err(|e| anyhow!("Unable to parse `#[getter]` attribute: {}", e))?;
                parsed.rename
            };
            ParsedAttribute::Property {
                prop_name,
                ty: PropAttrTy::Getter,
            }
        }
        "setter" => {
            let prop_name = if attr.tokens.is_empty() {
                None
            } else {
                let parsed: PropertyAttr = attr
                    .parse_args()
                    .map_err(|e| anyhow!("Unable to parse `#[setter]` attribute: {}", e))?;
                parsed.rename
            };
            ParsedAttribute::Property {
                prop_name,
                ty: PropAttrTy::Setter,
            }
        }
        "constructor" => ParsedAttribute::Constructor,
        attr => bail!("Invalid attribute `#[{}]`.", attr),
    })
}

#[cfg(test)]
mod tests {
    use super::RenameRule;

    #[test]
    fn test_rename_magic() {
        for &(magic, expected) in &[
            ("__construct", "__construct"),
            ("__destruct", "__destruct"),
            ("__call", "__call"),
            ("__call_static", "__callStatic"),
            ("__get", "__get"),
            ("__set", "__set"),
            ("__isset", "__isset"),
            ("__unset", "__unset"),
            ("__sleep", "__sleep"),
            ("__wakeup", "__wakeup"),
            ("__serialize", "__serialize"),
            ("__unserialize", "__unserialize"),
            ("__to_string", "__toString"),
            ("__invoke", "__invoke"),
            ("__set_state", "__set_state"),
            ("__clone", "__clone"),
            ("__debug_info", "__debugInfo"),
        ] {
            assert_eq!(magic, RenameRule::None.rename(magic));
            assert_eq!(expected, RenameRule::Camel.rename(magic));
            assert_eq!(expected, RenameRule::Snake.rename(magic));
        }
    }

    #[test]
    fn test_rename_php_methods() {
        for &(original, camel, snake) in &[("get_name", "getName", "get_name")] {
            assert_eq!(original, RenameRule::None.rename(original));
            assert_eq!(camel, RenameRule::Camel.rename(original));
            assert_eq!(snake, RenameRule::Snake.rename(original));
        }
    }
}
