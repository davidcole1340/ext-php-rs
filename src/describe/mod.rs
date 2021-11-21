//! Types used to describe downstream extensions. Used by the `cargo-php`
//! CLI application to generate PHP stub files used by IDEs.

mod stub;

use crate::flags::DataType;
use std::borrow::Cow;

pub use stub::ToStub;

#[repr(C)]
#[derive(Debug)]
pub struct Description {
    /// Extension description.
    pub module: Module,
    /// ext-php-rs version.
    pub version: &'static str,
}

impl Description {
    /// Creates a new description.
    ///
    /// # Parameters
    ///
    /// * `module` - The extension module representation.
    pub fn new(module: Module) -> Self {
        Self {
            module,
            version: crate::VERSION,
        }
    }
}

/// Represents an extension containing a set of exports.
#[repr(C)]
#[derive(Debug)]
pub struct Module {
    pub name: Cow<'static, str>,
    pub functions: Vec<Function>,
    pub classes: Vec<Class>,
    pub constants: Vec<Constant>,
}

/// Represents a set of comments on an export.
#[repr(C)]
#[derive(Debug)]
pub struct DocBlock(pub Vec<Cow<'static, str>>);

/// Represents an exported function.
#[repr(C)]
#[derive(Debug)]
pub struct Function {
    pub name: Cow<'static, str>,
    pub docs: DocBlock,
    pub ret: Option<Retval>,
    pub params: Vec<Parameter>,
}

/// Represents a parameter attached to an exported function or method.
#[repr(C)]
#[derive(Debug)]
pub struct Parameter {
    pub name: Cow<'static, str>,
    pub ty: Option<DataType>,
    pub nullable: bool,
    pub default: Option<Cow<'static, str>>,
}

/// Represents an exported class.
#[repr(C)]
#[derive(Debug)]
pub struct Class {
    pub name: Cow<'static, str>,
    pub docs: DocBlock,
    pub extends: Option<Cow<'static, str>>,
    pub implements: Vec<Cow<'static, str>>,
    pub properties: Vec<Property>,
    pub methods: Vec<Method>,
    pub constants: Vec<Constant>,
}

/// Represents a property attached to an exported class.
#[repr(C)]
#[derive(Debug)]
pub struct Property {
    pub name: Cow<'static, str>,
    pub docs: DocBlock,
    pub ty: Option<DataType>,
    pub vis: Visibility,
    pub static_: bool,
    pub nullable: bool,
    pub default: Option<Cow<'static, str>>,
}

/// Represents a method attached to an exported class.
#[repr(C)]
#[derive(Debug)]
pub struct Method {
    pub name: Cow<'static, str>,
    pub docs: DocBlock,
    pub ty: MethodType,
    pub params: Vec<Parameter>,
    pub retval: Option<Retval>,
    pub _static: bool,
    pub visibility: Visibility,
}

/// Represents a value returned from a function or method.
#[repr(C)]
#[derive(Debug)]
pub struct Retval {
    pub ty: DataType,
    pub nullable: bool,
}

/// Enumerator used to differentiate between methods.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum MethodType {
    Member,
    Static,
    Constructor,
}

/// Enumerator used to differentiate between different method and property
/// visibilties.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum Visibility {
    Private,
    Protected,
    Public,
}

/// Represents an exported constant, stand alone or attached to a class.
#[repr(C)]
#[derive(Debug)]
pub struct Constant {
    pub name: Cow<'static, str>,
    pub docs: DocBlock,
    pub value: Option<Cow<'static, str>>,
}
