pub mod stub;

use std::borrow::Cow;

use ext_php_rs::flags::DataType;

#[derive(Debug)]
pub struct Module {
    pub name: Cow<'static, str>,
    pub functions: Vec<Function>,
    pub classes: Vec<Class>,
}

impl Module {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: name.into(),
            functions: vec![],
            classes: vec![],
        }
    }
}

#[derive(Debug)]
pub struct DocBlock(pub Vec<Cow<'static, str>>);

#[derive(Debug)]
pub struct Function {
    pub name: Cow<'static, str>,
    pub ret: Option<Retval>,
    pub params: Vec<Parameter>,
}

#[derive(Debug)]
pub struct Parameter {
    pub name: Cow<'static, str>,
    pub ty: Option<DataType>,
    pub nullable: bool,
    pub default: Option<Cow<'static, str>>,
}

#[derive(Debug)]
pub struct Class {
    pub name: Cow<'static, str>,
    pub doc: DocBlock,
    pub extends: Option<Cow<'static, str>>,
    pub implements: Vec<Cow<'static, str>>,
    pub properties: Vec<Property>,
    pub methods: Vec<Method>,
}

#[derive(Debug)]
pub struct Property {
    pub name: Cow<'static, str>,
    pub ty: Option<DataType>,
    pub vis: Visibility,
    pub static_: bool,
    pub nullable: bool,
    pub default: Option<Cow<'static, str>>,
}

#[derive(Debug)]
pub struct Method {
    pub name: Cow<'static, str>,
    pub ty: MethodType,
    pub params: Vec<Parameter>,
    pub retval: Option<Retval>,
    pub _static: bool,
    pub visibility: Visibility,
}

#[derive(Debug)]
pub struct Retval {
    pub ty: DataType,
    pub nullable: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum MethodType {
    Member,
    Static,
    Constructor,
}

#[derive(Debug, Clone, Copy)]
pub enum Visibility {
    Private,
    Protected,
    Public,
}
