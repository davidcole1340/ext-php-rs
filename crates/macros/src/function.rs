use std::collections::HashMap;

use crate::helpers::get_docs;
use crate::{syn_ext::DropLifetimes, STATE};
use anyhow::{anyhow, bail, Result};
use darling::{FromMeta, ToTokens};
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use syn::{
    punctuated::Punctuated, AttributeArgs, FnArg, GenericArgument, ItemFn, Lit, PathArguments,
    ReturnType, Signature, Token, Type, TypePath,
};

#[derive(Default, Debug, FromMeta)]
#[darling(default)]
pub struct AttrArgs {
    optional: Option<String>,
    ignore_module: bool,
    defaults: HashMap<String, Lit>,
    name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub name: String,
    pub ty: String,
    pub nullable: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub docs: Vec<String>,
    pub ident: String,
    pub args: Vec<Arg>,
    pub optional: Option<String>,
    pub output: Option<(String, bool)>,
}

pub fn parser(args: AttributeArgs, input: ItemFn) -> Result<(TokenStream, Function)> {
    let attr_args = match AttrArgs::from_list(&args) {
        Ok(args) => args,
        Err(e) => bail!("Unable to parse attribute arguments: {:?}", e),
    };

    let ItemFn { sig, .. } = &input;
    let Signature {
        ident,
        output,
        inputs,
        ..
    } = &sig;

    let internal_ident = Ident::new(&format!("_internal_php_{}", ident), Span::call_site());
    let args = build_args(inputs, &attr_args.defaults)?;
    let optional = find_optional_parameter(args.iter(), attr_args.optional);
    let arg_definitions = build_arg_definitions(&args);
    let arg_parser = build_arg_parser(
        args.iter(),
        &optional,
        &quote! { return; },
        ParserType::Function,
    )?;
    let arg_accessors = build_arg_accessors(&args);

    let return_type = get_return_type(output)?;

    let func = quote! {
        #input

        ::ext_php_rs::zend_fastcall! {
            #[doc(hidden)]
            pub extern fn #internal_ident(ex: &mut ::ext_php_rs::zend::ExecuteData, retval: &mut ::ext_php_rs::types::Zval) {
                use ::ext_php_rs::convert::IntoZval;

                #(#arg_definitions)*
                #arg_parser

                let result = #ident(#(#arg_accessors, )*);

                if let Err(e) = result.set_zval(retval, false) {
                    let e: ::ext_php_rs::exception::PhpException = e.into();
                    e.throw().expect("Failed to throw exception");
                }
            }
        }
    };

    let mut state = STATE.lock();

    if state.built_module && !attr_args.ignore_module {
        bail!("The `#[php_module]` macro must be called last to ensure functions are registered. To ignore this error, pass the `ignore_module` option into this attribute invocation: `#[php_function(ignore_module)]`");
    }

    let function = Function {
        name: attr_args.name.unwrap_or_else(|| ident.to_string()),
        docs: get_docs(&input.attrs),
        ident: internal_ident.to_string(),
        args,
        optional,
        output: return_type,
    };

    state.functions.push(function.clone());

    Ok((func, function))
}

fn build_args(
    inputs: &Punctuated<FnArg, Token![,]>,
    defaults: &HashMap<String, Lit>,
) -> Result<Vec<Arg>> {
    inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Receiver(_) => bail!(
                "`self` is not permitted in PHP functions. See the `#[php_method]` attribute."
            ),
            FnArg::Typed(ty) => {
                let name = match &*ty.pat {
                    syn::Pat::Ident(pat) => pat.ident.to_string(),
                    _ => bail!("Invalid parameter type."),
                };
                Arg::from_type(name.clone(), &ty.ty, defaults.get(&name), false)
                    .ok_or_else(|| anyhow!("Invalid parameter type for parameter `{}`.", name))
            }
        })
        .collect::<Result<Vec<_>>>()
}

fn build_arg_definitions(args: &[Arg]) -> Vec<TokenStream> {
    args.iter()
        .map(|ty| {
            let ident = ty.get_name_ident();
            let definition = ty.get_arg_definition();
            quote! {
                let mut #ident = #definition;
            }
        })
        .collect()
}

pub fn find_optional_parameter<'a>(
    args: impl DoubleEndedIterator<Item = &'a Arg>,
    optional: Option<String>,
) -> Option<String> {
    if optional.is_some() {
        return optional;
    }

    let mut optional = None;

    for arg in args.rev() {
        if arg.nullable {
            optional.replace(arg.name.clone());
        } else {
            break;
        }
    }

    optional
}

pub enum ParserType {
    Function,
    Method,
    StaticMethod,
}

