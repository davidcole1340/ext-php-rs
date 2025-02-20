//! Types used to describe downstream extensions. Used by the `cargo-php`
//! CLI application to generate PHP stub files used by IDEs.

pub mod abi;
mod stub;

use crate::{flags::DataType, prelude::ModuleBuilder};
use abi::*;

pub use stub::ToStub;

#[repr(C)]
pub struct Description {
    /// Extension description.
    pub module: ModuleBuilder,
    /// ext-php-rs version.
    pub version: &'static str,
}

impl Description {
    /// Creates a new description.
    ///
    /// # Parameters
    ///
    /// * `module` - The extension module representation.
    pub fn new(module: ModuleBuilder) -> Self {
        Self {
            module,
            version: crate::VERSION,
        }
    }
}

/// Represents an extension containing a set of exports.
#[repr(C)]
pub struct Module {
    pub name: Str,
    pub functions: Vec<Function>,
    pub classes: Vec<Class>,
    pub constants: Vec<Constant>,
}

/// Represents a set of comments on an export.
#[repr(C)]
pub struct DocBlock(pub Vec<Str>);

/// Represents an exported function.
#[repr(C)]
pub struct Function {
    pub name: Str,
    pub docs: DocBlock,
    pub ret: Option<Retval>,
    pub params: Vec<Parameter>,
}

/// Represents a parameter attached to an exported function or method.
#[repr(C)]
pub struct Parameter {
    pub name: Str,
    pub ty: Option<DataType>,
    pub nullable: bool,
    pub default: Option<Str>,
}

/// Represents an exported class.
#[repr(C)]
pub struct Class {
    pub name: Str,
    pub docs: DocBlock,
    pub extends: Option<Str>,
    pub implements: Vec<Str>,
    pub properties: Vec<Property>,
    pub methods: Vec<Method>,
    pub constants: Vec<Constant>,
}

/// Represents a property attached to an exported class.
#[repr(C)]
pub struct Property {
    pub name: Str,
    pub docs: DocBlock,
    pub ty: Option<DataType>,
    pub vis: Visibility,
    pub static_: bool,
    pub nullable: bool,
    pub default: Option<Str>,
}

/// Represents a method attached to an exported class.
#[repr(C)]
pub struct Method {
    pub name: Str,
    pub docs: DocBlock,
    pub ty: MethodType,
    pub params: Vec<Parameter>,
    pub retval: Option<Retval>,
    pub _static: bool,
    pub visibility: Visibility,
}

/// Represents a value returned from a function or method.
#[repr(C)]
pub struct Retval {
    pub ty: DataType,
    pub nullable: bool,
}

/// Enumerator used to differentiate between methods.
#[repr(C)]
#[derive(Clone, Copy)]
pub enum MethodType {
    Member,
    Static,
    Constructor,
}

/// Enumerator used to differentiate between different method and property
/// visibilties.
#[repr(C)]
#[derive(Clone, Copy)]
pub enum Visibility {
    Private,
    Protected,
    Public,
}

/// Represents an exported constant, stand alone or attached to a class.
#[repr(C)]
pub struct Constant {
    pub name: Str,
    pub docs: DocBlock,
    pub value: Option<Str>,
}
