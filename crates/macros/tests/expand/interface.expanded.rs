#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2024::*;
#[macro_use]
extern crate std;
use ext_php_rs::types::ZendClassObject;
use ext_php_rs::php_interface;
use ext_php_rs::zend::ce;
pub trait EmptyObjectTrait {
    const HELLO: &'static str = "HELLO";
    const ONE: u64 = 12;
    fn void();
    fn non_static(&self, data: String) -> String;
    fn ref_to_like_this_class(
        &self,
        data: String,
        other: &ZendClassObject<PhpInterfaceEmptyObjectTrait>,
    ) -> String;
}
pub struct PhpInterfaceEmptyObjectTrait;
impl PhpInterfaceEmptyObjectTrait {
    pub const HELLO: &'static str = "HELLO";
    pub const ONE: u64 = 12;
}
impl ::ext_php_rs::class::RegisteredClass for PhpInterfaceEmptyObjectTrait {
    const CLASS_NAME: &'static str = "ExtPhpRs\\Interface\\EmptyObjectInterface";
    const BUILDER_MODIFIER: Option<
        fn(::ext_php_rs::builders::ClassBuilder) -> ::ext_php_rs::builders::ClassBuilder,
    > = None;
    const EXTENDS: Option<::ext_php_rs::class::ClassEntryInfo> = None;
    const FLAGS: ::ext_php_rs::flags::ClassFlags = ::ext_php_rs::flags::ClassFlags::Interface;
    const IMPLEMENTS: &'static [::ext_php_rs::class::ClassEntryInfo] = &[
        (ce::throwable, "\\Throwable"),
    ];
    fn get_metadata() -> &'static ::ext_php_rs::class::ClassMetadata<Self> {
        static METADATA: ::ext_php_rs::class::ClassMetadata<
            PhpInterfaceEmptyObjectTrait,
        > = ::ext_php_rs::class::ClassMetadata::new();
        &METADATA
    }
    fn method_builders() -> Vec<
        (
            ::ext_php_rs::builders::FunctionBuilder<'static>,
            ::ext_php_rs::flags::MethodFlags,
        ),
    > {
        <[_]>::into_vec(
            ::alloc::boxed::box_new([
                (
                    ::ext_php_rs::builders::FunctionBuilder::new_abstract("void")
                        .not_required(),
                    ::ext_php_rs::flags::MethodFlags::Public
                        | ::ext_php_rs::flags::MethodFlags::Abstract
                        | ::ext_php_rs::flags::MethodFlags::Static,
                ),
                (
                    ::ext_php_rs::builders::FunctionBuilder::new_abstract("nonStatic")
                        .arg(
                            ::ext_php_rs::args::Arg::new(
                                "data",
                                <String as ::ext_php_rs::convert::FromZvalMut>::TYPE,
                            ),
                        )
                        .not_required()
                        .returns(
                            <String as ::ext_php_rs::convert::IntoZval>::TYPE,
                            false,
                            <String as ::ext_php_rs::convert::IntoZval>::NULLABLE,
                        ),
                    ::ext_php_rs::flags::MethodFlags::Public
                        | ::ext_php_rs::flags::MethodFlags::Abstract,
                ),
                (
                    ::ext_php_rs::builders::FunctionBuilder::new_abstract(
                            "refToLikeThisClass",
                        )
                        .arg(
                            ::ext_php_rs::args::Arg::new(
                                "data",
                                <String as ::ext_php_rs::convert::FromZvalMut>::TYPE,
                            ),
                        )
                        .arg(
                            ::ext_php_rs::args::Arg::new(
                                "other",
                                <&ZendClassObject<
                                    PhpInterfaceEmptyObjectTrait,
                                > as ::ext_php_rs::convert::FromZvalMut>::TYPE,
                            ),
                        )
                        .not_required()
                        .returns(
                            <String as ::ext_php_rs::convert::IntoZval>::TYPE,
                            false,
                            <String as ::ext_php_rs::convert::IntoZval>::NULLABLE,
                        ),
                    ::ext_php_rs::flags::MethodFlags::Public
                        | ::ext_php_rs::flags::MethodFlags::Abstract,
                ),
            ]),
        )
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
        ::ext_php_rs::internal::class::PhpClassImplCollector::<Self>::default()
            .get_constants()
    }
    fn get_properties<'a>() -> std::collections::HashMap<
        &'static str,
        ::ext_php_rs::internal::property::PropertyInfo<'a, Self>,
    > {
        HashMap::new()
    }
}
impl ::ext_php_rs::internal::class::PhpClassImpl<PhpInterfaceEmptyObjectTrait>
for ::ext_php_rs::internal::class::PhpClassImplCollector<PhpInterfaceEmptyObjectTrait> {
    fn get_methods(
        self,
    ) -> ::std::vec::Vec<
        (
            ::ext_php_rs::builders::FunctionBuilder<'static>,
            ::ext_php_rs::flags::MethodFlags,
        ),
    > {
        ::alloc::vec::Vec::new()
    }
    fn get_method_props<'a>(
        self,
    ) -> ::std::collections::HashMap<
        &'static str,
        ::ext_php_rs::props::Property<'a, PhpInterfaceEmptyObjectTrait>,
    > {
        ::core::panicking::panic("not yet implemented")
    }
    fn get_constructor(
        self,
    ) -> ::std::option::Option<
        ::ext_php_rs::class::ConstructorMeta<PhpInterfaceEmptyObjectTrait>,
    > {
        None
    }
    fn get_constants(
        self,
    ) -> &'static [(
        &'static str,
        &'static dyn ::ext_php_rs::convert::IntoZvalDyn,
        &'static [&'static str],
    )] {
        &[
            ("HELLO", &PhpInterfaceEmptyObjectTrait::HELLO, &[]),
            ("ONE", &PhpInterfaceEmptyObjectTrait::ONE, &[]),
        ]
    }
}
impl<'a> ::ext_php_rs::convert::FromZendObject<'a> for &'a PhpInterfaceEmptyObjectTrait {
    #[inline]
    fn from_zend_object(
        obj: &'a ::ext_php_rs::types::ZendObject,
    ) -> ::ext_php_rs::error::Result<Self> {
        let obj = ::ext_php_rs::types::ZendClassObject::<
            PhpInterfaceEmptyObjectTrait,
        >::from_zend_obj(obj)
            .ok_or(::ext_php_rs::error::Error::InvalidScope)?;
        Ok(&**obj)
    }
}
impl<'a> ::ext_php_rs::convert::FromZendObjectMut<'a>
for &'a mut PhpInterfaceEmptyObjectTrait {
    #[inline]
    fn from_zend_object_mut(
        obj: &'a mut ::ext_php_rs::types::ZendObject,
    ) -> ::ext_php_rs::error::Result<Self> {
        let obj = ::ext_php_rs::types::ZendClassObject::<
            PhpInterfaceEmptyObjectTrait,
        >::from_zend_obj_mut(obj)
            .ok_or(::ext_php_rs::error::Error::InvalidScope)?;
        Ok(&mut **obj)
    }
}
impl<'a> ::ext_php_rs::convert::FromZval<'a> for &'a PhpInterfaceEmptyObjectTrait {
    const TYPE: ::ext_php_rs::flags::DataType = ::ext_php_rs::flags::DataType::Object(
        Some(
            <PhpInterfaceEmptyObjectTrait as ::ext_php_rs::class::RegisteredClass>::CLASS_NAME,
        ),
    );
    #[inline]
    fn from_zval(zval: &'a ::ext_php_rs::types::Zval) -> ::std::option::Option<Self> {
        <Self as ::ext_php_rs::convert::FromZendObject>::from_zend_object(zval.object()?)
            .ok()
    }
}
impl<'a> ::ext_php_rs::convert::FromZvalMut<'a>
for &'a mut PhpInterfaceEmptyObjectTrait {
    const TYPE: ::ext_php_rs::flags::DataType = ::ext_php_rs::flags::DataType::Object(
        Some(
            <PhpInterfaceEmptyObjectTrait as ::ext_php_rs::class::RegisteredClass>::CLASS_NAME,
        ),
    );
    #[inline]
    fn from_zval_mut(
        zval: &'a mut ::ext_php_rs::types::Zval,
    ) -> ::std::option::Option<Self> {
        <Self as ::ext_php_rs::convert::FromZendObjectMut>::from_zend_object_mut(
                zval.object_mut()?,
            )
            .ok()
    }
}
impl ::ext_php_rs::convert::IntoZendObject for PhpInterfaceEmptyObjectTrait {
    #[inline]
    fn into_zend_object(
        self,
    ) -> ::ext_php_rs::error::Result<
        ::ext_php_rs::boxed::ZBox<::ext_php_rs::types::ZendObject>,
    > {
        Ok(::ext_php_rs::types::ZendClassObject::new(self).into())
    }
}
impl ::ext_php_rs::convert::IntoZval for PhpInterfaceEmptyObjectTrait {
    const TYPE: ::ext_php_rs::flags::DataType = ::ext_php_rs::flags::DataType::Object(
        Some(
            <PhpInterfaceEmptyObjectTrait as ::ext_php_rs::class::RegisteredClass>::CLASS_NAME,
        ),
    );
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
