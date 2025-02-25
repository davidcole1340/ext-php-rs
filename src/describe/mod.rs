//! Types used to describe downstream extensions. Used by the `cargo-php`
//! CLI application to generate PHP stub files used by IDEs.
use bitflags::bitflags_match;
use std::vec::Vec as StdVec;

use crate::{
    builders::{ClassBuilder, FunctionBuilder},
    constant::IntoConst,
    flags::{DataType, MethodFlags, PropertyFlags},
    prelude::ModuleBuilder,
};
use abi::*;

pub mod abi;
mod stub;

pub use stub::ToStub;

/// A slice of strings containing documentation comments.
pub type DocComments = &'static [&'static str];

/// Representation of the extension used to generate PHP stubs.
#[repr(C)]
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

/// Represents a set of comments on an export.
#[repr(C)]
pub struct DocBlock(pub Vec<Str>);

impl From<&'static [&'static str]> for DocBlock {
    fn from(val: &'static [&'static str]) -> Self {
        Self(
            val.iter()
                .map(|s| (*s).into())
                .collect::<StdVec<_>>()
                .into(),
        )
    }
}

/// Represents an extension containing a set of exports.
#[repr(C)]
pub struct Module {
    /// Name of the extension.
    pub name: RString,
    /// Functions exported by the extension.
    pub functions: Vec<Function>,
    /// Classes exported by the extension.
    pub classes: Vec<Class>,
    /// Constants exported by the extension.
    pub constants: Vec<Constant>,
}

/// Builds a [`Module`] from a [`ModuleBuilder`].
/// This is used to generate the PHP stubs for the module.
impl From<ModuleBuilder<'_>> for Module {
    fn from(builder: ModuleBuilder) -> Self {
        let functions = builder.functions;
        Self {
            name: builder.name.into(),
            functions: functions
                .into_iter()
                .map(Function::from)
                .collect::<StdVec<_>>()
                .into(),
            classes: builder
                .classes
                .into_iter()
                .map(|c| c().into())
                .collect::<StdVec<_>>()
                .into(),
            constants: builder
                .constants
                .into_iter()
                .map(Constant::from)
                .collect::<StdVec<_>>()
                .into(),
        }
    }
}

/// Represents an exported function.
#[repr(C)]
pub struct Function {
    /// Name of the function.
    pub name: RString,
    /// Documentation comments for the function.
    pub docs: DocBlock,
    /// Return value of the function.
    pub ret: Option<Retval>,
    /// Parameters of the function.
    pub params: Vec<Parameter>,
}

impl From<FunctionBuilder<'_>> for Function {
    fn from(val: FunctionBuilder<'_>) -> Self {
        let ret_allow_null = val.ret_as_null;
        Function {
            name: val.name.into(),
            docs: DocBlock(
                val.docs
                    .iter()
                    .map(|d| (*d).into())
                    .collect::<StdVec<_>>()
                    .into(),
            ),
            ret: val
                .retval
                .map(|r| Retval {
                    ty: r,
                    nullable: r != DataType::Mixed && ret_allow_null,
                })
                .into(),
            params: val
                .args
                .into_iter()
                .map(Parameter::from)
                .collect::<StdVec<_>>()
                .into(),
        }
    }
}

/// Represents a parameter attached to an exported function or method.
#[repr(C)]
pub struct Parameter {
    /// Name of the parameter.
    pub name: RString,
    /// Type of the parameter.
    pub ty: Option<DataType>,
    /// Whether the parameter is nullable.
    pub nullable: bool,
    /// Default value of the parameter.
    pub default: Option<RString>,
}

/// Represents an exported class.
#[repr(C)]
pub struct Class {
    /// Name of the class.
    pub name: RString,
    /// Documentation comments for the class.
    pub docs: DocBlock,
    /// Name of the class the exported class extends. (Not implemented #326)
    pub extends: Option<RString>,
    /// Names of the interfaces the exported class implements. (Not implemented #326)
    pub implements: Vec<RString>,
    /// Properties of the class.
    pub properties: Vec<Property>,
    /// Methods of the class.
    pub methods: Vec<Method>,
    /// Constants of the class.
    pub constants: Vec<Constant>,
}

impl From<ClassBuilder> for Class {
    fn from(val: ClassBuilder) -> Self {
        Self {
            name: val.name.into(),
            docs: DocBlock(
                val.docs
                    .iter()
                    .map(|doc| (*doc).into())
                    .collect::<StdVec<_>>()
                    .into(),
            ),
            extends: abi::Option::None, // TODO: Implement extends #326
            implements: vec![].into(),  // TODO: Implement implements #326
            properties: val
                .properties
                .into_iter()
                .map(Property::from)
                .collect::<StdVec<_>>()
                .into(),
            methods: val
                .methods
                .into_iter()
                .map(Method::from)
                .collect::<StdVec<_>>()
                .into(),
            constants: val
                .constants
                .into_iter()
                .map(|(name, _, docs)| (name, docs))
                .map(Constant::from)
                .collect::<StdVec<_>>()
                .into(),
        }
    }
}

