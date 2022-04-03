use std::{collections::HashMap, marker::PhantomData};

use crate::{
    builders::FunctionBuilder,
    class::{ConstructorMeta, RegisteredClass},
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
    fn get_methods(self) -> Vec<FunctionBuilder<'static>>;
    fn get_method_props<'a>(self) -> HashMap<&'static str, Property<'a, T>>;
    fn get_constructor(self) -> Option<ConstructorMeta<T>>;
}

// Default implementation for classes without an `impl` block. Classes that do
// have an `impl` block will override this by implementing `PhpClassImpl` for
// `PhpClassImplCollector<ClassName>` (note the missing reference). This is
// `dtolnay` specialisation: https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md
impl<T: RegisteredClass> PhpClassImpl<T> for &'_ PhpClassImplCollector<T> {
    #[inline]
    fn get_methods(self) -> Vec<FunctionBuilder<'static>> {
        println!("&get_methods");
        Default::default()
    }

    #[inline]
    fn get_method_props<'a>(self) -> HashMap<&'static str, Property<'a, T>> {
        println!("&get_method_props");
        Default::default()
    }

    #[inline]
    fn get_constructor(self) -> Option<ConstructorMeta<T>> {
        println!("&get_constructor");
        Default::default()
    }
}
