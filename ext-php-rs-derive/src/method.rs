use anyhow::{anyhow, bail, Result};
use quote::ToTokens;
use std::collections::HashMap;

use crate::{
    function,
    impl_::{parse_attribute, ParsedAttribute, PropAttrTy, RenameRule, Visibility},
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, FnArg, ImplItemMethod, Lit, Pat, Signature, Token, Type};

#[derive(Debug, Clone)]
pub enum Arg {
    Receiver(bool),
    Typed(function::Arg),
}

#[derive(Debug)]
pub struct AttrArgs {
    pub defaults: HashMap<String, Lit>,
    pub optional: Option<String>,
    pub visibility: Visibility,
}

#[derive(Debug, Clone)]
pub struct Method {
    /// Method name
    pub name: String,
    /// extern "C" function ident
    pub ident: String,
    /// Rust internal function ident
    pub orig_ident: String,
    pub args: Vec<Arg>,
    pub optional: Option<String>,
    pub output: Option<(String, bool)>,
    pub _static: bool,
    pub visibility: Visibility,
}

pub struct ParsedMethod {
    pub tokens: TokenStream,
    pub method: Method,
    pub property: Option<(String, PropAttrTy)>,
    pub constructor: bool,
}

impl ParsedMethod {
    pub fn new(
        tokens: TokenStream,
        method: Method,
        property: Option<(String, PropAttrTy)>,
        constructor: bool,
    ) -> Self {
        Self {
            tokens,
            method,
            property,
            constructor,
        }
    }
}

pub fn parser(input: &mut ImplItemMethod, rename_rule: RenameRule) -> Result<ParsedMethod> {
    let mut defaults = HashMap::new();
    let mut optional = None;
    let mut visibility = Visibility::Public;
    let mut as_prop = None;
    let mut identifier = None;
    let mut is_constructor = false;

    for attr in input.attrs.iter() {
        match parse_attribute(attr)? {
            ParsedAttribute::Default(list) => defaults = list,
            ParsedAttribute::Optional(name) => optional = Some(name),
            ParsedAttribute::Visibility(vis) => visibility = vis,
            ParsedAttribute::Rename(ident) => identifier = Some(ident),
            ParsedAttribute::Property { prop_name, ty } => {
                if as_prop.is_some() {
                    bail!(
                        "Only one `#[getter]` and/or `#[setter]` attribute may be used per method."
                    );
                }
                let prop_name = prop_name.unwrap_or_else(|| {
                    input
                        .sig
                        .ident
                        .to_token_stream()
                        .to_string()
                        .trim_start_matches("get_")
                        .trim_start_matches("set_")
                        .to_string()
                });
                as_prop = Some((prop_name, ty))
            }
            ParsedAttribute::Constructor => is_constructor = true,
        }
    }

    input.attrs.clear();

    let ImplItemMethod { sig, .. } = &input;
    let Signature {
        ident,
        output,
        inputs,
        ..
    } = &sig;

    let name = identifier.unwrap_or_else(|| rename_rule.rename(ident.to_string()));
    if name == "__construct" {
        is_constructor = true;
    }

    if is_constructor && (!matches!(visibility, Visibility::Public) || as_prop.is_some()) {
        bail!("`#[constructor]` attribute cannot be combined with the visibility or getter/setter attributes.");
    }

    let bail = if is_constructor {
        quote! { return ConstructorResult::ArgError; }
    } else {
        quote! { return; }
    };
    let internal_ident = Ident::new(&format!("_internal_php_{}", ident), Span::call_site());
    let args = build_args(inputs, &defaults)?;
    let optional = function::find_optional_parameter(
        args.iter().filter_map(|arg| match arg {
            Arg::Typed(arg) => Some(arg),
            _ => None,
        }),
        optional,
    );
    let (arg_definitions, is_static) = build_arg_definitions(&args);
    let arg_parser = build_arg_parser(args.iter(), &optional, &bail)?;
    let arg_accessors = build_arg_accessors(&args, &bail);
    let this = if is_static {
        quote! { Self:: }
    } else {
        quote! { this. }
    };

    let func = if is_constructor {
        quote! {
            #input

            #[doc(hidden)]
            pub fn #internal_ident(
                ex: &mut ::ext_php_rs::php::execution_data::ExecutionData
            ) -> ::ext_php_rs::php::types::object::ConstructorResult<Self> {
                use ::ext_php_rs::php::types::zval::IntoZval;
                use ::ext_php_rs::php::types::object::ConstructorResult;

                #(#arg_definitions)*
                #arg_parser

                Self::#ident(#(#arg_accessors,)*).into()
            }
        }
    } else {
        quote! {
            #input

            #[doc(hidden)]
            pub extern "C" fn #internal_ident(
                ex: &mut ::ext_php_rs::php::execution_data::ExecutionData,
                retval: &mut ::ext_php_rs::php::types::zval::Zval
            ) {
                use ::ext_php_rs::php::types::zval::IntoZval;

                #(#arg_definitions)*
                #arg_parser

                let result = #this #ident(#(#arg_accessors, )*);

                if let Err(e) = result.set_zval(retval, false) {
                    let e: ::ext_php_rs::php::exceptions::PhpException = e.into();
                    e.throw().expect("Failed to throw exception");
                }
            }
        }
    };

    let method = Method {
        name,
        ident: internal_ident.to_string(),
        orig_ident: ident.to_string(),
        args,
        optional,
        output: crate::function::get_return_type(output)?,
        _static: is_static,
        visibility,
    };

    Ok(ParsedMethod::new(func, method, as_prop, is_constructor))
}