/// Represents a property attached to an exported class.
#[repr(C)]
pub struct Property {
    /// Name of the property.
    pub name: RString,
    /// Documentation comments for the property.
    pub docs: DocBlock,
    /// Type of the property (Not implemented #376)
    pub ty: Option<DataType>,
    /// Visibility of the property.
    pub vis: Visibility,
    /// Whether the property is static.
    pub static_: bool,
    /// Whether the property is nullable. (Not implemented #376)
    pub nullable: bool,
    /// Default value of the property. (Not implemented #376)
    pub default: Option<RString>,
}

impl From<(String, PropertyFlags, DocComments)> for Property {
    fn from(value: (String, PropertyFlags, DocComments)) -> Self {
        let (name, flags, docs) = value;
        let static_ = flags.contains(PropertyFlags::Static);
        let vis = Visibility::from(flags);
        // TODO: Implement ty #376
        let ty = abi::Option::None;
        // TODO: Implement default #376
        let default = abi::Option::<abi::RString>::None;
        // TODO: Implement nullable #376
        let nullable = false;
        let docs = docs.into();
        println!("Property: {:?}", name);
        Self {
            name: name.into(),
            docs,
            ty,
            vis,
            static_,
            nullable,
            default,
        }
    }
}

/// Represents a method attached to an exported class.
#[repr(C)]
pub struct Method {
    /// Name of the method.
    pub name: RString,
    /// Documentation comments for the method.
    pub docs: DocBlock,
    /// Type of the method.
    pub ty: MethodType,
    /// Parameters of the method.
    pub params: Vec<Parameter>,
    /// Return value of the method.
    pub retval: Option<Retval>,
    /// Whether the method is static.
    pub _static: bool,
    /// Visibility of the method.
    pub visibility: Visibility,
}

impl From<(FunctionBuilder<'_>, MethodFlags)> for Method {
    fn from(val: (FunctionBuilder<'_>, MethodFlags)) -> Self {
        let (builder, flags) = val;
        let ret_allow_null = builder.ret_as_null;
        Method {
            name: builder.name.into(),
            docs: DocBlock(
                builder
                    .docs
                    .iter()
                    .map(|d| (*d).into())
                    .collect::<StdVec<_>>()
                    .into(),
            ),
            retval: builder
                .retval
                .map(|r| Retval {
                    ty: r,
                    nullable: r != DataType::Mixed && ret_allow_null,
                })
                .into(),
            params: builder
                .args
                .into_iter()
                .map(|a| a.into())
                .collect::<StdVec<_>>()
                .into(),
            ty: flags.into(),
            _static: flags.contains(MethodFlags::Static),
            visibility: flags.into(),
        }
    }
}

/// Represents a value returned from a function or method.
#[repr(C)]
pub struct Retval {
    /// Type of the return value.
    pub ty: DataType,
    /// Whether the return value is nullable.
    pub nullable: bool,
}

/// Enumerator used to differentiate between methods.
#[repr(C)]
#[derive(Clone, Copy)]
pub enum MethodType {
    /// A member method.
    Member,
    /// A static method.
    Static,
    /// A constructor.
    Constructor,
}

impl From<MethodFlags> for MethodType {
    fn from(value: MethodFlags) -> Self {
        match value {
            MethodFlags::Static => Self::Static,
            MethodFlags::IsConstructor => Self::Constructor,
            _ => Self::Member,
        }
    }
}

/// Enumerator used to differentiate between different method and property
/// visibilties.
#[repr(C)]
#[derive(Clone, Copy)]
pub enum Visibility {
    /// Private visibility.
    Private,
    /// Protected visibility.
    Protected,
    /// Public visibility.
    Public,
}

impl From<PropertyFlags> for Visibility {
    fn from(value: PropertyFlags) -> Self {
        bitflags_match!(value, {
            PropertyFlags::Public => Visibility::Public,
            PropertyFlags::Protected => Visibility::Protected,
            PropertyFlags::Private => Visibility::Private,
            _ => Visibility::Public,
        })
    }
}

impl From<MethodFlags> for Visibility {
    fn from(value: MethodFlags) -> Self {
        bitflags_match!(value, {
            MethodFlags::Public => Self::Public,
            MethodFlags::Protected => Self::Protected,
            MethodFlags::Private => Self::Private,
            _ => Self::Public,
        })
    }
}

/// Represents an exported constant, stand alone or attached to a class.
#[repr(C)]
pub struct Constant {
    /// Name of the constant.
    pub name: RString,
    /// Documentation comments for the constant.
    pub docs: DocBlock,
    /// Value of the constant.
    pub value: Option<RString>,
}

impl From<(String, DocComments)> for Constant {
    fn from(val: (String, DocComments)) -> Self {
        let (name, docs) = val;
        Constant {
            name: name.into(),
            value: abi::Option::None,
            docs: docs.into(),
        }
    }
}

impl From<(String, Box<dyn IntoConst + Send>, DocComments)> for Constant {
    fn from(val: (String, Box<dyn IntoConst + Send + 'static>, DocComments)) -> Self {
        let (name, _, docs) = val;
        Constant {
            name: name.into(),
            value: abi::Option::None,
            docs: docs.into(),
        }
    }
}
