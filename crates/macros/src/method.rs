use crate::function::{Args, FnArgs, MethodReceiver};
use crate::prelude::*;
use crate::{function::Function, helpers::GetDocs};
use darling::ast::NestedMeta;
use darling::FromMeta;
use proc_macro2::TokenStream;
use syn::{FnArg, Ident, ImplItemFn, ItemFn, TraitItemFn};

// pub enum FunctionLikeType {
//     Interface(TraitItemFn),
//     Impl(ImplItemFn),
//     Function(ItemFn),
// }

pub trait FunctionLike: GetDocs {
    fn name(&self) -> String;

    fn args(&self) -> impl Iterator<Item = &FnArg>;

    fn signature(&self) -> &syn::Signature;
}

pub trait MethodLike: FunctionLike {}

// TraitMethod to Interface >>>
impl GetDocs for TraitItemFn {
    fn get_docs(&self) -> Vec<String> {
        self.attrs.as_slice().get_docs()
    }
}

impl FunctionLike for TraitItemFn {
    fn name(&self) -> String {
        self.sig.ident.to_string()
    }

    fn args(&self) -> impl Iterator<Item = &FnArg> {
        self.sig.inputs.iter()
    }

    fn signature(&self) -> &syn::Signature {
        &self.sig
    }
}

impl MethodLike for TraitItemFn {}
// <<< TraitMethod to Interface

// ImplMethod to class method >>>
impl GetDocs for ImplItemFn {
    fn get_docs(&self) -> Vec<String> {
        self.attrs.as_slice().get_docs()
    }
}

impl FunctionLike for ImplItemFn {
    fn name(&self) -> String {
        self.sig.ident.to_string()
    }

    fn args(&self) -> impl Iterator<Item = &FnArg> {
        self.sig.inputs.iter()
    }

    fn signature(&self) -> &syn::Signature {
        &self.sig
    }
}

impl MethodLike for ImplItemFn {}
// <<< ImplMethod to class method

// Function to function >>>
impl GetDocs for ItemFn {
    fn get_docs(&self) -> Vec<String> {
        self.attrs.as_slice().get_docs()
    }
}

impl FunctionLike for ItemFn {
    fn name(&self) -> String {
        self.sig.ident.to_string()
    }

    fn args(&self) -> impl Iterator<Item = &FnArg> {
        self.sig.inputs.iter()
    }

    fn signature(&self) -> &syn::Signature {
        &self.sig
    }
}
// Function to function >>>

pub trait ToFunction<'a> {
    fn to_function(&'a self, opts: TokenStream) -> Result<Function<'a>>;
}

impl<'a, T: FunctionLike> ToFunction<'a> for T {
    fn to_function(&'a self, opts: TokenStream) -> Result<Function<'a>> {
        let meta = NestedMeta::parse_meta_list(opts)?;
        let opts = match FnArgs::from_list(&meta) {
            Ok(opts) => opts,
            Err(e) => bail!("Failed to parse attribute options: {:?}", e),
        };

        let args = Args::parse_from_fnargs(self.args(), opts.defaults)?;

        let docs = self.get_docs();

        Function::new(self.signature(), opts.name, args, opts.optional, docs)
    }
}

//-------------------

#[derive(Debug)]
enum MethodVis {
    Public,
    Private,
    Protected,
}

#[derive(Debug)]
enum MethodTy {
    Normal,
    Constructor,
    Getter,
    Setter,
    Abstract,
}

#[derive(Debug)]
pub struct MethodArgs {
    fn_args: FnArgs,
    vis: MethodVis,
    ty: MethodTy,
}

pub trait ToMethod<'a> {
    fn to_method(&'a self, opts: TokenStream) -> Result<Function<'a>>;
}

impl<'a, T: MethodLike> ToMethod<'a> for T {
    fn to_method(&'a self, opts: TokenStream) -> Result<Function<'a>> {
        let meta = NestedMeta::parse_meta_list(opts)?;
        let opts = match FnArgs::from_list(&meta) {
            Ok(opts) => opts,
            Err(e) => bail!("Failed to parse attribute options: {:?}", e),
        };
        todo!()
    }
}
