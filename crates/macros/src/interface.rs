use std::collections::HashMap;

use crate::{
    function::{Args, CallType, Function, MethodReceiver},
    helpers::get_docs,
    impl_::{Constant, FnBuilder, MethodArgs, MethodTy},
    prelude::*,
};
use darling::{ast::NestedMeta, FromMeta, ToTokens};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, ItemTrait, Lit, TraitItem, TraitItemFn};

#[derive(Debug, Default, FromMeta)]
#[darling(default)]
pub struct StructArgs {
    /// The name of the PHP interface. Defaults to the same name as the trait.
    name: Option<String>,
}

#[derive(Debug)]
struct ParsedTrait<'a> {
    path: &'a syn::Path,
    constructor: Option<Function<'a>>,
    functions: Vec<FnBuilder>,
    constants: Vec<Constant<'a>>,
}

impl<'a> ParsedTrait<'a> {
    fn parse(&mut self, items: impl Iterator<Item = &'a mut syn::TraitItem>) -> Result<()> {
        for item in items {
            match item {
                syn::TraitItem::Fn(method) => {
                    let name = method.sig.ident.to_string();
                    let docs = get_docs(&method.attrs);
                    let mut opts = MethodArgs::new(name);
                    opts.parse(&mut method.attrs)?;
                    let args = Args::parse_from_fnargs(method.sig.inputs.iter(), opts.defaults)?;
                    let mut func =
                        Function::new(&method.sig, Some(opts.name), args, opts.optional, docs)?;

                    if matches!(opts.ty, MethodTy::Constructor) {
                        if self.constructor.replace(func).is_some() {
                            bail!(method => "Only one constructor can be provided per class.");
                        }
                    } else {
                        let call_type = CallType::Method {
                            class: self.path,
                            receiver: if func.args.receiver.is_some() {
                                // `&self` or `&mut self`
                                MethodReceiver::Class
                            } else if func
                                .args
                                .typed
                                .first()
                                .map(|arg| arg.name == "self_")
                                .unwrap_or_default()
                            {
                                // `self_: &[mut] ZendClassObject<Self>`
                                // Need to remove arg from argument list
                                func.args.typed.pop();
                                MethodReceiver::ZendClassObject
                            } else {
                                // Static method
                                MethodReceiver::Static
                            },
                        };
                        let builder = func.function_builder(call_type)?;
                        self.functions.push(FnBuilder {
                            builder,
                            vis: opts.vis,
                            r#abstract: true,
                        });
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

pub fn parser(args: TokenStream, input: ItemTrait) -> Result<TokenStream> {
    let meta = NestedMeta::parse_meta_list(args)?;

    let args = match StructArgs::from_list(&meta) {
        Ok(args) => args,
        Err(e) => bail!("Failed to parse struct arguments: {:?}", e),
    };

    Ok(quote! {})
}
