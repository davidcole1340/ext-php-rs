use std::convert::TryFrom;

use darling::FromAttributes;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Expr, Fields, Ident, ItemEnum, Lit};

use crate::{
    helpers::get_docs,
    parsing::{PhpRename, RenameRule, Visibility},
    prelude::*,
};

#[derive(FromAttributes, Default, Debug)]
#[darling(default, attributes(php), forward_attrs(doc))]
struct PhpEnumAttribute {
    #[darling(flatten)]
    rename: PhpRename,
    #[darling(default)]
    allow_discriminants: bool,
    rename_cases: Option<RenameRule>,
    vis: Option<Visibility>,
    attrs: Vec<syn::Attribute>,
}

#[derive(FromAttributes, Default, Debug)]
#[darling(default, attributes(php), forward_attrs(doc))]
struct PhpEnumVariantAttribute {
    #[darling(flatten)]
    rename: PhpRename,
    discriminant: Option<Expr>,
    // TODO: Implement doc support for enum variants
    #[allow(dead_code)]
    attrs: Vec<syn::Attribute>,
}

pub fn parser(mut input: ItemEnum) -> Result<TokenStream> {
    let php_attr = PhpEnumAttribute::from_attributes(&input.attrs)?;
    input.attrs.retain(|attr| !attr.path().is_ident("php"));

    let docs = get_docs(&php_attr.attrs)?;
    let mut cases = vec![];
    let mut discriminant_type = DiscriminantType::None;

    for variant in &mut input.variants {
        if variant.fields != Fields::Unit {
            bail!("Enum cases must be unit variants, found: {:?}", variant);
        }
        if !php_attr.allow_discriminants && variant.discriminant.is_some() {
            bail!(variant => "Native discriminants are currently not exported to PHP. To set a discriminant, use the `#[php(allow_discriminants)]` attribute on the enum. To export discriminants, set the #[php(discriminant = ...)] attribute on the enum case.");
        }

        let variant_attr = PhpEnumVariantAttribute::from_attributes(&variant.attrs)?;
        variant.attrs.retain(|attr| !attr.path().is_ident("php"));
        let docs = get_docs(&variant_attr.attrs)?;
        let discriminant = variant_attr
            .discriminant
            .as_ref()
            .map(TryInto::try_into)
            .transpose()?;

        if let Some(d) = &discriminant {
            match d {
                Discriminant::String(_) => {
                    if discriminant_type == DiscriminantType::Integer {
                        bail!(variant => "Mixed discriminants are not allowed in enums, found string and integer discriminants");
                    }

                    discriminant_type = DiscriminantType::String;
                }
                Discriminant::Integer(_) => {
                    if discriminant_type == DiscriminantType::String {
                        bail!(variant => "Mixed discriminants are not allowed in enums, found string and integer discriminants");
                    }

                    discriminant_type = DiscriminantType::Integer;
                }
            }
        } else if discriminant_type != DiscriminantType::None {
            bail!(variant => "Discriminant must be specified for all enum cases, found: {:?}", variant);
        }

        cases.push(EnumCase {
            ident: variant.ident.clone(),
            name: variant_attr.rename.rename(
                variant.ident.to_string(),
                php_attr.rename_cases.unwrap_or(RenameRule::Pascal),
            ),
            attrs: variant_attr,
            discriminant,
            docs,
        });

        if !cases
            .iter()
            .filter_map(|case| case.discriminant.as_ref())
            .all_unique()
        {
            bail!(variant => "Enum cases must have unique discriminants, found duplicates in: {:?}", cases);
        }
    }

    let enum_props = Enum {
        ident: &input.ident,
        attrs: php_attr,
        docs,
        cases,
        flags: None, // TODO: Implement flags support
    };

    Ok(quote! {
        #[allow(dead_code)]
        #input

        #enum_props
    })
}

#[derive(Debug)]
pub struct Enum<'a> {
    ident: &'a Ident,
    attrs: PhpEnumAttribute,
    docs: Vec<String>,
    cases: Vec<EnumCase>,
    // TODO: Implement flags support
    #[allow(dead_code)]
    flags: Option<String>,
}

