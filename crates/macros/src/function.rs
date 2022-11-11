use std::collections::HashMap;

use darling::{FromMeta, ToTokens};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{spanned::Spanned, AttributeArgs, FnArg, Ident, ItemFn, Lit, PatType, Type};

use crate::prelude::*;
use crate::syn_ext::DropLifetimes;

#[derive(Debug, Default, FromMeta)]
#[darling(default)]
pub struct FnArgs {
    /// The name of the function.
    name: Option<String>,
    /// The first optional argument of the function signature.
    optional: Option<Ident>,
    /// Default values for optional arguments.
    defaults: HashMap<Ident, Lit>,
}

pub fn wrap(input: syn::Path) -> Result<TokenStream> {
    let Some(func_name) = input.get_ident() else {
        bail!(input => "Pass a PHP function name into `wrap_function!()`.");
    };
    let builder_func = format_ident!("_internal_{func_name}");
    let err = format!("Failed to build function `{}`.", func_name);

    Ok(quote! {{
        (<#builder_func as ::ext_php_rs::internal::function::PhpFunction>::FUNCTION_ENTRY)()
            .expect(#err)
    }})
}

pub fn parser(opts: AttributeArgs, input: ItemFn) -> Result<TokenStream> {
    let opts = match FnArgs::from_list(&opts) {
        Ok(opts) => opts,
        Err(e) => bail!("Failed to parse attribute options: {:?}", e),
    };
    let args = Args::parse_from_fnargs(input.sig.inputs.iter(), opts.defaults)?;
    if let Some(ReceiverArg { span, .. }) = args.receiver {
        bail!(span => "Receiver arguments are invalid on PHP functions. See `#[php_impl]`.");
    }
    let func = Function::parse(&input.sig, opts.name, args, opts.optional)?;
    let function_impl = func.php_function_impl()?;

    Ok(quote! {
        #input
        #function_impl
    })
}

#[derive(Debug)]
pub struct Function<'a> {
    /// Identifier of the Rust function associated with the function.
    pub ident: &'a Ident,
    /// Name of the function in PHP.
    pub name: String,
    /// Function arguments.
    pub args: Args<'a>,
    /// Function outputs.
    pub output: Option<&'a Type>,
    /// The first optional argument of the function.
    pub optional: Option<Ident>,
}

#[derive(Debug)]
pub enum CallType<'a> {
    Function,
    Method {
        class: &'a syn::Path,
        receiver: MethodReceiver,
    },
}

/// Type of receiver on the method.
#[derive(Debug)]
pub enum MethodReceiver {
    /// Static method - has no receiver.
    Static,
    /// Class method, takes `&self` or `&mut self`.
    Class,
    /// Class method, takes `&mut ZendClassObject<Self>`.
    ZendClassObject,
}

impl<'a> Function<'a> {
    /// Parse a function.
    ///
    /// # Parameters
    ///
    /// * `sig` - Function signature.
    /// * `name` - Function name in PHP land.
    /// * `args` - Function arguments.
    /// * `optional` - The ident of the first optional argument.
    pub fn parse(
        sig: &'a syn::Signature,
        name: Option<String>,
        args: Args<'a>,
        optional: Option<Ident>,
    ) -> Result<Self> {
        Ok(Self {
            ident: &sig.ident,
            name: name.unwrap_or_else(|| sig.ident.to_string()),
            args,
            output: match &sig.output {
                syn::ReturnType::Default => None,
                syn::ReturnType::Type(_, ty) => Some(&**ty),
            },
            optional,
        })
    }

    /// Generates an internal identifier for the function.
    pub fn internal_ident(&self) -> Ident {
        format_ident!("_internal_{}", &self.ident)
    }

