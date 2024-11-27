use std::collections::HashMap;

use anyhow::Result;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Item, ItemMod, Type};

use crate::{
    class::{Class, Property},
    constant::Constant,
    function::{Arg, Function},
    module_builder::ModuleBuilder,
};

pub fn parser(input: ItemMod) -> Result<TokenStream> {
    if input.ident != "module" {
        return Ok(
            quote! { compile_error!("The `php_module` attribute must be used on a module named `module`."); },
        );
    }
    if input.content.is_none() {
        return Ok(quote! {});
    }

    let mut builder = ModuleBuilder::new();
    let (_, content) = &input.content.expect("module content is missing");
    for item in content {
        match item {
            Item::Fn(f) => {
                if f.attrs.iter().any(|a| a.path.is_ident("php_startup")) {
                    builder.set_startup_function(f.clone());
                    continue;
                } else if f.attrs.iter().any(|a| a.path.is_ident("php_function")) {
                    builder.add_function(f.clone());
                    continue;
                }
            }
            Item::Const(c) => {
                builder.add_constant(c.clone());
                continue;
            }
            Item::Struct(s) => {
                if s.attrs.iter().any(|a| a.path.is_ident("php_class")) {
                    builder.add_class(s.clone());
                    continue;
                }
            }
            Item::Impl(i) => {
                if i.attrs.iter().any(|a| a.path.is_ident("php_impl")) {
                    builder.add_implementation(i.clone());
                    continue;
                }
            }
            _ => {}
        }
        builder.add_unmapped(item.clone());
    }

    // let ItemFn { sig, block, .. } = input;
    // let Signature { output, inputs, .. } = sig;
    // let stmts = &block.stmts;

    // let registered_classes_impls = state
    //     .classes
    //     .values()
    //     .map(generate_registered_class_impl)
    //     .collect::<Result<Vec<_>>>()?;
    // let describe_fn = generate_stubs(&state);

    Ok(builder.build())
}

/// Generates an implementation for `RegisteredClass` on the given class.
pub fn generate_registered_class_impl(class: &Class) -> Result<TokenStream> {
    let self_ty = Ident::new(&class.struct_path, Span::call_site());
    let class_name = &class.class_name;
    let meta = Ident::new(&format!("_{}_META", &class.struct_path), Span::call_site());
    let prop_tuples = class
        .properties
        .iter()
        .map(|(name, prop)| prop.as_prop_tuple(name));
    let constructor = if let Some(constructor) = &class.constructor {
        let func = Ident::new(&constructor.ident, Span::call_site());
        let args = constructor.get_arg_definitions();
        quote! {
            Some(::ext_php_rs::class::ConstructorMeta {
                constructor: Self::#func,
                build_fn: {
                    use ::ext_php_rs::builders::FunctionBuilder;
                    fn build_fn(func: FunctionBuilder) -> FunctionBuilder {
                        func
                        #(#args)*
                    }
                    build_fn
                }
            })
        }
    } else {
        quote! { None }
    };

    Ok(quote! {
        static #meta: ::ext_php_rs::class::ClassMetadata<#self_ty> = ::ext_php_rs::class::ClassMetadata::new();

        impl ::ext_php_rs::class::RegisteredClass for #self_ty {
            const CLASS_NAME: &'static str = #class_name;
            const CONSTRUCTOR: ::std::option::Option<
                ::ext_php_rs::class::ConstructorMeta<Self>
            > = #constructor;

            fn get_metadata() -> &'static ::ext_php_rs::class::ClassMetadata<Self> {
                &#meta
            }

            fn get_properties<'a>() -> ::std::collections::HashMap<&'static str, ::ext_php_rs::props::Property<'a, Self>> {
                use ::std::iter::FromIterator;

                ::std::collections::HashMap::from_iter([
                    #(#prop_tuples)*
                ])
            }
        }
    })
}

pub trait Describe {
    fn describe(&self) -> TokenStream;
}

