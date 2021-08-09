use darling::FromMeta;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DeriveInput, ItemFn, ReturnType, Signature};

extern crate proc_macro;

/// Derives the implementation of `ZendObjectOverride` for the given structure.
#[proc_macro_derive(ZendObjectHandler)]
pub fn object_handler_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let handlers = Ident::new(
        format!("__{}_OBJECT_HANDLERS", name).as_str(),
        Span::call_site(),
    );

    let output = quote! {
        static mut #handlers: Option<
            *mut ::ext_php_rs::php::types::object::ZendObjectHandlers
        > = None;

        impl ::ext_php_rs::php::types::object::ZendObjectOverride for #name {
            extern "C" fn create_object(
                ce: *mut ::ext_php_rs::php::class::ClassEntry,
            ) -> *mut ::ext_php_rs::php::types::object::ZendObject {
                // SAFETY: The handlers are only modified once, when they are first accessed.
                // At the moment we only support single-threaded PHP installations therefore the pointer contained
                // inside the option can be passed around.
                unsafe {
                    if #handlers.is_none() {
                        #handlers = Some(::ext_php_rs::php::types::object::ZendObjectHandlers::init::<#name>());
                    }

                    // The handlers unwrap can never fail - we check that it is none above.
                    // Unwrapping the result from `new_ptr` is nessacary as C cannot handle results.
                    ::ext_php_rs::php::types::object::ZendClassObject::<#name>::new_ptr(
                        ce,
                        #handlers.unwrap()
                    ).expect("Failed to allocate memory for new Zend object.")
                }
            }
        }
    };

    TokenStream::from(output)
}

#[derive(Debug, FromMeta)]
struct PhpFunctionArgs {
    #[darling(default)]
    optional: Option<String>,
}