    /// Generates the function builder for the function.
    pub fn function_builder(&self, call_type: CallType) -> Result<TokenStream> {
        let ident = self.ident;
        let name = &self.name;
        let (required, not_required) = self.args.split_args(self.optional.as_ref());

        // `handler` impl
        let required_arg_names: Vec<_> = required.iter().map(|arg| arg.name).collect();
        let not_required_arg_names: Vec<_> = not_required.iter().map(|arg| arg.name).collect();
        let arg_declerations = self
            .args
            .typed
            .iter()
            .map(TypedArg::arg_decleration)
            .collect::<Result<Vec<_>>>()?;
        let arg_accessors = self.args.typed.iter().map(|arg| {
            arg.accessor(|e| {
                quote! {
                    #e.throw().expect("Failed to throw PHP exception.");
                    return;
                }
            })
        });

        // `entry` impl
        let required_args = required
            .iter()
            .map(TypedArg::arg_builder)
            .collect::<Result<Vec<_>>>()?;
        let not_required_args = not_required
            .iter()
            .map(TypedArg::arg_builder)
            .collect::<Result<Vec<_>>>()?;
        let output = self.output.as_ref().map(|output| {
            quote! {
                .returns(
                    <#output as ::ext_php_rs::convert::IntoZval>::TYPE,
                    false,
                    <#output as ::ext_php_rs::convert::IntoZval>::NULLABLE,
                )
            }
        });

        let result = match call_type {
            CallType::Function => quote! {
                let parse = ex.parser()
                    #(.arg(&mut #required_arg_names))*
                    .not_required()
                    #(.arg(&mut #not_required_arg_names))*
                    .parse();
                if parse.is_err() {
                    return;
                }

                #ident(#({#arg_accessors}),*)
            },
            CallType::Method { class, receiver } => {
                let this = match receiver {
                    MethodReceiver::Static => quote! {
                        let parse = ex.parser();
                    },
                    MethodReceiver::ZendClassObject | MethodReceiver::Class => quote! {
                        let (parse, this) = ex.parser_method::<#class>();
                        let this = match this {
                            Some(this) => this,
                            None => {
                                ::ext_php_rs::exception::PhpException::default("Failed to retrieve reference to `$this`".into())
                                    .throw()
                                    .unwrap();
                                return;
                            }
                        };
                    },
                };
                let call = match receiver {
                    MethodReceiver::Static => {
                        quote! { #class::#ident(#({#arg_accessors}),*) }
                    }
                    MethodReceiver::Class => quote! { this.#ident(#({#arg_accessors}),*) },
                    MethodReceiver::ZendClassObject => {
                        quote! { #class::#ident(this, #({#arg_accessors}),*) }
                    }
                };
                quote! {
                    #this
                    let parse_result = parse
                        #(.arg(&mut #required_arg_names))*
                        .not_required()
                        #(.arg(&mut #not_required_arg_names))*
                        .parse();
                    if parse_result.is_err() {
                        return;
                    }

                    #call
                }
            }
        };

        Ok(quote! {
            ::ext_php_rs::builders::FunctionBuilder::new(#name, {
                ::ext_php_rs::zend_fastcall! {
                    extern fn handler(
                        ex: &mut ::ext_php_rs::zend::ExecuteData,
                        retval: &mut ::ext_php_rs::types::Zval,
                    ) {
                        use ::ext_php_rs::convert::IntoZval;

                        #(#arg_declerations)*
                        let result = {
                            #result
                        };

                        if let Err(e) = result.set_zval(retval, false) {
                            let e: ::ext_php_rs::exception::PhpException = e.into();
                            e.throw().expect("Failed to throw PHP exception.");
                        }
                    }
                }
                handler
            })
            #(.arg(#required_args))*
            .not_required()
            #(.arg(#not_required_args))*
            #output
        })
    }

    /// Generates a struct and impl for the `PhpFunction` trait.
    pub fn php_function_impl(&self) -> Result<TokenStream> {
        let internal_ident = self.internal_ident();
        let builder = self.function_builder(CallType::Function)?;

        Ok(quote! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            struct #internal_ident;

            impl ::ext_php_rs::internal::function::PhpFunction for #internal_ident {
                const FUNCTION_ENTRY: fn() -> ::ext_php_rs::error::Result<
                    ::ext_php_rs::zend::FunctionEntry
                > = {
                    fn entry() -> ::ext_php_rs::error::Result<
                        ::ext_php_rs::zend::FunctionEntry
                    > {
                        #builder.build()
                    }
                    entry
                };
            }
        })
    }

