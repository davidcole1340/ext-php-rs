use darling::{FromAttributes};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemTrait, TraitItem, TraitItemFn};
use crate::helpers::CleanPhpAttr;

use crate::parsing::{PhpRename, RenameRule};
use crate::prelude::*;

#[derive(FromAttributes, Debug, Default)]
#[darling(attributes(php), forward_attrs(doc), default)]
pub struct StructAttributes {
    #[darling(flatten)]
    rename: PhpRename,
}

pub fn parser(mut input: ItemTrait) -> Result<TokenStream> {
    let attr = StructAttributes::from_attributes(&input.attrs)?;
    let ident = &input.ident;

    let interface_name = format_ident!("PhpInterface{ident}");
    let name = attr.rename.rename(ident.to_string(), RenameRule::Pascal);
    input.attrs.clean_php();

    let mut interface_methods: Vec<TraitItemFn> = Vec::new();
    for i in input.items.clone().into_iter() {
        match i {
            TraitItem::Fn(f) => {
                if f.default.is_some() {
                    bail!("Interface could not have default impl");
                }
                interface_methods.push(f);
            }
            _ => {}
        }
    };

    Ok(quote! {
        #input

        pub struct #interface_name;

        impl ::ext_php_rs::class::RegisteredClass for #interface_name {
            const CLASS_NAME: &'static str = #name;

            const BUILDER_MODIFIER: Option<
            fn(::ext_php_rs::builders::ClassBuilder) -> ::ext_php_rs::builders::ClassBuilder,
            > = None;

            const EXTENDS: Option<::ext_php_rs::class::ClassEntryInfo> = None;

            const FLAGS: ::ext_php_rs::flags::ClassFlags = ::ext_php_rs::flags::ClassFlags::Interface;

            const IMPLEMENTS: &'static [::ext_php_rs::class::ClassEntryInfo] = &[];

            fn get_metadata() -> &'static ::ext_php_rs::class::ClassMetadata<Self> {
                static METADATA: ::ext_php_rs::class::ClassMetadata<#interface_name> =
                ::ext_php_rs::class::ClassMetadata::new();

                &METADATA
            }

            fn method_builders() -> Vec<(
                ::ext_php_rs::builders::FunctionBuilder<'static>,
                ::ext_php_rs::flags::MethodFlags,
            )> {
                vec![ ]
            }

            fn constructor() -> Option<::ext_php_rs::class::ConstructorMeta<Self>> {
                None
            }

            fn constants() -> &'static [(
                &'static str,
                &'static dyn ext_php_rs::convert::IntoZvalDyn,
                ext_php_rs::describe::DocComments,
            )] {
                &[]
            }

            fn get_properties<'a>() -> std::collections::HashMap<&'static str, ::ext_php_rs::internal::property::PropertyInfo<'a, Self>> {
                HashMap::new()
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
