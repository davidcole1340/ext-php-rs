use std::{collections::HashMap, marker::PhantomData};

use crate::{
    builders::FunctionBuilder,
    class::{ConstructorMeta, RegisteredClass},
    convert::{IntoZval, IntoZvalDyn},
    flags::MethodFlags,
    props::Property,
};

/// Collector used to collect methods for PHP classes.
pub struct PhpClassImplCollector<T: RegisteredClass>(PhantomData<T>);

impl<T: RegisteredClass> Default for PhpClassImplCollector<T> {
    #[inline]
    fn default() -> Self {
        Self(PhantomData)
    }
}

pub trait PhpClassImpl<T: RegisteredClass> {
    fn get_methods(self) -> Vec<(FunctionBuilder<'static>, MethodFlags)>;
    fn get_method_props<'a>(self) -> HashMap<&'static str, Property<'a, T>>;
    fn get_constructor(self) -> Option<ConstructorMeta<T>>;
    fn get_constants(self) -> &'static [(&'static str, &'static dyn IntoZvalDyn)];
}

// Default implementation for classes without an `impl` block. Classes that do
// have an `impl` block will override this by implementing `PhpClassImpl` for
// `PhpClassImplCollector<ClassName>` (note the missing reference). This is
// `dtolnay` specialisation: https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md
impl<T: RegisteredClass> PhpClassImpl<T> for &'_ PhpClassImplCollector<T> {
    #[inline]
    fn get_methods(self) -> Vec<(FunctionBuilder<'static>, MethodFlags)> {
        Default::default()
    }

    #[inline]
    fn get_method_props<'a>(self) -> HashMap<&'static str, Property<'a, T>> {
        Default::default()
    }

    #[inline]
    fn get_constructor(self) -> Option<ConstructorMeta<T>> {
        Default::default()
    }

    #[inline]
    fn get_constants(self) -> &'static [(&'static str, &'static dyn IntoZvalDyn)] {
        &[]
    }
}

// This implementation is only used for `TYPE` and `NULLABLE`.
impl<T: RegisteredClass + IntoZval> IntoZval for PhpClassImplCollector<T> {
    const TYPE: crate::flags::DataType = T::TYPE;
    const NULLABLE: bool = T::NULLABLE;

    #[inline]
    fn set_zval(self, _: &mut crate::types::Zval, _: bool) -> crate::error::Result<()> {
        unreachable!();
    }
}
