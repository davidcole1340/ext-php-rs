use crate::prelude::*;
use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::AttributeArgs;

#[derive(Debug, Default, FromMeta)]
#[darling(default)]
pub struct StructArgs {
    /// The name of the PHP class. Defaults to the same name as the struct.
    name: Option<String>,
    /// A modifier function which should accept one argument, a `ClassBuilder`,
    /// and return the same object. Allows the user to modify the class before
    /// it is built.
    modifier: Option<syn::Ident>,
    /// An expression of `ClassFlags` to be applied to the class.
    flags: Option<syn::Expr>,
}

/// Sub-attributes which are parsed by this macro. Must be placed underneath the
/// main `#[php_class]` attribute.
#[derive(Debug, Default)]
struct ClassAttrs {
    extends: Option<syn::Expr>,
    implements: Vec<syn::Expr>,
}

impl ClassAttrs {
    fn parse(&mut self, attrs: &mut Vec<syn::Attribute>) -> Result<()> {
        let mut unparsed = vec![];
        unparsed.append(attrs);
        for attr in unparsed {
            if attr.path.is_ident("extends") {
                if self.extends.is_some() {
                    bail!(attr => "Only one `#[extends]` attribute is valid per struct.");
                }
                let extends: syn::Expr = match attr.parse_args() {
                    Ok(extends) => extends,
                    Err(_) => bail!(attr => "Invalid arguments passed to extends attribute."),
                };
                self.extends = Some(extends);
            } else if attr.path.is_ident("implements") {
                let implements: syn::Expr = match attr.parse_args() {
                    Ok(extends) => extends,
                    Err(_) => bail!(attr => "Invalid arguments passed to implements attribute."),
                };
                self.implements.push(implements);
            } else {
                attrs.push(attr);
            }
        }
        Ok(())
    }
}

pub fn parser(args: AttributeArgs, mut input: syn::ItemStruct) -> Result<TokenStream> {
    let ident = &input.ident;
    let args = match StructArgs::from_list(&args) {
        Ok(args) => args,
        Err(e) => bail!("Failed to parse struct arguments: {:?}", e),
    };
    let mut class_attrs = ClassAttrs::default();
    class_attrs.parse(&mut input.attrs)?;

    let fields = match &mut input.fields {
        syn::Fields::Named(fields) => parse_fields(fields.named.iter_mut())?,
        _ => vec![],
    };
    let class_impl = generate_registered_class_impl(
        ident,
        args.name.as_deref(),
        args.modifier.as_ref(),
        class_attrs.extends.as_ref(),
        &class_attrs.implements,
        &fields,
        args.flags.as_ref(),
    );

    Ok(quote! {
        #input
        #class_impl
        ::ext_php_rs::class_derives!(#ident);
    })
}

fn parse_fields<'a>(
    fields: impl Iterator<Item = &'a mut syn::Field>,
) -> Result<Vec<(String, &'a syn::Ident)>> {
    #[derive(Debug, Default, FromMeta)]
    #[darling(default)]
    struct FieldAttr {
        rename: Option<String>,
    }

    let mut result = vec![];
    for field in fields {
        let mut unparsed = vec![];
        unparsed.append(&mut field.attrs);
        for attr in unparsed {
            if attr.path.is_ident("prop") {
                let meta = match attr.parse_meta() {
                    Ok(meta) => meta,
                    Err(_) => bail!(attr => "Failed to parse attribute arguments"),
                };
                let ident = field
                    .ident
                    .as_ref()
                    .expect("Named field struct should have ident.");
                let field_name = match meta {
                    syn::Meta::List(_) => FieldAttr::from_meta(&meta).unwrap(),
                    _ => FieldAttr::default(),
                }
                .rename
                .unwrap_or_else(|| ident.to_string());
                result.push((field_name, ident))
            } else {
                field.attrs.push(attr);
            }
        }
    }
    Ok(result)
}

/// Generates an implementation of `RegisteredClass` for struct `ident`.
fn generate_registered_class_impl(
    ident: &syn::Ident,
    class_name: Option<&str>,
    modifier: Option<&syn::Ident>,
    extends: Option<&syn::Expr>,
    implements: &[syn::Expr],
    fields: &[(String, &syn::Ident)],
    flags: Option<&syn::Expr>,
) -> TokenStream {
    let ident_str = ident.to_string();
    let class_name = match class_name {
        Some(class_name) => class_name,
        None => &ident_str,
    };
    let modifier = modifier.option_tokens();
    let extends = extends.option_tokens();
    let fields = fields.iter().map(|(name, ident)| {
        quote! {
            (#name, ::ext_php_rs::props::Property::field(|this: &mut Self| &mut this.#ident))
        }
    });
    let flags = match flags {
        Some(flags) => flags.to_token_stream(),
        None => quote! { ::ext_php_rs::flags::ClassFlags::empty() }.to_token_stream(),
    };
    quote! {
        impl ::ext_php_rs::class::RegisteredClass for #ident {
            const CLASS_NAME: &'static str = #class_name;
            const BUILDER_MODIFIER: ::std::option::Option<
                fn(::ext_php_rs::builders::ClassBuilder) -> ::ext_php_rs::builders::ClassBuilder
            > = #modifier;
            const EXTENDS: ::std::option::Option<
                fn() -> &'static ::ext_php_rs::zend::ClassEntry
            > = #extends;
            const IMPLEMENTS: &'static [fn() -> &'static ::ext_php_rs::zend::ClassEntry] = &[
                #(#implements,)*
            ];
            const FLAGS: ::ext_php_rs::flags::ClassFlags = #flags;

            #[inline]
            fn get_metadata() -> &'static ::ext_php_rs::class::ClassMetadata<Self> {
                static METADATA: ::ext_php_rs::class::ClassMetadata<#ident> =
                    ::ext_php_rs::class::ClassMetadata::new();
                &METADATA
            }

            fn get_properties<'a>() -> ::std::collections::HashMap<
                &'static str, ::ext_php_rs::props::Property<'a, Self>
            > {
                use ::std::iter::FromIterator;
                ::std::collections::HashMap::from_iter([
                    #(#fields,)*
                ])
            }

            #[inline]
            fn method_builders() -> ::std::vec::Vec<
                (::ext_php_rs::builders::FunctionBuilder<'static>, ::ext_php_rs::flags::MethodFlags)
            > {
                use ::ext_php_rs::internal::class::PhpClassImpl;
                ::ext_php_rs::internal::class::PhpClassImplCollector::<Self>::default().get_methods()
            }

            #[inline]
            fn constructor() -> ::std::option::Option<::ext_php_rs::class::ConstructorMeta<Self>> {
                use ::ext_php_rs::internal::class::PhpClassImpl;
                ::ext_php_rs::internal::class::PhpClassImplCollector::<Self>::default().get_constructor()
            }

            #[inline]
            fn constants() -> &'static [(&'static str, &'static dyn ::ext_php_rs::convert::IntoZvalDyn)] {
                use ::ext_php_rs::internal::class::PhpClassImpl;
                ::ext_php_rs::internal::class::PhpClassImplCollector::<Self>::default().get_constants()
            }
        }
    }
}
