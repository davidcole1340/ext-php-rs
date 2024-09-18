use anyhow::{anyhow, bail, Result};
use quote::ToTokens;
use std::collections::HashMap;
use syn::ReturnType;

use crate::helpers::get_docs;
use crate::{
    function::{self, ParserType},
    impl_::{parse_attribute, ParsedAttribute, PropAttrTy, RenameRule, Visibility},
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, FnArg, ImplItemMethod, Lit, Pat, Token, Type};

#[derive(Debug, Clone)]
pub enum Arg {
    Receiver(MethodType),
    Typed(function::Arg),
}

#[derive(Debug, Clone)]
pub struct Method {
    /// Method name
    pub name: String,
    /// extern "C" function ident
    pub ident: String,
    /// Rust internal function ident
    pub orig_ident: String,
    pub docs: Vec<String>,
    pub args: Vec<Arg>,
    pub optional: Option<String>,
    pub output: Option<(String, bool)>,
    pub _static: bool,
    pub _abstract: bool,
    pub visibility: Visibility,
}

pub struct ParsedMethod {
    pub tokens: TokenStream,
    pub method: Method,
    pub property: Option<(String, PropAttrTy)>,
    pub constructor: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum MethodType {
    Receiver,
    ReceiverClassObject,
    Static,
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

pub fn parser(
    struct_ty: &Type,
    mut input: ImplItemMethod,
    rename_rule: RenameRule,
) -> Result<ParsedMethod> {
    let mut defaults = HashMap::new();
    let mut optional = None;
    let mut visibility = Visibility::Public;
    let mut as_prop = None;
    let mut identifier = None;
    let mut is_abstract = false;
    let mut is_constructor = false;
    let docs = get_docs(&input.attrs);

    for attr in input.attrs.iter() {
        if let Some(attr) = parse_attribute(attr)? {
            match attr {
                ParsedAttribute::Default(list) => defaults = list,
                ParsedAttribute::Optional(name) => optional = Some(name),
                ParsedAttribute::Visibility(vis) => visibility = vis,
                ParsedAttribute::Abstract => is_abstract = true,
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
                _ => bail!("Invalid attribute for method."),
            }
        }
    }

    input.attrs.clear();

    let ident = &input.sig.ident;
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
    let internal_ident = Ident::new(&format!("_internal_php_{ident}"), Span::call_site());
    let args = build_args(struct_ty, &mut input.sig.inputs, &defaults)?;
    let optional = function::find_optional_parameter(
        args.iter().filter_map(|arg| match arg {
            Arg::Typed(arg) => Some(arg),
            _ => None,
        }),
        optional,
    );
    let (arg_definitions, method_type) = build_arg_definitions(&args);
    let arg_parser = build_arg_parser(
        args.iter(),
        &optional,
        &bail,
        match method_type {
            MethodType::Static => ParserType::StaticMethod,
            _ => ParserType::Method,
        },
    )?;
    let arg_accessors = build_arg_accessors(&args, &bail);

    let func = if is_constructor {
        quote! {
            #input

            #[doc(hidden)]
            pub fn #internal_ident(
                ex: &mut ::ext_php_rs::zend::ExecuteData
            ) -> ::ext_php_rs::class::ConstructorResult<Self> {
                use ::ext_php_rs::convert::IntoZval;
                use ::ext_php_rs::class::ConstructorResult;

                #(#arg_definitions)*
                #arg_parser

                Self::#ident(#(#arg_accessors,)*).into()
            }
        }
    } else {
        let this = match method_type {
            MethodType::Receiver => quote! { this. },
            MethodType::ReceiverClassObject | MethodType::Static => quote! { Self:: },
        };

        quote! {
            #input

            ::ext_php_rs::zend_fastcall! {
                #[doc(hidden)]
                pub extern fn #internal_ident(
                    ex: &mut ::ext_php_rs::zend::ExecuteData,
                    retval: &mut ::ext_php_rs::types::Zval
                ) {
                    use ::ext_php_rs::convert::IntoZval;

                    #(#arg_definitions)*
                    #arg_parser

                    let result = #this #ident(#(#arg_accessors,)*);

                    if let Err(e) = result.set_zval(retval, false) {
                        let e: ::ext_php_rs::exception::PhpException = e.into();
                        e.throw().expect("Failed to throw exception");
                    }
                }
            }
        }
    };

    let method = Method {
        name,
        ident: internal_ident.to_string(),
        orig_ident: ident.to_string(),
        docs,
        args,
        optional,
        output: get_return_type(struct_ty, &input.sig.output)?,
        _static: matches!(method_type, MethodType::Static),
        _abstract: is_abstract,
        visibility,
    };

    Ok(ParsedMethod::new(func, method, as_prop, is_constructor))
}

pub fn get_return_type(self_ty: &Type, output_type: &ReturnType) -> Result<Option<(String, bool)>> {
    Ok(match output_type {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => {
            let mut ty = ty.clone();
            replace_self(self_ty, &mut ty);
            crate::function::Arg::from_type("".to_string(), &ty, None, true)
                .map(|arg| (arg.ty, arg.nullable))
        }
    })
}

