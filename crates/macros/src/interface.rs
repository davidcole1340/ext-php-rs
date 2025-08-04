use std::collections::{HashMap, HashSet};

use crate::class::ClassEntryAttribute;
use crate::constant::PhpConstAttribute;
use crate::function::{Args, Function};
use crate::helpers::{get_docs, CleanPhpAttr};
use darling::util::Flag;
use darling::FromAttributes;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{Expr, Ident, ItemTrait, Path, TraitItem, TraitItemConst, TraitItemFn};

use crate::impl_::{FnBuilder, MethodModifier};
use crate::parsing::{PhpRename, RenameRule, Visibility};
use crate::prelude::*;

#[derive(FromAttributes, Debug, Default)]
#[darling(attributes(php), forward_attrs(doc), default)]
pub struct StructAttributes {
    #[darling(flatten)]
    rename: PhpRename,
    #[darling(multiple)]
    extends: Vec<ClassEntryAttribute>,
}

pub fn parser(mut input: ItemTrait) -> Result<TokenStream> {
    let interface_data: InterfaceData = input.parse()?;
    let interface_tokens = quote! { #interface_data };

    Ok(quote! {
        #input

        #interface_tokens
    })
}

trait Parse<'a, T> {
    fn parse(&'a mut self) -> Result<T>;
}

struct InterfaceData<'a> {
    ident: &'a Ident,
    name: String,
    path: Path,
    attrs: StructAttributes,
    constructor: Option<Function<'a>>,
    methods: Vec<FnBuilder>,
    constants: Vec<Constant<'a>>,
}

