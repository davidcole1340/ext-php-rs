use std::collections::HashMap;

use darling::ast::NestedMeta;
use darling::{FromMeta, ToTokens};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::parse::ParseStream;
use syn::{Attribute, Expr, Fields, ItemStruct, ItemTrait, Lit, LitStr, Meta, Token};

use crate::function::{Args, CallType, Function, MethodReceiver};
use crate::helpers::get_docs;
use crate::prelude::*;

#[derive(Debug)]
enum MethodVis {
    Public,
    Private,
    Protected,
}

#[derive(Debug)]
struct MethodArgs {
    name: String,
    optional: Option<Ident>,
    defaults: HashMap<Ident, Lit>,
    vis: MethodVis,
}

#[derive(Debug, Default, FromMeta)]
#[darling(default)]
struct InterfaceArgs {
    name: Option<String>,
}

#[derive(Debug, Default)]
struct InterfaceAttrs {
    implements: Vec<syn::Expr>,
    docs: Vec<String>,
}

impl InterfaceAttrs {
    fn parse(&mut self, attrs: &mut Vec<syn::Attribute>) -> Result<()> {
        let mut unparsed = vec![];
        unparsed.append(attrs);
        for attr in unparsed {
            let path = attr.path();

            if path.is_ident("implements") {
                let implements: syn::Expr = match attr.parse_args() {
                    Ok(extends) => extends,
                    Err(_) => bail!(attr => "Invalid arguments passed to implements attribute."),
                };
                self.implements.push(implements);
            }
        }
        self.docs = get_docs(attrs);
        Ok(())
    }
}

impl MethodArgs {
    fn new(name: String) -> Self {
        Self {
            name,
            optional: Default::default(),
            defaults: Default::default(),
            vis: MethodVis::Public,
        }
    }

    fn parse(&mut self, attrs: &mut Vec<syn::Attribute>) -> Result<()> {
        let mut unparsed = vec![];
        unparsed.append(attrs);
        for attr in unparsed {
            let path = &attr.path();

            if path.is_ident("optional") {
                if self.optional.is_some() {
                    bail!(attr => "Only one `#[optional]` attribute is valid per method.");
                }
                let optional = attr.parse_args().map_err(
                    |e| err!(e.span() => "Invalid arguments passed to `#[optional]` attribute. {}", e),
                )?;
                self.optional = Some(optional);
            } else if path.is_ident("defaults") {
                let defaults = HashMap::from_meta(&attr.meta).map_err(
                    |e| err!(e.span() => "Invalid arguments passed to `#[defaults]` attribute. {}", e),
                )?;
                self.defaults = defaults;
            } else if path.is_ident("public") {
                self.vis = MethodVis::Public;
            } else if path.is_ident("protected") {
                self.vis = MethodVis::Protected;
            } else if path.is_ident("private") {
                self.vis = MethodVis::Private;
            } else {
                attrs.push(attr);
            }
        }
        Ok(())
    }
}

pub fn parser(args: TokenStream, mut input: ItemTrait) -> Result<TokenStream> {
    let meta = NestedMeta::parse_meta_list(args)?;
    let args = match InterfaceArgs::from_list(&meta) {
        Ok(args) => args,
        Err(e) => bail!(input => "Failed to parse impl attribute arguments: {:?}", e),
    };

    let mut parsed = ParsedTrait { functions: vec![] };
    parsed.parse(input.items.iter_mut())?;
    let interface_struct_name = format_ident!("PhpInterface{}", input.ident);
    let functions = &parsed.functions;

    Ok(quote::quote! {
        #input

        pub(crate) struct #interface_struct_name;

        fn get_methods() -> ::std::vec::Vec<
            (::ext_php_rs::builders::FunctionBuilder<'static>, ::ext_php_rs::flags::MethodFlags)
        > {
            vec![#(#functions),*]
        }
    })
}

#[derive(Debug)]
struct ParsedTrait {
    functions: Vec<FnBuilder>,
}

impl ParsedTrait {
    fn parse<'a>(&mut self, items: impl Iterator<Item = &'a mut syn::TraitItem>) -> Result<()> {
        for item in items {
            match item {
                syn::TraitItem::Fn(method) => {
                    let name = method.sig.ident.to_string();
                    let docs = get_docs(&method.attrs);
                    let mut opts = MethodArgs::new(name);
                    opts.parse(&mut method.attrs)?;

                    let args = Args::parse_from_fnargs(method.sig.inputs.iter(), opts.defaults)?;
                    let func =
                        Function::new(&method.sig, Some(opts.name), args, opts.optional, docs)?;

                    let builder = func.function_builder(CallType::MethodInterface)?;
                    self.functions.push(FnBuilder {
                        builder,
                        vis: opts.vis,
                    });
                }
                _ => todo!(),
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct FnBuilder {
    pub builder: TokenStream,
    pub vis: MethodVis,
}

impl quote::ToTokens for FnBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let builder = &self.builder;
        // TODO(cole_d): allow more flags via attributes
        let mut flags = vec![];
        flags.push(match self.vis {
            MethodVis::Public => quote! { ::ext_php_rs::flags::MethodFlags::Public },
            MethodVis::Protected => quote! { ::ext_php_rs::flags::MethodFlags::Protected },
            MethodVis::Private => quote! { ::ext_php_rs::flags::MethodFlags::Private },
        });
        flags.push(quote! { ::ext_php_rs::flags::MethodFlags::Abstract });

        quote! {
            (#builder, #(#flags)|*)
        }
        .to_tokens(tokens);
    }
}
