use std::collections::{HashMap, HashSet};

use darling::util::Flag;
use darling::{FromAttributes};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Expr, Ident, ItemTrait, Path, TraitItem, TraitItemConst, TraitItemFn};
use crate::class::ClassEntryAttribute;
use crate::constant::PhpConstAttribute;
use crate::function::{Args, Function};
use crate::helpers::{get_docs, CleanPhpAttr};

use crate::impl_::{Constant, FnBuilder, MethodModifier};
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
    let attr = StructAttributes::from_attributes(&input.attrs)?;
    let ident = &input.ident;

    let interface_name = format_ident!("PhpInterface{ident}");
    let ts = quote! { #interface_name };
    let path: Path = syn::parse2(ts)?;

    let name = attr.rename.rename(ident.to_string(), RenameRule::Pascal);
    input.attrs.clean_php();

    let methods: Vec<FnBuilder> = input.items.iter_mut()
        .flat_map(
            |item: &mut TraitItem| {
                match item {
                    TraitItem::Fn(f) => Some(f),
                    _ => None,
                }
            })
        .flat_map(|f| f.parse())
        .collect();

    let constants: Vec<_> = input.items.iter_mut()
        .flat_map(|item: &mut TraitItem| {
            match item {
                TraitItem::Const(c) => Some(c),
                _ => None,
            }
        })
        .flat_map(|c| c.parse())
        .map(|c| {
            let name = &c.name;
            let ident = c.ident;
            let docs = &c.docs;
            quote! {
                (#name, &#path::#ident, &[#(#docs),*])
            }
        })
        .collect();

    let impl_const: Vec<&TraitItemConst> = input.items.iter().flat_map(|item| {
        match item {
            TraitItem::Const(c) => Some(c),
            _ => None,
        }
    })
    .map(|c| {
            if c.default.is_none() {
                bail!("Interface const cannot be empty");
            }
            Ok(c)
        })
        .flat_map(|c| c)
        .collect();

    let implements = attr.extends;

    Ok(quote! {
        #input

        pub struct #interface_name;

        impl #interface_name {
            #(pub #impl_const)*
        }

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
                vec![#(#methods),*]
            }

            fn constructor() -> Option<::ext_php_rs::class::ConstructorMeta<Self>> {
                None
            }

            fn constants() -> &'static [(
                &'static str,
                &'static dyn ext_php_rs::convert::IntoZvalDyn,
                ext_php_rs::describe::DocComments,
            )] {
                use ::ext_php_rs::internal::class::PhpClassImpl;
                ::ext_php_rs::internal::class::PhpClassImplCollector::<Self>::default().get_constants()
            }

            fn get_properties<'a>() -> std::collections::HashMap<&'static str, ::ext_php_rs::internal::property::PropertyInfo<'a, Self>> {
                HashMap::new()
            }

        }
            impl ::ext_php_rs::internal::class::PhpClassImpl<#path>
                for ::ext_php_rs::internal::class::PhpClassImplCollector<#path>
            {
                fn get_methods(self) -> ::std::vec::Vec<
                    (::ext_php_rs::builders::FunctionBuilder<'static>, ::ext_php_rs::flags::MethodFlags)
                > {
                    vec![]
                }

                fn get_method_props<'a>(self) -> ::std::collections::HashMap<&'static str, ::ext_php_rs::props::Property<'a, #path>> {
                    todo!()
                }

                fn get_constructor(self) -> ::std::option::Option<::ext_php_rs::class::ConstructorMeta<#path>> {
                    None
                }

                fn get_constants(self) -> &'static [(&'static str, &'static dyn ::ext_php_rs::convert::IntoZvalDyn, &'static [&'static str])] {
                    &[#(#constants),*]
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
    })
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

trait Parse<'a, T> {
    fn parse(&'a mut self) -> Result<T>;
}

impl<'a> Parse<'a, Constant<'a>> for TraitItemConst {
    fn parse(&'a mut self) -> Result<Constant<'a>> {
        let attr = PhpConstAttribute::from_attributes(&self.attrs)?;
        let name = self.ident.to_string();
        let docs = get_docs(&attr.attrs)?;
        self.attrs.clean_php();

        Ok(Constant::new(name, &self.ident, docs))
    }
}

impl<'a> Parse<'a, FnBuilder> for TraitItemFn {
    fn parse(&'a mut self) -> Result<FnBuilder> {
        let php_attr = PhpFunctionInterfaceAttribute::from_attributes(
            &self.attrs
        )?;
        if self.default.is_some() {
            bail!("Interface could not have default impl");
        }

        let mut args = Args::parse_from_fnargs(
            self.sig.inputs.iter(),
            php_attr.defaults
        )?;
        let docs = get_docs(&php_attr.attrs)?;

        self.attrs.clean_php();

        let mut modifiers: HashSet<MethodModifier> = HashSet::new();
        modifiers.insert(MethodModifier::Abstract);
        if args.typed.first().is_some_and(|arg| arg.name == "self_") {
            args.typed.pop();
        } else if args.receiver.is_none() {
            modifiers.insert(MethodModifier::Static);
        };

        let f = Function::new(
            &self.sig,
            php_attr
                .rename
                .rename(self.sig.ident.to_string(), RenameRule::Camel),
            args,
            php_attr.optional,
            docs,
        );

        Ok(FnBuilder {
            builder: f.abstract_function_builder(),
            vis: php_attr.vis.unwrap_or(Visibility::Public),
            modifiers,
        })
    }
}
