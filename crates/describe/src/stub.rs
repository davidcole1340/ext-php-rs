use std::collections::HashMap;
use ext_php_rs::flags::DataType;

use crate::{Class, DocBlock, Function, Method, MethodType, Module, Parameter, Property, Visibility};
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

        // To account for namespaces we need to group by them. [`None`] as the key represents no
        // namespace, while [`Some`] represents a namespace.
        let mut entries: HashMap<Option<&str>, Vec<String>> = HashMap::new();

        // Inserts a value into the entries hashmap. Takes a key and an entry, creating the internal
        // vector if it doesn't already exist.
        let mut insert = |ns, entry| {
            let bucket = entries.entry(ns).or_insert_with(|| Vec::new());
            bucket.push(entry);
        };

        for func in &self.functions {
            let (ns, name) = split_namespace(func.name.as_ref());
            insert(ns, func.to_stub()?);
        }

        for class in &self.classes {
            let (ns, name) = split_namespace(class.name.as_ref());
            insert(ns, class.to_stub()?);
        }

        for (ns, entries) in entries.iter() {
            if let Some(ns) = ns {
                writeln!(buf, "namespace {} {{", ns)?;
            } else {
                writeln!(buf, "namespace {{")?;
            }

            for entry in entries.iter() {
                writeln!(buf, "{}", entry)?;
            }

            writeln!(buf, "}}")?; // /ns
        }

        Ok(())
    }
}

impl ToStub for Function {
    fn fmt_stub(&self, buf: &mut String) -> FmtResult {
        self.docs.fmt_stub(buf)?;

        let (_, name) = split_namespace(self.name.as_ref());
        write!(
            buf,
            "function {}({})",
            name,
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

impl ToStub for DocBlock {
    fn fmt_stub(&self, buf: &mut String) -> FmtResult {
        if !self.0.is_empty() {
            writeln!(buf, "/**")?;
            for comment in self.0.iter() {
                writeln!(buf, " *{}", comment)?;
            }
            writeln!(buf, " */")?;
        }
        Ok(())
    }
}

impl ToStub for Class {
    fn fmt_stub(&self, buf: &mut String) -> FmtResult {
        self.docs.fmt_stub(buf)?;

        let (_, name) = split_namespace(self.name.as_ref());
        write!(buf, "class {} ", name)?;

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
        self.docs.fmt_stub(buf)?;
        self.vis.fmt_stub(buf)?;

        write!(buf, " ")?;

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
        self.docs.fmt_stub(buf)?;
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

/// Takes a class name and splits the namespace off from the actual class name.
///
/// # Returns
///
/// A tuple, where the first item is the namespace (or [`None`] if not namespaced), and the second
/// item is the class name.
fn split_namespace(class: &str) -> (Option<&str>, &str) {
    let idx = class.rfind('\\');

    if let Some(idx) = idx {
        (Some(&class[0..idx]), &class[idx + 1..])
    } else {
        (None, class)
    }
}

mod test {
    use super::split_namespace;

    #[test]
    pub fn test_split_ns() {
        assert_eq!(split_namespace("ext\\php\\rs"), (Some("ext\\php"), "rs"));
        assert_eq!(split_namespace("test_solo_ns"), (None, "test_solo_ns"));
        assert_eq!(split_namespace("simple\\ns"), (Some("simple"), "ns"));
    }
}
