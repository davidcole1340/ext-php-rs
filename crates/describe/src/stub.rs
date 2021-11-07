use ext_php_rs::flags::DataType;

use crate::{Class, Function, Method, MethodType, Module, Parameter, Property, Visibility};
use std::fmt::{Error as FmtError, Result as FmtResult, Write};

pub trait ToStub {
    fn to_stub(&self) -> Result<String, FmtError> {
        let mut buf = String::new();
        self.fmt_stub(&mut buf)?;
        Ok(buf)
    }

    fn fmt_stub(&self, buf: &mut String) -> FmtResult;
}

impl ToStub for Module {
    fn fmt_stub(&self, buf: &mut String) -> FmtResult {
        writeln!(buf, "<?php")?;
        writeln!(buf)?;
        writeln!(buf, "// Stubs for {}", self.name)?;
        writeln!(buf)?;

        for funct in &self.functions {
            funct.fmt_stub(buf)?;
            writeln!(buf)?;
        }

        for class in &self.classes {
            class.fmt_stub(buf)?;
            writeln!(buf)?;
        }

        Ok(())
    }
}

impl ToStub for Function {
    fn fmt_stub(&self, buf: &mut String) -> FmtResult {
        write!(
            buf,
            "function {}({})",
            self.name,
            self.params
                .iter()
                .map(ToStub::to_stub)
                .collect::<Result<Vec<_>, FmtError>>()?
                .join(", ")
        )?;

        if let Some(retval) = &self.ret {
            write!(buf, ": ")?;
            if retval.nullable {
                write!(buf, "?")?;
            }
            retval.ty.fmt_stub(buf)?;
        }

        write!(buf, " {{}}")
    }
}

impl ToStub for Parameter {
    fn fmt_stub(&self, buf: &mut String) -> FmtResult {
        if let Some(ty) = &self.ty {
            if self.nullable {
                write!(buf, "?")?;
            }

            ty.fmt_stub(buf)?;
            write!(buf, " ")?;
        }

        write!(buf, "${}", self.name)
    }
}

impl ToStub for DataType {
    fn fmt_stub(&self, buf: &mut String) -> FmtResult {
        write!(
            buf,
            "{}",
            match self {
                DataType::True | DataType::False => "bool",
                DataType::Long => "int",
                DataType::Double => "float",
                DataType::String => "string",
                DataType::Array => "array",
                DataType::Object(Some(ty)) => ty,
                DataType::Object(None) => "object",
                DataType::Resource => "resource",
                DataType::Reference => "reference",
                DataType::Callable => "callable",
                DataType::Bool => "bool",
                _ => "mixed",
            }
        )
    }
}

impl ToStub for Class {
    fn fmt_stub(&self, buf: &mut String) -> FmtResult {
        write!(buf, "class {} ", self.name)?;

        if let Some(extends) = &self.extends {
            write!(buf, "extends {} ", extends)?;
        }

        if !self.implements.is_empty() {
            write!(buf, "implements {} ", self.implements.join(", "))?;
        }

        writeln!(buf, "{{")?;

        for prop in self.properties.iter() {
            prop.fmt_stub(buf)?;
            writeln!(buf)?;
        }

        for method in self.methods.iter() {
            method.fmt_stub(buf)?;
            writeln!(buf)?;
        }

        write!(buf, "}}")
    }
}

impl ToStub for Property {
    fn fmt_stub(&self, buf: &mut String) -> FmtResult {
        self.vis.fmt_stub(buf)?;
        if self.static_ {
            write!(buf, "static ")?;
        }
        if let Some(ty) = &self.ty {
            ty.fmt_stub(buf)?;
        }
        write!(buf, "${}", self.name)?;
        if let Some(default) = &self.default {
            write!(buf, " = {}", default)?;
        }
        write!(buf, ";")
    }
}

impl ToStub for Visibility {
    fn fmt_stub(&self, buf: &mut String) -> FmtResult {
        write!(
            buf,
            "{}",
            match self {
                Visibility::Private => "private",
                Visibility::Protected => "protected",
                Visibility::Public => "public",
            }
        )
    }
}

impl ToStub for Method {
    fn fmt_stub(&self, buf: &mut String) -> FmtResult {
        self.visibility.fmt_stub(buf)?;
        write!(buf, " ")?;

        if matches!(self.ty, MethodType::Static) {
            write!(buf, "static ")?;
        }

        write!(
            buf,
            "function {}({})",
            self.name,
            self.params
                .iter()
                .map(ToStub::to_stub)
                .collect::<Result<Vec<_>, FmtError>>()?
                .join(", ")
        )?;

        if !matches!(self.ty, MethodType::Constructor) {
            if let Some(retval) = &self.retval {
                write!(buf, ": ")?;
                if retval.nullable {
                    write!(buf, "?")?;
                }
                retval.ty.fmt_stub(buf)?;
            }
        }

        write!(buf, " {{}}")
    }
}