    /// Returns a constructor metadata object for this function. This doesn't
    /// check if the function is a constructor, however.
    pub fn constructor_meta(&self, class: &syn::Path) -> Result<TokenStream> {
        let ident = self.ident;
        let (required, not_required) = self.args.split_args(self.optional.as_ref());
        let required_args = required
            .iter()
            .map(TypedArg::arg_builder)
            .collect::<Result<Vec<_>>>()?;
        let not_required_args = not_required
            .iter()
            .map(TypedArg::arg_builder)
            .collect::<Result<Vec<_>>>()?;

        let required_arg_names: Vec<_> = required.iter().map(|arg| arg.name).collect();
        let not_required_arg_names: Vec<_> = not_required.iter().map(|arg| arg.name).collect();
        let arg_declerations = self
            .args
            .typed
            .iter()
            .map(TypedArg::arg_decleration)
            .collect::<Result<Vec<_>>>()?;
        let arg_accessors = self.args.typed.iter().map(|arg| {
            arg.accessor(
                |e| quote! { return ::ext_php_rs::class::ConstructorResult::Exception(#e); },
            )
        });

        Ok(quote! {
            ::ext_php_rs::class::ConstructorMeta {
                constructor: {
                    fn inner(ex: &mut ::ext_php_rs::zend::ExecuteData) -> ::ext_php_rs::class::ConstructorResult<#class> {
                        #(#arg_declerations)*
                        let parse = ex.parser()
                            #(.arg(&mut #required_arg_names))*
                            .not_required()
                            #(.arg(&mut #not_required_arg_names))*
                            .parse();
                        if parse.is_err() {
                            return ::ext_php_rs::class::ConstructorResult::ArgError;
                        }
                        #class::#ident(#({#arg_accessors}),*).into()
                    }
                    inner
                },
                build_fn: {
                    fn inner(func: ::ext_php_rs::builders::FunctionBuilder) -> ::ext_php_rs::builders::FunctionBuilder {
                        func
                            #(.arg(#required_args))*
                            .not_required()
                            #(.arg(#not_required_args))*
                    }
                    inner
                }
            }
        })
    }
}

#[derive(Debug)]
pub struct ReceiverArg {
    pub mutable: bool,
    pub span: Span,
}

#[derive(Debug)]
pub struct TypedArg<'a> {
    pub name: &'a Ident,
    pub ty: &'a Type,
    pub nullable: bool,
    pub default: Option<Lit>,
}

#[derive(Debug)]
pub struct Args<'a> {
    pub receiver: Option<ReceiverArg>,
    pub typed: Vec<TypedArg<'a>>,
}

impl<'a> Args<'a> {
    pub fn parse_from_fnargs(
        args: impl Iterator<Item = &'a FnArg>,
        mut defaults: HashMap<Ident, Lit>,
    ) -> Result<Self> {
        let mut result = Self {
            receiver: None,
            typed: vec![],
        };
        for arg in args {
            match arg {
                FnArg::Receiver(receiver) => {
                    if receiver.reference.is_none() {
                        bail!(receiver => "PHP objects are heap-allocated and cannot be passed by value. Try using `&self` or `&mut self`.");
                    } else if result.receiver.is_some() {
                        bail!(receiver => "Too many receivers specified.")
                    }
                    result.receiver.replace(ReceiverArg {
                        mutable: receiver.mutability.is_some(),
                        span: receiver.span(),
                    });
                }
                FnArg::Typed(PatType { pat, ty, .. }) => {
                    let ident = match &**pat {
                        syn::Pat::Ident(syn::PatIdent { ident, .. }) => ident,
                        _ => bail!(pat => "Unsupported argument."),
                    };
                    let nullable = type_is_nullable(&**ty)?;
                    result.typed.push(TypedArg {
                        name: ident,
                        ty: &**ty,
                        nullable,
                        default: defaults.remove(ident),
                    });
                }
            }
        }
        Ok(result)
    }