fn build_args(
    inputs: &Punctuated<FnArg, Token![,]>,
    defaults: &HashMap<String, Lit>,
) -> Result<Vec<Arg>> {
    inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Receiver(receiver) => {
                if receiver.reference.is_none() {
                    bail!("`self` parameter must be a reference.");
                }
                Ok(Arg::Receiver(receiver.mutability.is_some()))
            }
            FnArg::Typed(ty) => {
                let name = match &*ty.pat {
                    Pat::Ident(pat) => pat.ident.to_string(),
                    _ => bail!("Invalid parameter type."),
                };
                let default = defaults.get(&name);
                Ok(Arg::Typed(
                    crate::function::Arg::from_type(name.clone(), &ty.ty, default, false)
                        .ok_or_else(|| anyhow!("Invalid parameter type for `{}`.", name))?,
                ))
            }
        })
        .collect()
}

fn build_arg_definitions(args: &[Arg]) -> (Vec<TokenStream>, bool) {
    let mut _static = true;

    (args.iter()
        .map(|ty| match ty {
            Arg::Receiver(mutability) => {
                let mutability = mutability.then(|| quote! { mut });
                _static = false;

                quote! {
                    // SAFETY: We are calling this on an execution data from a class method.
                    let #mutability this = match unsafe { ex.get_object::<Self>() } {
                        Some(this) => this,
                        None => return ::ext_php_rs::php::exceptions::throw(
                            ::ext_php_rs::php::class::ClassEntry::exception(),
                            "Failed to retrieve reference to object function was called on."
                        ).expect("Failed to throw exception: Failed to retrieve reference to object function was called on."),
                    };
                }
            }
            Arg::Typed(arg) => {
                let ident = arg.get_name_ident();
                let definition = arg.get_arg_definition();
                quote! {
                    let mut #ident = #definition;
                }
            },
        })
        .collect(), _static)
}

fn build_arg_parser<'a>(
    args: impl Iterator<Item = &'a Arg>,
    optional: &Option<String>,
    ret: &TokenStream,
) -> Result<TokenStream> {
    function::build_arg_parser(
        args.filter_map(|arg| match arg {
            Arg::Typed(arg) => Some(arg),
            _ => None,
        }),
        optional,
        ret,
    )
}

fn build_arg_accessors(args: &[Arg], ret: &TokenStream) -> Vec<TokenStream> {
    args.iter()
        .filter_map(|arg| match arg {
            Arg::Typed(arg) => Some(arg.get_accessor(ret)),
            _ => None,
        })
        .collect()
}

impl Method {
    #[inline]
    pub fn get_name_ident(&self) -> Ident {
        Ident::new(&self.ident, Span::call_site())
    }

    pub fn get_arg_definitions(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.args.iter().filter_map(move |arg| match arg {
            Arg::Typed(arg) => {
                let def = arg.get_arg_definition();
                let prelude = self.optional.as_ref().and_then(|opt| {
                    if opt.eq(&arg.name) {
                        Some(quote! { .not_required() })
                    } else {
                        None
                    }
                });
                Some(quote! { #prelude.arg(#def) })
            }
            _ => None,
        })
    }

    pub fn get_builder(&self, class_path: &Ident) -> TokenStream {
        let name = &self.name;
        let name_ident = self.get_name_ident();
        let args = self.get_arg_definitions();
        let output = self.output.as_ref().map(|(ty, nullable)| {
            let ty: Type = syn::parse_str(ty).unwrap();

            // TODO allow reference returns?
            quote! {
                .returns(<#ty as ::ext_php_rs::php::types::zval::IntoZval>::TYPE, false, #nullable)
            }
        });

        quote! {
            ::ext_php_rs::php::function::FunctionBuilder::new(#name, #class_path :: #name_ident)
                #(#args)*
                #output
                .build()
        }
    }

    pub fn get_flags(&self) -> TokenStream {
        let mut flags = vec![match self.visibility {
            Visibility::Public => quote! { Public },
            Visibility::Protected => quote! { Protected },
            Visibility::Private => quote! { Private },
        }];

        if self._static {
            flags.push(quote! { Static });
        }

        flags
            .iter()
            .map(|flag| quote! { ::ext_php_rs::php::flags::MethodFlags::#flag })
            .collect::<Punctuated<TokenStream, Token![|]>>()
            .to_token_stream()
    }
}