/// Takes a type `ty` and replaces all instances of `Self` with the type
/// `self_ty`.
fn replace_self(self_ty: &Type, ty: &mut Type) {
    match ty {
        Type::Array(syn::TypeArray { elem, .. }) => replace_self(self_ty, elem),
        Type::BareFn(syn::TypeBareFn { inputs, output, .. }) => {
            for input in inputs {
                replace_self(self_ty, &mut input.ty);
            }
            if let ReturnType::Type(_, ty) = output {
                replace_self(self_ty, ty);
            }
        }
        Type::Group(syn::TypeGroup { elem, .. }) => replace_self(self_ty, elem),
        Type::Paren(syn::TypeParen { elem, .. }) => replace_self(self_ty, elem),
        Type::Path(syn::TypePath { qself, path }) => {
            if let Some(syn::QSelf { ty, .. }) = qself {
                replace_self(self_ty, ty);
            }
            for seg in &mut path.segments {
                if seg.ident == "Self" {
                    seg.ident =
                        Ident::new(&self_ty.to_token_stream().to_string(), Span::call_site());
                }
                match &mut seg.arguments {
                    syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                        args,
                        ..
                    }) => {
                        for arg in args {
                            if let syn::GenericArgument::Type(ty) = arg {
                                replace_self(self_ty, ty);
                            }
                        }
                    }
                    syn::PathArguments::Parenthesized(syn::ParenthesizedGenericArguments {
                        inputs,
                        output,
                        ..
                    }) => {
                        for input in inputs {
                            replace_self(self_ty, input);
                        }

                        if let ReturnType::Type(_, ty) = output {
                            replace_self(self_ty, ty);
                        }
                    }
                    _ => {}
                }
            }
        }
        Type::Ptr(syn::TypePtr { elem, .. }) => replace_self(self_ty, elem),
        Type::Reference(syn::TypeReference { elem, .. }) => replace_self(self_ty, elem),
        Type::Slice(syn::TypeSlice { elem, .. }) => replace_self(self_ty, elem),
        Type::Tuple(syn::TypeTuple { elems, .. }) => {
            for elem in elems {
                replace_self(self_ty, elem);
            }
        }
        _ => {}
    }
}

fn build_args(
    struct_ty: &Type,
    inputs: &mut Punctuated<FnArg, Token![,]>,
    defaults: &HashMap<String, Lit>,
) -> Result<Vec<Arg>> {
    inputs
        .iter_mut()
        .map(|arg| match arg {
            FnArg::Receiver(receiver) => {
                if receiver.reference.is_none() {
                    bail!("`self` parameter must be a reference.");
                }
                Ok(Arg::Receiver(MethodType::Receiver))
            }
            FnArg::Typed(ty) => {
                let mut this = false;
                let attrs = std::mem::take(&mut ty.attrs);
                for attr in attrs.into_iter() {
                    if let Some(attr) = parse_attribute(&attr)? {
                        match attr {
                            ParsedAttribute::This => this = true,
                            _ => bail!("Invalid attribute for argument."),
                        }
                    }
                }

                if this {
                    Ok(Arg::Receiver(MethodType::ReceiverClassObject))
                } else {
                    let name = match &*ty.pat {
                        Pat::Ident(pat) => pat.ident.to_string(),
                        _ => bail!("Invalid parameter type."),
                    };
                    let default = defaults.get(&name);
                    let mut ty = ty.ty.clone();
                    replace_self(struct_ty, &mut ty);

                    Ok(Arg::Typed(
                        crate::function::Arg::from_type(name.clone(), &ty, default, false)
                            .ok_or_else(|| anyhow!("Invalid parameter type for `{}`.", name))?,
                    ))
                }
            }
        })
        .collect()
}

fn build_arg_definitions(args: &[Arg]) -> (Vec<TokenStream>, MethodType) {
    let mut method_type = MethodType::Static;

    (
        args.iter()
            .filter_map(|ty| match ty {
                Arg::Receiver(t) => {
                    method_type = *t;
                    None
                }
                Arg::Typed(arg) => {
                    let ident = arg.get_name_ident();
                    let definition = arg.get_arg_definition();
                    Some(quote! {
                        let mut #ident = #definition;
                    })
                }
            })
            .collect(),
        method_type,
    )
}

fn build_arg_parser<'a>(
    args: impl Iterator<Item = &'a Arg>,
    optional: &Option<String>,
    ret: &TokenStream,
    ty: ParserType,
) -> Result<TokenStream> {
    function::build_arg_parser(
        args.filter_map(|arg| match arg {
            Arg::Typed(arg) => Some(arg),
            _ => None,
        }),
        optional,
        ret,
        ty,
    )
}

fn build_arg_accessors(args: &[Arg], ret: &TokenStream) -> Vec<TokenStream> {
    args.iter()
        .filter_map(|arg| match arg {
            Arg::Typed(arg) => Some(arg.get_accessor(ret)),
            Arg::Receiver(MethodType::ReceiverClassObject) => Some(quote! { this }),
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
                .returns(<#ty as ::ext_php_rs::convert::IntoZval>::TYPE, false, #nullable)
            }
        });

        quote! {
            ::ext_php_rs::builders::FunctionBuilder::new(#name, #class_path :: #name_ident)
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

        if self._abstract {
            flags.push(quote! { Abstract });
        }

        flags
            .iter()
            .map(|flag| quote! { ::ext_php_rs::flags::MethodFlags::#flag })
            .collect::<Punctuated<TokenStream, Token![|]>>()
            .to_token_stream()
    }
}