impl ToTokens for InterfaceData<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let interface_name = format_ident!("PhpInterface{}", self.ident);
        let name = &self.name;
        let implements = &self.attrs.extends;
        let methods_sig = &self.methods;
        let path = &self.path;
        let constants = &self.constants;

        let constructor = self.constructor
            .as_ref()
            .map(|func| func.constructor_meta(&path))
            .option_tokens();

        quote! {
            pub struct #interface_name;

            impl ::ext_php_rs::class::RegisteredClass for #interface_name {
                const CLASS_NAME: &'static str = #name;

                const BUILDER_MODIFIER: Option<
                fn(::ext_php_rs::builders::ClassBuilder) -> ::ext_php_rs::builders::ClassBuilder,
                > = None;

                const EXTENDS: Option<::ext_php_rs::class::ClassEntryInfo> = None;

                const FLAGS: ::ext_php_rs::flags::ClassFlags = ::ext_php_rs::flags::ClassFlags::Interface;

                const IMPLEMENTS: &'static [::ext_php_rs::class::ClassEntryInfo] = &[
                    #(#implements,)*
                ];

                fn get_metadata() -> &'static ::ext_php_rs::class::ClassMetadata<Self> {
                    static METADATA: ::ext_php_rs::class::ClassMetadata<#interface_name> =
                    ::ext_php_rs::class::ClassMetadata::new();

                    &METADATA
                }

                fn method_builders() -> Vec<(
                    ::ext_php_rs::builders::FunctionBuilder<'static>,
                    ::ext_php_rs::flags::MethodFlags,
                )> {
                    vec![#(#methods_sig),*]
                }

                fn constructor() -> Option<::ext_php_rs::class::ConstructorMeta<Self>> {
                    None
                }

                fn constants() -> &'static [(
                    &'static str,
                    &'static dyn ext_php_rs::convert::IntoZvalDyn,
                    ext_php_rs::describe::DocComments,
                )] {
                    &[#(#constants),*]
                }

                fn get_properties<'a>() -> ::std::collections::HashMap<&'static str, ::ext_php_rs::internal::property::PropertyInfo<'a, Self>> {
                    panic!("Non supported for Interface");
                }
            }

            impl<'a> ::ext_php_rs::convert::FromZendObject<'a> for &'a #interface_name {
                #[inline]
                fn from_zend_object(
                    obj: &'a ::ext_php_rs::types::ZendObject,
                ) -> ::ext_php_rs::error::Result<Self> {
                    let obj = ::ext_php_rs::types::ZendClassObject::<#interface_name>::from_zend_obj(obj)
                        .ok_or(::ext_php_rs::error::Error::InvalidScope)?;
                    Ok(&**obj)
                }
            }
            impl<'a> ::ext_php_rs::convert::FromZendObjectMut<'a> for &'a mut #interface_name {
                #[inline]
                fn from_zend_object_mut(
                    obj: &'a mut ::ext_php_rs::types::ZendObject,
                ) -> ::ext_php_rs::error::Result<Self> {
                    let obj = ::ext_php_rs::types::ZendClassObject::<#interface_name>::from_zend_obj_mut(obj)
                        .ok_or(::ext_php_rs::error::Error::InvalidScope)?;
                    Ok(&mut **obj)
                }
            }
            impl<'a> ::ext_php_rs::convert::FromZval<'a> for &'a #interface_name {
                const TYPE: ::ext_php_rs::flags::DataType = ::ext_php_rs::flags::DataType::Object(Some(
                    <#interface_name as ::ext_php_rs::class::RegisteredClass>::CLASS_NAME,
                ));
                #[inline]
                fn from_zval(zval: &'a ::ext_php_rs::types::Zval) -> ::std::option::Option<Self> {
                    <Self as ::ext_php_rs::convert::FromZendObject>::from_zend_object(zval.object()?).ok()
                }
            }
            impl<'a> ::ext_php_rs::convert::FromZvalMut<'a> for &'a mut #interface_name {
                const TYPE: ::ext_php_rs::flags::DataType = ::ext_php_rs::flags::DataType::Object(Some(
                    <#interface_name as ::ext_php_rs::class::RegisteredClass>::CLASS_NAME,
                ));
                #[inline]
                fn from_zval_mut(zval: &'a mut ::ext_php_rs::types::Zval) -> ::std::option::Option<Self> {
                    <Self as ::ext_php_rs::convert::FromZendObjectMut>::from_zend_object_mut(zval.object_mut()?)
                        .ok()
                }
            }
            impl ::ext_php_rs::convert::IntoZendObject for #interface_name {
                #[inline]
                fn into_zend_object(
                    self,
                ) -> ::ext_php_rs::error::Result<::ext_php_rs::boxed::ZBox<::ext_php_rs::types::ZendObject>>
                {
                    Ok(::ext_php_rs::types::ZendClassObject::new(self).into())
                }
            }
            impl ::ext_php_rs::convert::IntoZval for #interface_name {
                const TYPE: ::ext_php_rs::flags::DataType = ::ext_php_rs::flags::DataType::Object(Some(
                    <#interface_name as ::ext_php_rs::class::RegisteredClass>::CLASS_NAME,
                ));
                const NULLABLE: bool = false;
                #[inline]
                fn set_zval(
                    self,
                    zv: &mut ::ext_php_rs::types::Zval,
                    persistent: bool,
                ) -> ::ext_php_rs::error::Result<()> {
                    use ::ext_php_rs::convert::IntoZendObject;
                    self.into_zend_object()?.set_zval(zv, persistent)
                }
            }
        }.to_tokens(tokens);
    }
}

impl<'a> InterfaceData<'a> {
    fn new(
        ident: &'a Ident,
        name: String,
        path: Path,
        attrs: StructAttributes,
        constructor: Option<Function<'a>>,
        methods: Vec<FnBuilder>,
        constants: Vec<Constant<'a>>,
    ) -> Self {
        Self {
            ident,
            name,
            path,
            attrs,
            constructor,
            methods,
            constants,
        }
    }
}

