use std::collections::HashMap;

use anyhow::{anyhow, bail, Result};
use darling::{FromMeta, ToTokens};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Attribute, AttributeArgs, ItemImpl, Lit, Meta, NestedMeta};

use crate::helpers::get_docs;
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
    This,
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

    let mut constructor = None;
    // let tokens = items
    //     .into_iter()
    //     .map(|item| {
    //         Ok(match item {
    //             syn::ImplItem::Const(constant) => {
    //                 // class.constants.push(Constant {
    //                 //     name: constant.ident.to_string(),
    //                 //     // visibility: Visibility::Public,
    //                 //     docs: get_docs(&constant.attrs),
    //                 //     value: constant.expr.to_token_stream().to_string(),
    //                 // });

    //                 // quote! {
    //                 //     #[allow(dead_code)]
    //                 //     #constant
    //                 // }
    //                 todo!("class constants")
    //             }
    //             syn::ImplItem::Method(method) => {
    //                 let parsed_method =
    //                     method::parser(&self_ty, method,
    // args.rename_methods.unwrap_or_default())?;

    //                 // TODO(david): How do we handle comments for getter/setter?
    // Take the comments                 // // from the methods??
    //                 if let Some((prop, ty)) = parsed_method.property {
    //                     // let prop = class
    //                     //     .properties
    //                     //     .entry(prop)
    //                     //     .or_insert_with(|| Property::method(vec![],
    // None));                     // let ident =
    // parsed_method.method.orig_ident.clone();

    //                     // match ty {
    //                     //     PropAttrTy::Getter => prop.add_getter(ident)?,
    //                     //     PropAttrTy::Setter => prop.add_setter(ident)?,
    //                     // }
    //                     todo!("class property methods")
    //                 }
    //                 if parsed_method.constructor {
    //                     constructor = Some(parsed_method.method);
    //                 }
    //                 parsed_method.tokens
    //             }
    //             item => item.to_token_stream(),
    //         })
    //     })
    //     .collect::<Result<Vec<_>>>()?;

    let mut tokens = vec![];
    let mut methods = vec![];
    for item in items.into_iter() {
        match item {
            syn::ImplItem::Const(constant) => {
                // class.constants.push(Constant {
                //     name: constant.ident.to_string(),
                //     // visibility: Visibility::Public,
                //     docs: get_docs(&constant.attrs),
                //     value: constant.expr.to_token_stream().to_string(),
                // });

                // quote! {
                //     #[allow(dead_code)]
                //     #constant
                // }
                todo!("class constants")
            }
            syn::ImplItem::Method(method) => {
                let parsed_method =
                    method::parser(&self_ty, method, args.rename_methods.unwrap_or_default())?;

                // TODO(david): How do we handle comments for getter/setter? Take the comments
                // // from the methods??
                if let Some((prop, ty)) = parsed_method.property {
                    // let prop = class
                    //     .properties
                    //     .entry(prop)
                    //     .or_insert_with(|| Property::method(vec![], None));
                    // let ident = parsed_method.method.orig_ident.clone();

                    // match ty {
                    //     PropAttrTy::Getter => prop.add_getter(ident)?,
                    //     PropAttrTy::Setter => prop.add_setter(ident)?,
                    // }
                    todo!("class property methods")
                }
                if parsed_method.constructor {
                    constructor = Some(parsed_method.method);
                } else {
                    methods.push(parsed_method.method);
                }
                tokens.push(parsed_method.tokens);
            }
            item => tokens.push(item.to_token_stream()),
        }
    }

    let constructor = if let Some(constructor) = constructor {
        let func = Ident::new(&constructor.ident, Span::call_site());
        let args = constructor.get_arg_definitions();
        quote! {
            Some(::ext_php_rs::class::ConstructorMeta {
                constructor: Self::#func,
                build_fn: {
                    use ::ext_php_rs::builders::FunctionBuilder;
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
    let methods = methods.into_iter().map(|method| {});

    Ok(quote! {
        impl #self_ty {
            #(#tokens)*
        }

        impl ::ext_php_rs::internal::class::PhpClassMethods<#self_ty> for ::ext_php_rs::internal::class::PhpClassPropertyCollector<#self_ty> {
            fn get_methods(self) -> ::std::vec::Vec<::ext_php_rs::builders::FunctionBuilder<'static>> {
            }

            fn get_method_props<'a>(self) -> ::std::collections::HashMap<&'static str, ::ext_php_rs::props::Property<'a, #self_ty>> {
                use ::std::iter::FromIterator;

                ::std::collections::HashMap::from_iter([])
            }

            fn get_constructor(self) -> ::std::option::Option<::ext_php_rs::class::ConstructorMeta<#self_ty>> {
                #constructor
            }
        }
    })
}

pub fn parse_attribute(attr: &Attribute) -> Result<Option<ParsedAttribute>> {
    let name = attr.path.to_token_stream().to_string();
    let meta = attr
        .parse_meta()
        .map_err(|_| anyhow!("Unable to parse attribute."))?;

    Ok(Some(match name.as_ref() {
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
        "this" => ParsedAttribute::This,
        _ => return Ok(None),
    }))
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
