use std::collections::HashMap;

use anyhow::{anyhow, Result};
use darling::FromMeta;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    spanned::Spanned as _, Attribute, AttributeArgs, Ident, Item, ItemConst, ItemFn, ItemImpl,
    ItemStruct, NestedMeta,
};

use crate::{
    class::{self, Class},
    constant::{self, Constant},
    function, impl_,
    module::{self, generate_registered_class_impl},
    startup_function,
};

#[derive(Default)]
pub(crate) struct ModuleBuilder {
    pub functions: Vec<ItemFn>,
    pub startup_function: Option<ItemFn>,
    pub constants: Vec<ItemConst>,
    pub classes: Vec<ItemStruct>,
    pub implementations: Vec<ItemImpl>,
    pub unmapped: Vec<Item>,
}

impl ModuleBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_function(&mut self, function: ItemFn) {
        self.functions.push(function);
    }

    pub fn set_startup_function(&mut self, function: ItemFn) {
        self.startup_function = Some(function);
    }

    pub fn add_constant(&mut self, constant: ItemConst) {
        self.constants.push(constant);
    }

    pub fn add_class(&mut self, class: ItemStruct) {
        self.classes.push(class);
    }

    pub fn add_implementation(&mut self, implementation: ItemImpl) {
        self.implementations.push(implementation);
    }

    pub fn add_unmapped(&mut self, item: Item) {
        self.unmapped.push(item);
    }

    pub fn build(&self) -> TokenStream {
        let (class_stream, mut classes) = self.build_classes();
        let impl_stream = &self
            .implementations
            .iter()
            .map(|implementation| {
                let args = implementation
                    .attrs
                    .iter()
                    .find(|attr| attr.path.is_ident("php_impl"));
                let args = parse_metadata(args.unwrap());
                impl_::parser(args, implementation.clone(), &mut classes)
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let (function_stream, functions) = self.build_functions();
        let (constant_stream, constants) = self.build_constants();
        let (startup_function, startup_ident) =
            self.build_startup_function(&classes, &constants).unwrap();

        let describe_fn = module::generate_stubs(&functions, &classes, &constants);

        let functions = functions
            .iter()
            .map(|func| func.get_builder())
            .collect::<Vec<_>>();
        let registered_classes_impls = classes
            .values()
            .map(generate_registered_class_impl)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let unmapped = &self.unmapped;

        quote! {
            mod module {
                use ::ext_php_rs::prelude::*;
                #class_stream
                #(#impl_stream)*
                #(#registered_classes_impls)*
                #function_stream
                #constant_stream
                #startup_function
                #(#unmapped)*

                #[doc(hidden)]
                #[no_mangle]
                pub extern "C" fn get_module() -> *mut ::ext_php_rs::zend::ModuleEntry {
                    // fn internal(#inputs) #output {
                    //     #(#stmts)*
                    // }

                    let mut builder = ::ext_php_rs::builders::ModuleBuilder::new(
                        env!("CARGO_PKG_NAME"),
                        env!("CARGO_PKG_VERSION")
                    )
                    .startup_function(#startup_ident)
                    #(.function(#functions.unwrap()))*
                    ;

                    // TODO allow result return types
                    // let builder = internal(builder);

                    match builder.build() {
                        Ok(module) => module.into_raw(),
                        Err(e) => panic!("Failed to build PHP module: {:?}", e),
                    }
                }

                #describe_fn
            }
        }
    }

    fn build_functions(&self) -> (TokenStream, Vec<function::Function>) {
        let functions = self
            .functions
            .iter()
            .map(|f| {
                let attr = f.attrs.iter().find(|a| a.path.is_ident("php_function"));
                let args = parse_attr(attr.unwrap()).unwrap();
                function::parser(args, f)
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let tokens = functions.iter().map(|(tokens, _)| tokens);

        (
            quote! { #(#tokens)* },
            functions.into_iter().map(|(_, f)| f).collect(),
        )
    }

    fn build_constants(&self) -> (TokenStream, Vec<Constant>) {
        let constants = self
            .constants
            .iter()
            .map(|c| constant::parser(&mut c.clone()))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let tokens = constants.iter().map(|(tokens, _)| tokens);

        (
            quote! { #(#tokens)* },
            constants.into_iter().map(|(_, c)| c).collect(),
        )
    }

    fn build_classes(&self) -> (TokenStream, HashMap<String, Class>) {
        let structs = self
            .classes
            .iter()
            .map(|class| {
                let args = class
                    .attrs
                    .iter()
                    .find(|attr| attr.path.is_ident("php_class"));
                let args = parse_metadata(args.unwrap());
                class::parser(args, class.clone())
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let tokens = structs.iter().map(|(tokens, _, _)| tokens);

        (
            quote! { #(#tokens)* },
            structs
                .into_iter()
                .map(|(_, name, class)| (name, class))
                .collect(),
        )
    }

    fn build_startup_function(
        &self,
        classes: &HashMap<String, Class>,
        constants: &[Constant],
    ) -> Result<(TokenStream, Ident)> {
        self.startup_function
            .as_ref()
            .map(|f| {
                let attr = f.attrs.iter().find(|a| a.path.is_ident("php_startup"));
                let args = parse_attr(attr.unwrap()).unwrap();
                startup_function::parser(Some(args), f, classes, constants)
            })
            .unwrap_or_else(|| {
                let parsed = syn::parse2(quote! {
                    fn php_module_startup() {}
                })
                .map_err(|_| anyhow!("Unable to generate PHP module startup function."))?;
                startup_function::parser(None, &parsed, classes, constants)
            })
    }
}

fn parse_attr<T>(attr: &Attribute) -> Result<T, TokenStream>
where
    T: FromMeta,
{
    let meta = parse_metadata(attr);

    parse_from_meta(&meta, Some(attr.span()))
}

fn parse_metadata(attr: &Attribute) -> Vec<NestedMeta> {
    if let Ok(args) = attr.parse_args::<TokenStream>().map(|args| args.into()) {
        syn::parse_macro_input::parse::<AttributeArgs>(args).unwrap_or_default()
    } else {
        vec![]
    }
}

fn parse_from_meta<T>(meta: &[NestedMeta], call_site: Option<Span>) -> Result<T, TokenStream>
where
    T: FromMeta,
{
    T::from_list(meta).map_err(|e| {
        syn::Error::new(
            call_site.unwrap_or_else(Span::call_site),
            format!("Unable to parse attribute arguments: {:?}", e),
        )
        .to_compile_error()
    })
}
