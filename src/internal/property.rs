use crate::{describe::DocComments, flags::PropertyFlags, props::Property};

pub struct PropertyInfo<'a, T> {
    pub prop: Property<'a, T>,
    pub flags: PropertyFlags,
    pub docs: DocComments,
}