impl<'a> Parse<'a, InterfaceData<'a>> for ItemTrait {
    fn parse(&'a mut self) -> Result<InterfaceData<'a>> {
        let attrs = StructAttributes::from_attributes(&self.attrs)?;
        let ident = &self.ident;
        let name = attrs.rename.rename(ident.to_string(), RenameRule::Pascal);
        self.attrs.clean_php();
        let interface_name = format_ident!("PhpInterface{ident}");
        let ts = quote! { #interface_name };
        let path: Path = syn::parse2(ts)?;
        let mut data = InterfaceData::new(
            ident,
            name,
            path,
            attrs,
            None,
            Vec::new(),
            Vec::new()
        );

        for item in &mut self.items {
            match item {
                TraitItem::Fn(f) => {
                    match f.parse()? {
                        MethodKind::Method(builder) => data.methods.push(builder),
                        MethodKind::Constructor(builder) => {
                            if data.constructor.replace(builder).is_some() {
                                bail!("Only one constructor can be provided per class.");
                            }
                        }
                    };
                },
                TraitItem::Const(c) => data.constants.push(c.parse()?),
                _ => {}
            }
        }

        Ok(data)
    }
}

#[derive(FromAttributes, Default, Debug)]
#[darling(default, attributes(php), forward_attrs(doc))]
pub struct PhpFunctionInterfaceAttribute {
    #[darling(flatten)]
    rename: PhpRename,
    defaults: HashMap<Ident, Expr>,
    optional: Option<Ident>,
    vis: Option<Visibility>,
    attrs: Vec<syn::Attribute>,
    getter: Flag,
    setter: Flag,
    constructor: Flag,
}

enum MethodKind<'a> {
    Method(FnBuilder),
    Constructor(Function<'a>),
}

impl<'a> Parse<'a, MethodKind<'a>> for TraitItemFn {
    fn parse(&'a mut self) -> Result<MethodKind<'a>> {
        if self.default.is_some() {
            bail!(self => "Interface could not have default impl");
        }

        let php_attr = PhpFunctionInterfaceAttribute::from_attributes(
            &self.attrs
        )?;
        self.attrs.clean_php();

        let mut args = Args::parse_from_fnargs(
            self.sig.inputs.iter(),
            php_attr.defaults
        )?;

        let docs = get_docs(&php_attr.attrs)?;

        let mut modifiers: HashSet<MethodModifier> = HashSet::new();
        modifiers.insert(MethodModifier::Abstract);

        if args.typed.first().is_some_and(|arg| arg.name == "self_") {
            args.typed.pop();
        } else if args.receiver.is_none() {
            modifiers.insert(MethodModifier::Static);
        }

        let f = Function::new(
            &self.sig,
            php_attr
                .rename
                .rename(self.sig.ident.to_string(), RenameRule::Camel),
            args,
            php_attr.optional,
            docs,
        );

        if php_attr.constructor.is_present() {
            Ok(MethodKind::Constructor(f))
        } else {
            let builder = FnBuilder {
                builder: f.abstract_function_builder(),
                vis: php_attr.vis.unwrap_or(Visibility::Public),
                modifiers,
            };

            Ok(MethodKind::Method(builder))
        }
    }
}

impl<'a> Parse<'a, Vec<MethodKind<'a>>> for ItemTrait {
    fn parse(&'a mut self) -> Result<Vec<MethodKind<'a>>> {
        Ok(self
            .items
            .iter_mut()
            .filter_map(|item| match item {
                TraitItem::Fn(f) => Some(f),
                _ => None,
            })
            .flat_map(Parse::parse)
            .collect())
    }
}

#[derive(Debug)]
struct Constant<'a> {
    name: String,
    expr: &'a Expr,
    docs: Vec<String>,
}

impl ToTokens for Constant<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let expr = &self.expr;
        let docs = &self.docs;
        quote! {
            (#name, &#expr, &[#(#docs),*])
        }
        .to_tokens(tokens);
    }
}

impl<'a> Constant<'a> {
    fn new(name: String, expr: &'a Expr, docs: Vec<String>) -> Self {
        Self { name, expr, docs }
    }
}

impl<'a> Parse<'a, Constant<'a>> for TraitItemConst {
    fn parse(&'a mut self) -> Result<Constant<'a>> {
        if self.default.is_none() {
            bail!(self => "Interface const could not be empty");
        }

        let attr = PhpConstAttribute::from_attributes(&self.attrs)?;
        let name = self.ident.to_string();
        let docs = get_docs(&attr.attrs)?;
        self.attrs.clean_php();

        let (_, expr) = self.default.as_ref().unwrap();
        Ok(Constant::new(name, expr, docs))
    }
}

impl<'a> Parse<'a, Vec<Constant<'a>>> for ItemTrait {
    fn parse(&'a mut self) -> Result<Vec<Constant<'a>>> {
        Ok(self
            .items
            .iter_mut()
            .filter_map(|item| match item {
                TraitItem::Const(c) => Some(c),
                _ => None,
            })
            .flat_map(Parse::parse)
            .collect())
    }
}