pub fn build_arg_parser<'a>(
    args: impl Iterator<Item = &'a Arg>,
    optional: &Option<String>,
    ret: &TokenStream,
    ty: ParserType,
) -> Result<TokenStream> {
    let mut rest_optional = false;

    let args = args
        .map(|arg| {
            let name = arg.get_name_ident();
            let prelude = optional.as_ref().and_then(|opt| if *opt == arg.name {
                rest_optional = true;
                Some(quote! { .not_required() })
            } else {
                None
            });

            if rest_optional && !arg.nullable && arg.default.is_none() {
                bail!(
                    "Parameter `{}` must be a variant of `Option` or have a default value as it is optional.",
                    arg.name
                )
            } else {
                Ok(quote! {
                    #prelude
                    .arg(&mut #name)
                })
            }
        })
        .collect::<Result<Vec<_>>>()?;
    let (parser, this) = match ty {
        ParserType::Function | ParserType::StaticMethod => {
            (quote! { let parser = ex.parser(); }, None)
        }
        ParserType::Method => (
            quote! { let (parser, this) = ex.parser_method::<Self>(); },
            Some(quote! {
                let this = match this {
                    Some(this) => this,
                    None => {
                        ::ext_php_rs::exception::PhpException::default("Failed to retrieve reference to `$this`".into())
                            .throw()
                            .unwrap();
                        return;
                    },
                };
            }),
        ),
    };

    Ok(quote! {
        #parser
        let parser = parser
            #(#args)*
            .parse();

        if parser.is_err() {
            #ret
        }

        #this
    })
}

fn build_arg_accessors(args: &[Arg]) -> Vec<TokenStream> {
    args.iter()
        .map(|arg| arg.get_accessor(&quote! { return; }))
        .collect()
}

pub fn get_return_type(output_type: &ReturnType) -> Result<Option<(String, bool)>> {
    Ok(match output_type {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => {
            Arg::from_type("".to_string(), ty, None, true).map(|arg| (arg.ty, arg.nullable))
        }
    })
}

impl Arg {
    pub fn new(name: String, ty: String, nullable: bool, default: Option<String>) -> Self {
        Self {
            name,
            ty,
            nullable,
            default,
        }
    }

    pub fn from_type(
        name: String,
        ty: &syn::Type,
        default: Option<&Lit>,
        is_return: bool,
    ) -> Option<Arg> {
        let default = default.map(|lit| lit.to_token_stream().to_string());
        match ty {
            Type::Path(TypePath { path, .. }) => {
                let mut path = path.clone();
                path.drop_lifetimes();

                let seg = path.segments.last()?;
                let result = Some(seg)
                    .filter(|seg| seg.ident == "Result")
                    .and_then(|seg| {
                        if let PathArguments::AngleBracketed(args) = &seg.arguments {
                            args.args
                                .iter()
                                .find(|arg| matches!(arg, GenericArgument::Type(_)))
                                .map(|ty| ty.to_token_stream().to_string())
                        } else {
                            None
                        }
                    });
                let stringified = match result {
                    Some(result) if is_return => result,
                    _ => path.to_token_stream().to_string(),
                };

                Some(Arg::new(
                    name,
                    stringified,
                    seg.ident == "Option" || default.is_some(),
                    default,
                ))
            }
            Type::Reference(ref_) => {
                // Returning references is invalid, so let's just create our arg
                Some(Arg::new(
                    name,
                    ref_.to_token_stream().to_string(),
                    false,
                    default,
                ))
            }
            _ => None,
        }
    }

    #[inline]
    pub fn get_type_ident(&self) -> TokenStream {
        let ty: Type = syn::parse_str(&self.ty).unwrap();
        quote! {
            <#ty as ::ext_php_rs::convert::FromZvalMut>::TYPE
        }
    }

    #[inline]
    pub fn get_name_ident(&self) -> Ident {
        Ident::new(&self.name, Span::call_site())
    }

    /// Returns a [`TokenStream`] containing the line required to retrieve the
    /// value from the argument.
    pub fn get_accessor(&self, ret: &TokenStream) -> TokenStream {
        let name = &self.name;
        let name_ident = self.get_name_ident();

        if let Some(default) = self.default.as_ref() {
            // `bool`s are not literals - need to use Ident.
            let val = syn::parse_str::<Literal>(default)
                .map(|lit| lit.to_token_stream())
                .or_else(|_| Ident::from_string(default).map(|ident| ident.to_token_stream()))
                .unwrap_or(quote! { Default::default() });

            quote! { #name_ident.val().unwrap_or(#val.into()) }
        } else if self.nullable {
            quote! { #name_ident.val() }
        } else {
            quote! {
                match #name_ident.val() {
                    Some(val) => val,
                    None => {
                        ::ext_php_rs::exception::PhpException::default(
                            concat!("Invalid value given for argument `", #name, "`.").into()
                        )
                        .throw()
                        .expect(concat!("Failed to throw exception: Invalid value given for argument `", #name, "`."));
                        #ret
                    }
                }
            }
        }
    }

    /// Returns a [`TokenStream`] containing the line required to instantiate
    /// the argument.
    pub fn get_arg_definition(&self) -> TokenStream {
        let name = &self.name;
        let ty = self.get_type_ident();

        let null = self.nullable.then(|| quote! { .allow_null() });
        let default = self.default.as_ref().map(|val| {
            quote! {
                .default(#val)
            }
        });

        quote! {
            ::ext_php_rs::args::Arg::new(#name, #ty) #null #default
        }
    }
}

impl Function {
    #[inline]
    pub fn get_name_ident(&self) -> Ident {
        Ident::new(&self.ident, Span::call_site())
    }

    pub fn get_builder(&self) -> TokenStream {
        let name = &self.name;
        let name_ident = self.get_name_ident();
        let args = self
            .args
            .iter()
            .map(|arg| {
                let def = arg.get_arg_definition();
                let prelude = self.optional.as_ref().and_then(|opt| {
                    if opt.eq(&arg.name) {
                        Some(quote! { .not_required() })
                    } else {
                        None
                    }
                });
                quote! { #prelude.arg(#def) }
            })
            .collect::<Vec<_>>();
        let output = self.output.as_ref().map(|(ty, nullable)| {
            let ty: Type = syn::parse_str(ty).expect("failed to parse ty");

            // TODO allow reference returns?
            quote! {
                .returns(<#ty as ::ext_php_rs::convert::IntoZval>::TYPE, false, #nullable)
            }
        });

        quote! {
            ::ext_php_rs::builders::FunctionBuilder::new(#name, #name_ident)
                #(#args)*
                #output
                .build()
        }
    }
}