impl ToTokens for Enum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        let enum_name = self
            .attrs
            .rename
            .rename(ident.to_string(), RenameRule::Pascal);
        let flags = quote! { ::ext_php_rs::flags::ClassFlags::Enum };
        let docs = &self.docs;
        let cases = &self.cases;

        let class = quote! {
            impl ::ext_php_rs::class::RegisteredClass for #ident {
                const CLASS_NAME: &'static str = #enum_name;
                const BUILDER_MODIFIER: ::std::option::Option<
                    fn(::ext_php_rs::builders::ClassBuilder) -> ::ext_php_rs::builders::ClassBuilder
                > = None;
                const EXTENDS: ::std::option::Option<
                    ::ext_php_rs::class::ClassEntryInfo
                > = None;
                const IMPLEMENTS: &'static [::ext_php_rs::class::ClassEntryInfo] = &[];
                const FLAGS: ::ext_php_rs::flags::ClassFlags = #flags;
                const DOC_COMMENTS: &'static [&'static str] = &[
                    #(#docs,)*
                ];

                fn get_metadata() -> &'static ::ext_php_rs::class::ClassMetadata<Self> {
                    static METADATA: ::ext_php_rs::class::ClassMetadata<#ident> =
                        ::ext_php_rs::class::ClassMetadata::new();
                    &METADATA
                }

                #[inline]
                fn get_properties<'a>() -> ::std::collections::HashMap<
                    &'static str, ::ext_php_rs::internal::property::PropertyInfo<'a, Self>
                > {
                    ::std::collections::HashMap::new()
                }

                #[inline]
                fn method_builders() -> ::std::vec::Vec<
                    (::ext_php_rs::builders::FunctionBuilder<'static>, ::ext_php_rs::flags::MethodFlags)
                > {
                    use ::ext_php_rs::internal::class::PhpClassImpl;
                    ::ext_php_rs::internal::class::PhpClassImplCollector::<Self>::default().get_methods()
                }

                #[inline]
                fn constructor() -> ::std::option::Option<::ext_php_rs::class::ConstructorMeta<Self>> {
                    None
                }

                #[inline]
                fn constants() -> &'static [(&'static str, &'static dyn ::ext_php_rs::convert::IntoZvalDyn, &'static [&'static str])] {
                    use ::ext_php_rs::internal::class::PhpClassImpl;
                    ::ext_php_rs::internal::class::PhpClassImplCollector::<Self>::default().get_constants()
                }
            }
        };
        let enum_impl = quote! {
            impl ::ext_php_rs::enum_::PhpEnum for #ident {
                const CASES: &'static [::ext_php_rs::enum_::EnumCase] = &[
                    #(#cases,)*
                ];
            }
        };

        tokens.extend(quote! {
            #class
            #enum_impl
        });
    }
}

#[derive(Debug)]
struct EnumCase {
    #[allow(dead_code)]
    ident: Ident,
    name: String,
    #[allow(dead_code)]
    attrs: PhpEnumVariantAttribute,
    discriminant: Option<Discriminant>,
    docs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Discriminant {
    String(String),
    Integer(i64),
}

impl TryFrom<&Expr> for Discriminant {
    type Error = syn::Error;

    fn try_from(expr: &Expr) -> Result<Self> {
        match expr {
            Expr::Lit(expr) => match &expr.lit {
                Lit::Str(s) => Ok(Discriminant::String(s.value())),
                Lit::Int(i) => i.base10_parse::<i64>().map(Discriminant::Integer).map_err(
                    |_| err!(expr => "Invalid integer literal for enum case: {:?}", expr.lit),
                ),
                _ => bail!(expr => "Unsupported discriminant type: {:?}", expr.lit),
            },
            _ => {
                bail!(expr => "Unsupported discriminant type, expected a literal of type string or i64, found: {:?}", expr);
            }
        }
    }
}

impl ToTokens for Discriminant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            Discriminant::String(s) => {
                quote! { ::ext_php_rs::enum_::Discriminant::String(#s.to_string()) }
            }
            Discriminant::Integer(i) => {
                quote! { ::ext_php_rs::enum_::Discriminant::Int(#i) }
            }
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DiscriminantType {
    None,
    String,
    Integer,
}

impl ToTokens for EnumCase {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.name;
        let discriminant = self
            .discriminant
            .as_ref()
            .map_or_else(|| quote! { None }, |v| quote! { Some(#v) });
        let docs = &self.docs;

        tokens.extend(quote! {
            ::ext_php_rs::enum_::EnumCase {
                name: #ident,
                discriminant: #discriminant,
                docs: &[#(#docs,)*],
            }
        });
    }
}
