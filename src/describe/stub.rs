//! Traits and implementations to convert describe units into PHP stub code.

use crate::flags::DataType;
use std::{cmp::Ordering, collections::HashMap};

use super::{
    abi::*, Class, Constant, DocBlock, Function, Method, MethodType, Module, Parameter, Property,
    Visibility,
};
use std::fmt::{Error as FmtError, Result as FmtResult, Write};
use std::{option::Option as StdOption, vec::Vec as StdVec};

/// Implemented on types which can be converted into PHP stubs.
pub trait ToStub {
    /// Converts the implementor into PHP code, represented as a PHP stub.
    /// Returned as a string.
    ///
    /// # Returns
    ///
    /// Returns a string on success. Returns an error if there was an error
    /// writing into the string.
    fn to_stub(&self) -> Result<String, FmtError> {
        let mut buf = String::new();
        self.fmt_stub(&mut buf)?;
        Ok(buf)
    }

    /// Converts the implementor into PHP code, represented as a PHP stub.
    ///
    /// # Parameters
    ///
    /// * `buf` - The buffer to write the PHP code into.
    ///
    /// # Returns
    ///
    /// Returns nothing on success. Returns an error if there was an error
    /// writing into the buffer.
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
        let mut entries: HashMap<StdOption<&str>, StdVec<String>> = HashMap::new();

        // Inserts a value into the entries hashmap. Takes a key and an entry, creating
        // the internal vector if it doesn't already exist.
        let mut insert = |ns, entry| {
            let bucket = entries.entry(ns).or_insert_with(StdVec::new);
            bucket.push(entry);
        };

        for c in &*self.constants {
            let (ns, _) = split_namespace(c.name.as_ref());
            insert(ns, c.to_stub()?);
        }

        for func in &*self.functions {
            let (ns, _) = split_namespace(func.name.as_ref());
            insert(ns, func.to_stub()?);
        }

        for class in &*self.classes {
            let (ns, _) = split_namespace(class.name.as_ref());
            insert(ns, class.to_stub()?);
        }

        let mut entries: StdVec<_> = entries.iter().collect();
        entries.sort_by(|(l, _), (r, _)| match (l, r) {
            (None, _) => Ordering::Greater,
            (_, None) => Ordering::Less,
            (Some(l), Some(r)) => l.cmp(r),
        });

        buf.push_str(
            &entries
                .into_iter()
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
                            .collect::<StdVec<_>>()
                            .join(NEW_LINE_SEPARATOR),
                    );

                    writeln!(buf, "}}")?;
                    Ok(buf)
                })
                .collect::<Result<StdVec<_>, FmtError>>()?
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
                .collect::<Result<StdVec<_>, FmtError>>()?
                .join(", ")
        )?;

        if let Option::Some(retval) = &self.ret {
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
        if let Option::Some(ty) = &self.ty {
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

        if let Option::Some(extends) = &self.extends {
            write!(buf, "extends {} ", extends)?;
        }

        if !self.implements.is_empty() {
            write!(
                buf,
                "implements {} ",
                self.implements
                    .iter()
                    .map(|s| s.str())
                    .collect::<StdVec<_>>()
                    .join(", ")
            )?;
        }

        writeln!(buf, "{{")?;

        fn stub<T: ToStub>(items: &[T]) -> impl Iterator<Item = Result<String, FmtError>> + '_ {
            items
                .iter()
                .map(|item| item.to_stub().map(|stub| indent(&stub, 4)))
        }

        buf.push_str(
            &stub(&self.constants)
                .chain(stub(&self.properties))
                .chain(stub(&self.methods))
                .collect::<Result<StdVec<_>, FmtError>>()?
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
        if let Option::Some(ty) = &self.ty {
            ty.fmt_stub(buf)?;
        }
        write!(buf, "${}", self.name)?;
        if let Option::Some(default) = &self.default {
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
                .collect::<Result<StdVec<_>, FmtError>>()?
                .join(", ")
        )?;

        if !matches!(self.ty, MethodType::Constructor) {
            if let Option::Some(retval) = &self.retval {
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

impl ToStub for Constant {
    fn fmt_stub(&self, buf: &mut String) -> FmtResult {
        self.docs.fmt_stub(buf)?;

        write!(buf, "const {} = ", self.name)?;
        if let Option::Some(value) = &self.value {
            write!(buf, "{}", value)?;
        } else {
            write!(buf, "null")?;
        }
        writeln!(buf, ";")
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
fn split_namespace(class: &str) -> (StdOption<&str>, &str) {
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
        .collect::<StdVec<_>>()
        .join(NEW_LINE_SEPARATOR)
}

#[cfg(test)]
mod test {
    use super::split_namespace;

    #[test]
    pub fn test_split_ns() {
        assert_eq!(split_namespace("ext\\php\\rs"), (Some("ext\\php"), "rs"));
        assert_eq!(split_namespace("test_solo_ns"), (None, "test_solo_ns"));
        assert_eq!(split_namespace("simple\\ns"), (Some("simple"), "ns"));
    }

    #[test]
    #[cfg(not(windows))]
    pub fn test_indent() {
        use super::indent;
        use crate::describe::stub::NEW_LINE_SEPARATOR;

        assert_eq!(indent("hello", 4), "    hello");
        assert_eq!(
            indent(&format!("hello{nl}world{nl}", nl = NEW_LINE_SEPARATOR), 4),
            format!("    hello{nl}    world{nl}", nl = NEW_LINE_SEPARATOR)
        );
    }
}
