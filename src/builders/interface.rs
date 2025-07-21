use crate::builders::FunctionBuilder;
use crate::flags::{ClassFlags, MethodFlags};
use crate::{builders::ClassBuilder, class::ClassEntryInfo, convert::IntoZvalDyn, describe::DocComments};
use crate::error::Result;

pub struct InterfaceBuilder {
    class_builder: ClassBuilder,
}

impl InterfaceBuilder {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Self {
            class_builder: ClassBuilder::new(name),
        }
    }

    pub fn implements(mut self, interface: ClassEntryInfo) -> Self {
        self.class_builder = self.class_builder.implements(interface);

        self
    }

    pub fn method(mut self, func: FunctionBuilder<'static>, flags: MethodFlags) -> Self {
        self.class_builder = self.class_builder.method(func, flags);

        self
    }

    pub fn dyn_constant<T: Into<String>>(
        mut self,
        name: T,
        value: &'static dyn IntoZvalDyn,
        docs: DocComments,
    ) -> Result<Self> {
        self.class_builder = self.class_builder.dyn_constant(name, value, docs)?;

        Ok(self)
    }

    pub fn builder(mut self) -> ClassBuilder {
        self.class_builder = self.class_builder.flags(ClassFlags::Interface);
        self.class_builder
    }
}