    /// Splits the typed arguments into two slices:
    ///
    /// 1. Required arguments.
    /// 2. Non-required arguments.
    ///
    /// # Parameters
    ///
    /// * `optional` - The first optional argument. If [`None`], the optional
    ///   arguments will be from the first nullable argument after the last
    ///   non-nullable argument to the end of the arguments.
    pub fn split_args(&self, optional: Option<&Ident>) -> (&[TypedArg<'a>], &[TypedArg<'a>]) {
        let mut mid = None;
        for (i, arg) in self.typed.iter().enumerate() {
            if let Some(optional) = optional {
                if optional == arg.name {
                    mid.replace(i);
                }
            } else if mid.is_none() && arg.nullable {
                mid.replace(i);
            } else if !arg.nullable {
                mid.take();
            }
        }
        match mid {
            Some(mid) => (&self.typed[..mid], &self.typed[mid..]),
            None => (&self.typed[..], &self.typed[0..0]),
        }
    }
}

impl<'a> TypedArg<'a> {
    /// Returns a 'clean type' with the lifetimes removed. This allows the type
    /// to be used outside of the original function context.
    fn clean_ty(&self) -> Type {
        let mut ty = self.ty.clone();
        ty.drop_lifetimes();
        ty
    }

    /// Returns a token stream containing an argument decleration, where the
    /// name of the variable holding the arg is the name of the argument.
    fn arg_decleration(&self) -> Result<TokenStream> {
        let name = self.name;
        let val = self.arg_builder()?;
        Ok(quote! {
            let mut #name = #val;
        })
    }

    /// Returns a token stream containing the `Arg` definition to be passed to
    /// `ext-php-rs`.
    fn arg_builder(&self) -> Result<TokenStream> {
        let name = self.name.to_string();
        let ty = self.clean_ty();
        let null = if self.nullable {
            Some(quote! { .allow_null() })
        } else {
            None
        };
        let default = self.default.as_ref().map(|val| {
            let val = val.to_token_stream().to_string();
            quote! {
                .default(#val)
            }
        });
        Ok(quote! {
            ::ext_php_rs::args::Arg::new(#name, <#ty as ::ext_php_rs::convert::FromZval>::TYPE)
                #null
                #default
        })
    }

    /// Get the accessor used to access the value of the argument.
    fn accessor(&self, bail_fn: impl Fn(TokenStream) -> TokenStream) -> TokenStream {
        let name = self.name;
        if let Some(default) = &self.default {
            quote! {
                #name.val().unwrap_or(#default)
            }
        } else if self.nullable {
            // Originally I thought we could just use the below case for `null` options, as
            // `val()` will return `Option<Option<T>>`, however, this isn't the case when
            // the argument isn't given, as the underlying zval is null.
            quote! {
                #name.val()
            }
        } else {
            let bail = bail_fn(quote! {
                ::ext_php_rs::exception::PhpException::default(
                    concat!("Invalid value given for argument `", stringify!(#name), "`.").into()
                )
            });
            quote! {
                match #name.val() {
                    Some(val) => val,
                    None => {
                        #bail;
                    }
                }
            }
        }
    }
}

/// Returns true of the given type is nullable in PHP.
// TODO(david): Eventually move to compile-time constants for this (similar to
// FromZval::NULLABLE).
pub fn type_is_nullable(ty: &Type) -> Result<bool> {
    Ok(match ty {
        syn::Type::Path(path) => path
            .path
            .segments
            .iter()
            .last()
            .map(|seg| seg.ident == "Option")
            .unwrap_or(false),
        syn::Type::Reference(_) => false, /* Reference cannot be nullable unless */
        // wrapped in `Option` (in that case it'd be a Path).
        _ => bail!(ty => "Unsupported argument type."),
    })
}
