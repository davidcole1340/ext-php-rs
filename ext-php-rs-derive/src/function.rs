use std::{collections::HashMap, str::FromStr};

use crate::{error::Result, STATE};
use darling::{FromMeta, ToTokens};
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use syn::{
    punctuated::Punctuated, AttributeArgs, FnArg, GenericArgument, ItemFn, Lit, PathArguments,
    PathSegment, ReturnType, Signature, Token, Type,
};

#[derive(Default, Debug, FromMeta)]
#[darling(default)]
pub struct AttrArgs {
    optional: Option<String>,
    ignore_module: bool,
    defaults: HashMap<String, Lit>,
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
    pub args: Vec<Arg>,
    pub optional: Option<String>,
    pub output: Option<(String, bool)>,
}

pub fn parser(args: AttributeArgs, input: ItemFn) -> Result<(TokenStream, Function)> {
    let attr_args = match AttrArgs::from_list(&args) {
        Ok(args) => args,
        Err(e) => return Err(format!("Unable to parse attribute arguments: {:?}", e).into()),
    };

    let ItemFn { sig, block, .. } = input;
    let Signature {
        ident,
        output,
        inputs,
        ..
    } = sig;
    let stmts = &block.stmts;

    let args = build_args(&inputs, &attr_args.defaults)?;
    let arg_definitions = build_arg_definitions(&args);
    let arg_parser = build_arg_parser(args.iter(), &attr_args.optional)?;
    let arg_accessors = build_arg_accessors(&args);

    let return_handler = build_return_handler(&output);
    let return_type = get_return_type(&output)?;

    let func = quote! {
        pub extern "C" fn #ident(ex: &mut ::ext_php_rs::php::execution_data::ExecutionData, retval: &mut ::ext_php_rs::php::types::zval::Zval) {
            use ::ext_php_rs::php::types::zval::IntoZval;

            fn internal(#inputs) #output {
                #(#stmts)*
            }

            #(#arg_definitions)*
            #arg_parser

            let result = internal(#(#arg_accessors, )*);

            #return_handler
        }
    };

    let mut state = STATE.lock()?;

    if state.built_module && !attr_args.ignore_module {
        return Err("The `#[php_module]` macro must be called last to ensure functions are registered. To ignore this error, pass the `ignore_module` option into this attribute invocation: `#[php_function(ignore_module)]`".into());
    }

    let function = Function {
        name: ident.to_string(),
        args,
        optional: attr_args.optional,
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
            FnArg::Receiver(_) => Err(
                "`self` is not permitted in PHP functions. See the `#[php_method]` attribute."
                    .into(),
            ),
            FnArg::Typed(ty) => {
                let name = match &*ty.pat {
                    syn::Pat::Ident(pat) => pat.ident.to_string(),
                    _ => return Err("Invalid parameter type.".into()),
                };
                syn_arg_to_arg(&name, &ty.ty, defaults.get(&name))
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

pub fn build_arg_parser<'a>(
    args: impl Iterator<Item = &'a Arg>,
    optional: &Option<String>,
) -> Result<TokenStream> {
    let mut rest_optional = false;

    let args = args
        .map(|arg| {
            let name = arg.get_name_ident();
            let prelude = if let Some(optional) = optional {
                if *optional == arg.name {
                    rest_optional = true;
                    quote! { .not_required() }
                } else {
                    quote! {}
                }
            } else {
                quote! {}
            };

            if rest_optional && !arg.nullable && arg.default.is_none() {
                Err(format!(
                    "Parameter `{}` must be a variant of `Option` or have a default value as it is optional.",
                    arg.name
                )
                .into())
            } else {
                Ok(quote! {
                    #prelude
                    .arg(&mut #name)
                })
            }
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        let parser = ::ext_php_rs::php::args::ArgParser::new(ex)
            #(#args)*
            .parse();

        if parser.is_err() {
            return;
        }
    })
}

fn build_arg_accessors(args: &[Arg]) -> Vec<TokenStream> {
    args.iter().map(|arg| arg.get_accessor()).collect()
}

pub fn build_return_handler(output_type: &ReturnType) -> TokenStream {
    let handler = match output_type {
        ReturnType::Default => Some(quote! { retval.set_null(); }),
        ReturnType::Type(_, ref ty) => match **ty {
            Type::Path(ref path) => match path.path.segments.last() {
                Some(path_seg) => match path_seg.ident.to_string().as_ref() {
                    "Result" => Some(quote! {
                        match result {
                            Ok(result) => match result.set_zval(retval, false) {
                                Ok(_) => {}
                                Err(e) => {
                                    ::ext_php_rs::php::exceptions::throw(
                                        ::ext_php_rs::php::class::ClassEntry::exception(),
                                        e.to_string().as_ref()
                                    ).expect("Failed to throw exception: Failed to set return value.");
                                },
                            },
                            Err(e) => {
                                ::ext_php_rs::php::exceptions::throw(
                                    ::ext_php_rs::php::class::ClassEntry::exception(),
                                    e.to_string().as_ref()
                                ).expect("Failed to throw exception: Error type returned from internal function.");
                            }
                        };
                    }),
                    "Option" => Some(quote! {
                        match result {
                            Some(result) => match result.set_zval(retval, false) {
                                Ok(_) => {}
                                Err(e) => {
                                    ::ext_php_rs::php::exceptions::throw(
                                        ::ext_php_rs::php::class::ClassEntry::exception(),
                                        e.to_string().as_ref()
                                    ).expect("Failed to throw exception: Failed to set return value.");
                                },
                            },
                            None => retval.set_null(),
                        };
                    }),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        },
    };

    match handler {
        Some(handler) => handler,
        None => quote! {
            match result.set_zval(retval, false) {
                Ok(_) => {},
                Err(e) => {
                    ::ext_php_rs::php::exceptions::throw(
                        ::ext_php_rs::php::class::ClassEntry::exception(),
                        e.to_string().as_ref()
                    ).expect("Failed to throw exception: Failed to set return value.");
                }
            }
        },
    }
}

pub fn get_return_type(output_type: &ReturnType) -> Result<Option<(String, bool)>> {
    Ok(match output_type {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => match **ty {
            Type::Path(ref path) => match path.path.segments.last() {
                Some(seg) => path_seg_to_arg("", seg).map(|arg| (arg.ty, arg.nullable)),
                None => return Err("Invalid return type.".into()),
            },
            _ => return Err("Invalid return type.".into()),
        },
    })
}

pub fn syn_arg_to_arg(name: &str, ty: &syn::Type, default: Option<&Lit>) -> Result<Arg> {
    let ty_path = match ty {
        Type::Path(path) => path,
        ty => {
            return Err(format!(
                "Unsupported parameter type for parameter `{}`: {:?}",
                name, ty
            )
            .into())
        }
    };

    let ty_seg = ty_path
        .path
        .segments
        .last()
        .ok_or(format!("Invalid parameter type for parameter `{}`.", name))?;

    path_seg_to_arg(name, ty_seg)
        .ok_or_else(|| format!("Invalid parameter type for parameter `{}`.", name).into())
        .map(|mut arg| {
            arg.default = default.map(|def| def.to_token_stream().to_string());
            arg
        })
}

pub fn path_seg_to_arg(name: &str, seg: &PathSegment) -> Option<Arg> {
    match seg.ident.to_string().as_ref() {
        "Vec" | "HashMap" | "ZendHashTable" => Some(Arg::new(name, "Array")),
        "Callable" => Some(Arg::new(name, "Callable")),
        "String" | "Binary" => Some(Arg::new(name, "String")),
        "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" => Some(Arg::new(name, "Long")),
        "f32" | "f64" => Some(Arg::new(name, "Double")),
        "bool" => Some(Arg::new(name, "Bool")),
        "Option" => match &seg.arguments {
            PathArguments::AngleBracketed(args) => match args.args.first() {
                Some(GenericArgument::Type(ty)) => {
                    let mut ty = syn_arg_to_arg(name, ty, None).ok()?;
                    ty.nullable = true;
                    Some(ty)
                }
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

impl Arg {
    pub fn new(name: &str, ty: &str) -> Self {
        Self {
            name: name.to_string(),
            ty: ty.to_string(),
            nullable: false,
            default: None,
        }
    }

    #[inline]
    pub fn get_type_ident(&self) -> Ident {
        Ident::new(&self.ty, Span::call_site())
    }

    #[inline]
    pub fn get_name_ident(&self) -> Ident {
        Ident::new(&self.name, Span::call_site())
    }

    /// Returns a [`TokenStream`] containing the line required to retrieve the value from the argument.
    pub fn get_accessor(&self) -> TokenStream {
        let name = &self.name;
        let name_ident = self.get_name_ident();

        if let Some(default) = self.default.as_ref() {
            // `bool`s are not literals - need to use Ident.
            let val = Literal::from_str(default)
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
                        ::ext_php_rs::php::exceptions::throw(
                            ::ext_php_rs::php::class::ClassEntry::exception(),
                            concat!("Invalid value given for argument `", #name, "`.")
                        )
                        .expect(concat!("Failed to throw exception: Invalid value given for argument `", #name, "`."));
                        return;
                    }
                }
            }
        }
    }

    /// Returns a [`TokenStream`] containing the line required to instantiate the argument.
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
            ::ext_php_rs::php::args::Arg::new(#name, ::ext_php_rs::php::enums::DataType::#ty) #null #default
        }
    }
}

impl Function {
    #[inline]
    pub fn get_name_ident(&self) -> Ident {
        Ident::new(&self.name, Span::call_site())
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
            let ty = Ident::new(ty, Span::call_site());
            // TODO allow reference returns?
            quote! {
                .returns(::ext_php_rs::php::enums::DataType::#ty, false, #nullable)
            }
        });

        quote! {
            ::ext_php_rs::php::function::FunctionBuilder::new(#name, #name_ident)
                #(#args)*
                #output
                .build()
        }
    }
}