pub(crate) fn generate_stubs(
    functions: &[Function],
    classes: &HashMap<String, Class>,
    constants: &[Constant],
) -> TokenStream {
    let module = (functions, classes, constants).describe();

    quote! {
        #[cfg(debug_assertions)]
        #[no_mangle]
        pub extern "C" fn ext_php_rs_describe_module() -> ::ext_php_rs::describe::Description {
            use ::ext_php_rs::describe::*;

            Description::new(#module)
        }
    }
}

impl Describe for Function {
    fn describe(&self) -> TokenStream {
        let name = &self.name;
        let ret = if let Some((ty, null)) = &self.output {
            let ty: Type = syn::parse_str(ty)
                .expect("unreachable - failed to parse previously parsed function return type");
            quote! {
                Some(Retval {
                    ty: <#ty as ::ext_php_rs::convert::IntoZval>::TYPE,
                    nullable: #null,
                })
            }
        } else {
            quote! { None }
        };
        let params = self.args.iter().map(Describe::describe);
        let docs = self.docs.iter().map(|doc| {
            quote! {
                #doc.into()
            }
        });

        quote! {
            Function {
                name: #name.into(),
                docs: DocBlock(vec![#(#docs,)*].into()),
                ret: abi::Option::#ret,
                params: vec![#(#params,)*].into(),
            }
        }
    }
}

impl Describe for Arg {
    fn describe(&self) -> TokenStream {
        let Arg { name, nullable, .. } = self;
        let ty: Type = syn::parse_str(&self.ty).expect("failed to parse previously parsed type");

        let mut ty =
            quote! { abi::Option::Some(<#ty as ::ext_php_rs::convert::FromZvalMut>::TYPE) };
        if self.variadic {
            ty = quote! { abi::Option::Some(::ext_php_rs::flags::DataType::Array) }
        }
        let default = if let Some(default) = &self.default {
            quote! { Some(#default.into()) }
        } else {
            quote! { None }
        };

        quote! {
            Parameter {
                name: #name.into(),
                ty: #ty,
                nullable: #nullable,
                default: abi::Option::#default,
            }
        }
    }
}

impl Describe for Class {
    fn describe(&self) -> TokenStream {
        let name = &self.class_name;
        let extends = if let Some(parent) = &self.parent {
            quote! { Some(#parent.into()) }
        } else {
            quote! { None }
        };
        let interfaces = self.interfaces.iter().map(|iface| quote! { #iface.into() });
        let properties = self.properties.iter().map(|d| d.describe());
        let mut methods: Vec<_> = self.methods.iter().map(Describe::describe).collect();
        let docs = self.docs.iter().map(|c| {
            quote! {
                #c.into()
            }
        });
        let constants = self.constants.iter().map(Describe::describe);

        if let Some(ctor) = &self.constructor {
            methods.insert(0, ctor.describe());
        }

        quote! {
            Class {
                name: #name.into(),
                docs: DocBlock(vec![#(#docs,)*].into()),
                extends: abi::Option::#extends,
                implements: vec![#(#interfaces,)*].into(),
                properties: vec![#(#properties,)*].into(),
                methods: vec![#(#methods,)*].into(),
                constants: vec![#(#constants,)*].into(),
            }
        }
    }
}

impl Describe for (&String, &Property) {
    fn describe(&self) -> TokenStream {
        let name = self.0;
        let docs = self.1.docs.iter().map(|doc| {
            quote! {
                #doc.into()
            }
        });

        // TODO(david): store metadata for ty, vis, static, null, default
        quote! {
            Property {
                name: #name.into(),
                docs: DocBlock(vec![#(#docs,)*].into()),
                ty: abi::Option::None,
                vis: Visibility::Public,
                static_: false,
                nullable: false,
                default: abi::Option::None,
            }
        }
    }
}

impl Describe for crate::method::Method {
    fn describe(&self) -> TokenStream {
        let crate::method::Method { name, _static, .. } = &self;
        let ty = if self.name == "__construct" {
            quote! { MethodType::Constructor }
        } else if self._static {
            quote! { MethodType::Static }
        } else {
            quote! { MethodType::Member }
        };
        let parameters = self.args.iter().filter_map(|arg| {
            if let crate::method::Arg::Typed(arg) = &arg {
                Some(arg.describe())
            } else {
                None
            }
        });
        let ret = if let Some((ty, null)) = &self.output {
            let ty: Type = syn::parse_str(ty).expect("failed to parse previously parsed type");
            quote! {
                Some(Retval {
                    ty: <#ty as ::ext_php_rs::convert::IntoZval>::TYPE,
                    nullable: #null,
                })
            }
        } else {
            quote! { None }
        };
        let vis = self.visibility.describe();
        let docs = self.docs.iter().map(|doc| {
            quote! {
                #doc.into()
            }
        });

        quote! {
            Method {
                name: #name.into(),
                docs: DocBlock(vec![#(#docs,)*].into()),
                ty: #ty,
                params: vec![#(#parameters,)*].into(),
                retval: abi::Option::#ret,
                _static: #_static,
                visibility: #vis,
            }
        }
    }
}

impl Describe for crate::impl_::Visibility {
    fn describe(&self) -> TokenStream {
        match self {
            crate::impl_::Visibility::Public => quote! { Visibility::Public },
            crate::impl_::Visibility::Protected => quote! { Visibility::Protected },
            crate::impl_::Visibility::Private => quote! { Visibility::Private },
        }
    }
}

impl Describe for crate::constant::Constant {
    fn describe(&self) -> TokenStream {
        let name = &self.name;
        let docs = self.docs.iter().map(|doc| {
            quote! {
                #doc.into()
            }
        });

        quote! {
            Constant {
                name: #name.into(),
                docs: DocBlock(vec![#(#docs,)*].into()),
                value: abi::Option::None,
            }
        }
    }
}

impl Describe for (&[Function], &HashMap<String, Class>, &[Constant]) {
    fn describe(&self) -> TokenStream {
        let functs = self.0.iter().map(Describe::describe);
        let classes = self.1.values().map(|class| class.describe());
        let constants = self.2.iter().map(Describe::describe);

        quote! {
            Module {
                name: env!("CARGO_PKG_NAME").into(),
                functions: vec![#(#functs,)*].into(),
                classes: vec![#(#classes,)*].into(),
                constants: vec![#(#constants,)*].into(),
            }
        }
    }
}
