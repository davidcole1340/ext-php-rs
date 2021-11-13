use crate::flags::DataType;
use std::collections::HashMap;

use super::{
    Class, DocBlock, Function, Method, MethodType, Module, Parameter, Property, Visibility,
};
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

        // To account for namespaces we need to group by them. [`None`] as the key
        // represents no namespace, while [`Some`] represents a namespace.
        let mut entries: HashMap<Option<&str>, Vec<String>> = HashMap::new();

        // Inserts a value into the entries hashmap. Takes a key and an entry, creating
        // the internal vector if it doesn't already exist.
        let mut insert = |ns, entry| {
            let bucket = entries.entry(ns).or_insert_with(Vec::new);
            bucket.push(entry);
        };

        for func in &self.functions {
            let (ns, _) = split_namespace(func.name.as_ref());
            insert(ns, func.to_stub()?);
        }

        for class in &self.classes {
            let (ns, _) = split_namespace(class.name.as_ref());
            insert(ns, class.to_stub()?);
        }

        buf.push_str(
            &entries
                .iter()
                .map(|(ns, entries)| {
                    let mut buf = String::new();
                    if let Some(ns) = ns {
                        writeln!(buf, "namespace {} {{", ns)?;
                    } else {
                        writeln!(buf, "namespace {{")?;
                    }

                    buf.push_str(
                        &entries
                            .iter()
                            .map(|entry| indent(entry, 4))
                            .collect::<Vec<_>>()
                            .join(NEW_LINE_SEPARATOR),
                    );

                    writeln!(buf, "}}")?;
                    Ok(buf)
                })
                .collect::<Result<Vec<_>, FmtError>>()?
                .join(NEW_LINE_SEPARATOR),
        );

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

        writeln!(buf, " {{}}")
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

        fn stub<T: ToStub>(items: &[T]) -> impl Iterator<Item = Result<String, FmtError>> + '_ {
            items
                .iter()
                .map(|item| item.to_stub().map(|stub| indent(&stub, 4)))
        }

        buf.push_str(
            &stub(&self.properties)
                .chain(stub(&self.methods))
                .collect::<Result<Vec<_>, FmtError>>()?
                .join(NEW_LINE_SEPARATOR),
        );

        writeln!(buf, "}}")
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
        writeln!(buf, ";")
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

        writeln!(buf, " {{}}")
    }
}

#[cfg(windows)]
const NEW_LINE_SEPARATOR: &str = "\r\n";
#[cfg(not(windows))]
const NEW_LINE_SEPARATOR: &str = "\n";

/// Takes a class name and splits the namespace off from the actual class name.
///
/// # Returns
///
/// A tuple, where the first item is the namespace (or [`None`] if not
/// namespaced), and the second item is the class name.
fn split_namespace(class: &str) -> (Option<&str>, &str) {
    let idx = class.rfind('\\');

    if let Some(idx) = idx {
        (Some(&class[0..idx]), &class[idx + 1..])
    } else {
        (None, class)
    }
}

/// Indents a given string to a given depth. Depth is given in number of spaces
/// to be appended. Returns a new string with the new indentation. Will not
/// indent whitespace lines.
///
/// # Paramters
///
/// * `s` - The string to indent.
/// * `depth` - The depth to indent the lines to, in spaces.
///
/// # Returns
///
/// The indented string.
fn indent(s: &str, depth: usize) -> String {
    let indent = format!("{:depth$}", "", depth = depth);

    s.split('\n')
        .map(|line| {
            let mut result = String::new();
            if line.chars().any(|c| !c.is_whitespace()) {
                result.push_str(&indent);
                result.push_str(line);
            }
            result
        })
        .collect::<Vec<_>>()
        .join(NEW_LINE_SEPARATOR)
}

#[cfg(test)]
mod test {
    use super::{indent, split_namespace};

    #[test]
    pub fn test_split_ns() {
        assert_eq!(split_namespace("ext\\php\\rs"), (Some("ext\\php"), "rs"));
        assert_eq!(split_namespace("test_solo_ns"), (None, "test_solo_ns"));
        assert_eq!(split_namespace("simple\\ns"), (Some("simple"), "ns"));
    }

    #[test]
    pub fn test_indent() {
        assert_eq!(indent("hello", 4), "    hello");
        assert_eq!(indent("hello\nworld\n", 4), "    hello\n    world\n");
    }
}
