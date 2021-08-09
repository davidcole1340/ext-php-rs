use darling::FromMeta;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    punctuated::Punctuated, AttributeArgs, FnArg, GenericArgument, ItemFn, PathArguments,
    ReturnType, Signature, Token, Type,
};

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, FromMeta)]
struct AttrArgs {
    #[darling(default)]
    optional: Option<String>,
}

#[derive(Debug)]
struct Arg {
    name: String,
    ty: String,
    nullable: bool,
}

pub fn parser(args: AttributeArgs, input: ItemFn) -> Result<TokenStream> {
    let attr_args = match AttrArgs::from_list(&args) {
        Ok(args) => args,
        Err(e) => return Err(format!("Unable to parse attribute arguments: {:?}", e)),
    };

    let ItemFn { sig, block, .. } = input;
    let Signature {
        ident,
        output,
        inputs,
        ..
    } = sig;
    let stmts = &block.stmts;

    let args = build_args(&inputs)?;
    let arg_definitions = build_arg_definitions(&args);
    let arg_parser = build_arg_parser(&args, &attr_args.optional)?;
    let arg_accessors = build_arg_accessors(&args);

    let return_handler = build_return_handler(&output);
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
    Ok(func)
}

fn build_args(inputs: &Punctuated<FnArg, Token![,]>) -> Result<Vec<Arg>> {
    inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Receiver(_) => Err("`self` is not permitted in PHP functions.".to_string()),
            FnArg::Typed(ty) => {
                let name = match &*ty.pat {
                    syn::Pat::Ident(pat) => pat.ident.to_string(),
                    _ => return Err("Invalid parameter type.".to_string()),
                };
                syn_arg_to_arg(name, &ty.ty)
            }
        })
        .collect::<Result<Vec<_>>>()
}

fn build_arg_definitions(args: &[Arg]) -> Vec<TokenStream> {
    args.iter().map(|ty| ty.get_arg_definition()).collect()
}

fn build_arg_parser(args: &[Arg], optional: &Option<String>) -> Result<TokenStream> {
    let mut rest_optional = false;

    let args = args
        .iter()
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

            if rest_optional && !arg.nullable {
                Err(format!(
                    "Parameter `{}` must be a variant of `Option` as it is optional.",
                    arg.name
                ))
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

fn build_return_handler(output_type: &ReturnType) -> TokenStream {
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
                            Err(e) => {
                                ::ext_php_rs::php::exceptions::throw(
                                    ::ext_php_rs::php::class::ClassEntry::exception(),
                                    e.to_string().as_ref()
                                ).expect("Failed to throw exception: Error type returned from internal function.");
                            }
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

fn syn_arg_to_arg(name: String, ty: &syn::Type) -> Result<Arg> {
    let ty_path = match ty {
        Type::Path(path) => path,
        ty => {
            return Err(format!(
                "Unsupported parameter type for parameter `{}`: {:?}",
                name, ty
            ))
        }
    };

    let ty_seg = ty_path
        .path
        .segments
        .last()
        .ok_or(format!("Invalid parameter type for parameter `{}`.", name))?;

    Ok(match ty_seg.ident.to_string().as_ref() {
        "Vec" | "HashMap" | "ZendHashTable" => Arg::new(&name, "Array"),
        "String" => Arg::new(&name, "String"),
        "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" => Arg::new(&name, "Long"),
        "f32" | "f64" => Arg::new(&name, "Double"),
        "Option" => match &ty_seg.arguments {
            PathArguments::AngleBracketed(args) => match args.args.first() {
                Some(GenericArgument::Type(ty)) => {
                    let mut ty = syn_arg_to_arg(name, ty)?;
                    ty.nullable = true;
                    ty
                }
                _ => return Err(format!("Invalid parameter type for parameter `{}`.", name)),
            },
            _ => return Err(format!("Invalid parameter type for parameter `{}`.", name)),
        },
        _ => return Err(format!("Invalid parameter type for parameter `{}`.", name)),
    })
}

impl Arg {
    fn new(name: &str, ty: &str) -> Self {
        Self {
            name: name.to_string(),
            ty: ty.to_string(),
            nullable: false,
        }
    }

    #[inline]
    fn get_type_ident(&self) -> Ident {
        Ident::new(&self.ty, Span::call_site())
    }

    #[inline]
    fn get_name_ident(&self) -> Ident {
        Ident::new(&self.name, Span::call_site())
    }

    /// Returns a [`TokenStream`] containing the line required to retrieve the value from the argument.
    fn get_accessor(&self) -> TokenStream {
        let name = &self.name;
        let name_ident = self.get_name_ident();

        if self.nullable {
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
    fn get_arg_definition(&self) -> TokenStream {
        let name = &self.name;
        let name_ident = self.get_name_ident();
        let ty = self.get_type_ident();

        let args = if self.nullable {
            quote! { .allow_null() }
        } else {
            quote! {}
        };

        quote! {
            let mut #name_ident = ::ext_php_rs::php::args::Arg::new(#name, ::ext_php_rs::php::enums::DataType::#ty) #args;
        }
    }
}