#[proc_macro_attribute]
pub fn php_function(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let macro_args = {
        let attr = parse_macro_input!(attr as AttributeArgs);
        match PhpFunctionArgs::from_list(&attr) {
            Ok(v) => v,
            Err(e) => return TokenStream::from(e.write_errors()),
        }
    };

    let ItemFn { sig, block, .. } = func;
    let Signature {
        ident,
        output,
        inputs,
        ..
    } = sig;
    let stmts = &block.stmts;

    let mut args = vec![];
    for i in inputs.iter() {
        args.push(match i {
            syn::FnArg::Receiver(_) => todo!(),
            syn::FnArg::Typed(i) => {
                let name = match &*i.pat {
                    syn::Pat::Ident(id) => id.ident.to_string(),
                    _ => panic!(
                        "Invalid parameter type. Function cannot accept `self` as an argument."
                    ),
                };
                pat_type_to_arg(name, &i.ty)
            }
        });
    }

    let arg_def = args.iter().map(|a| a.to_arg()).collect::<Vec<_>>();
    let mut rest_optional = false;
    let arg_parse = args
        .iter()
        .map(|a| {
            let name = Ident::new(&a.name, Span::call_site());
            let pre = if let Some(optional) = &macro_args.optional {
                if *optional == a.name {
                    rest_optional = true;
                    quote! { .not_required() }
                } else {
                    quote! {}
                }
            } else {
                quote! {}
            };
            if rest_optional && !a.nullable {
                panic!(
                    "Parameter `{}` must be a variant of `Option` as it is optional.",
                    a.name
                );
            }
            quote! {
                #pre
                .arg(&mut #name)
            }
        })
        .collect::<Vec<_>>();
    let arg_get = args
        .iter()
        .map(|a| a.get_zval_conversion_fn(false))
        .collect::<Vec<_>>();

    let return_handler = build_return_handler(&output);

    TokenStream::from(quote! {
        pub extern "C" fn #ident(ex: &mut ::ext_php_rs::php::execution_data::ExecutionData, retval: &mut ::ext_php_rs::php::types::zval::Zval) {
            use ::ext_php_rs::php::types::zval::IntoZval;

            fn internal(#inputs) #output {
                #(#stmts)*
            }

            #(#arg_def)*

            let parser = ::ext_php_rs::php::args::ArgParser::new(ex)
                #(#arg_parse)*
                .parse();

            if parser.is_err() {
                return;
            }

            let result = internal(#(#arg_get, )*);

            #return_handler
        }
    })
}

fn build_return_handler(output: &ReturnType) -> TokenStream2 {
    match output {
        syn::ReturnType::Default => quote! {
            retval.set_null();
        },
        syn::ReturnType::Type(_, ref ty) => {
            if let syn::Type::Path(ref pat) = **ty {
                if let Some(seg) = pat.path.segments.last() {
                    match seg.ident.to_string().as_ref() {
                        "Result" => {
                            return quote! {
                                match result {
                                    Ok(r) => match r.set_zval(retval, false) {
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
                            }
                        }
                        "Option" => {
                            return quote! {
                                match result {
                                    Some(r) => match r.set_zval(retval, false) {
                                        Ok(_) => {}
                                        Err(e) => {
                                            ::ext_php_rs::php::exceptions::throw(
                                                ::ext_php_rs::php::class::ClassEntry::exception(),
                                                e.to_string().as_ref()
                                            ).expect("Failed to throw exception: Failed to set return value.");
                                        },
                                    },
                                    None => retval.set_null()
                                };
                            }
                        }
                        _ => {}
                    }
                }
            }

            quote! {
                match result.set_zval(retval, false) {
                    Ok(_) => {}
                    Err(e) => {
                        ::ext_php_rs::php::exceptions::throw(
                            ::ext_php_rs::php::class::ClassEntry::exception(),
                            e.to_string().as_ref()
                        ).expect("Failed to throw exception: Failed to set return value.");
                    },
                }
            }
        }
    }
}

fn pat_type_to_arg(name: String, ty: &syn::Type) -> Type {
    let tp = if let syn::Type::Path(path) = ty {
        path
    } else {
        panic!("unsupported parameter type");
    };

    let seg = tp
        .path
        .segments
        .last()
        .expect(format!("Invalid parameter type for parameter `{}`.", name).as_ref());

    match seg.ident.to_string().as_ref() {
        "Vec" | "HashMap" | "ZendHashTable" => Type {
            name,
            ty: DataType::Array,
            nullable: false,
        },
        "Option" => match &seg.arguments {
            syn::PathArguments::AngleBracketed(t) => {
                match t.args.first().expect("unsupported parameter type") {
                    syn::GenericArgument::Type(ty) => {
                        let mut ty = pat_type_to_arg(name, ty);
                        ty.nullable = true;
                        ty
                    }
                    _ => panic!("unsupported parameter type"),
                }
            }
            _ => panic!("unsupported parameter type"),
        },
        "String" => Type {
            name,
            ty: DataType::String,
            nullable: false,
        },
        "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" => Type {
            name,
            ty: DataType::Long,
            nullable: false,
        },
        "f32" | "f64" => Type {
            name,
            ty: DataType::Double,
            nullable: false,
        },
        // "bool" => "Bool",
        v => panic!("Invalid parameter type for parameter `{}`: `{}`.", name, v),
    }
}

struct Type {
    name: String,
    ty: DataType,
    nullable: bool,
}

enum DataType {
    String,
    Double,
    Long,
    Array,
}

impl Type {
    fn get_data_type(&self) -> Ident {
        Ident::new(
            match self {
                Type {
                    ty: DataType::String,
                    ..
                } => "String",
                Type {
                    ty: DataType::Long, ..
                } => "Long",
                Type {
                    ty: DataType::Double,
                    ..
                } => "Double",
                Type {
                    ty: DataType::Array,
                    ..
                } => "Array",
            },
            Span::call_site(),
        )
    }

    fn get_zval_conversion_fn(&self, optional: bool) -> TokenStream2 {
        let name = &self.name;
        let name_ident = Ident::new(&name, Span::call_site());

        if self.nullable || optional {
            quote! { #name_ident.val() }
        } else {
            quote! {
                match #name_ident.val() {
                    Some(v) => v,
                    None => {
                        ::ext_php_rs::php::exceptions::throw(
                            ::ext_php_rs::php::class::ClassEntry::exception(),
                            concat!("Invalid value given for argument `", #name, "`.")
                        ).expect(concat!("Failed to throw exception: Invalid value given for argument `", #name, "`."));
                        return;
                    }
                }
            }
        }
    }

    fn to_arg(&self) -> TokenStream2 {
        let name = &self.name;
        let name_ident = Ident::new(&self.name, Span::call_site());
        let ty = self.get_data_type();

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
