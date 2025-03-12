use darling::ast::NestedMeta;
use darling::{FromMeta, ToTokens};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse::ParseStream;
use syn::{Attribute, Expr, Fields, ItemStruct, LitStr, Meta, Token};

use crate::helpers::get_docs;
use crate::prelude::*;

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
    docs: Vec<String>,
}

impl ClassAttrs {
    fn parse(&mut self, attrs: &mut Vec<syn::Attribute>) -> Result<()> {
        let mut unparsed = vec![];
        unparsed.append(attrs);
        for attr in unparsed {
            let path = attr.path();

            if path.is_ident("extends") {
                if self.extends.is_some() {
                    bail!(attr => "Only one `#[extends]` attribute is valid per struct.");
                }
                let extends: syn::Expr = match attr.parse_args() {
                    Ok(extends) => extends,
                    Err(_) => bail!(attr => "Invalid arguments passed to extends attribute."),
                };
                self.extends = Some(extends);
            } else if path.is_ident("implements") {
                let implements: syn::Expr = match attr.parse_args() {
                    Ok(extends) => extends,
                    Err(_) => bail!(attr => "Invalid arguments passed to implements attribute."),
                };
                self.implements.push(implements);
            } else {
                attrs.push(attr);
            }
        }
        self.docs = get_docs(attrs);
        Ok(())
    }
}

pub fn parser(args: TokenStream, mut input: ItemStruct) -> Result<TokenStream> {
    let ident = &input.ident;
    let meta = NestedMeta::parse_meta_list(args)?;
    let args = match StructArgs::from_list(&meta) {
        Ok(args) => args,
        Err(e) => bail!("Failed to parse struct arguments: {:?}", e),
    };

    let mut class_attrs = ClassAttrs::default();
    class_attrs.parse(&mut input.attrs)?;

    let fields = match &mut input.fields {
        Fields::Named(fields) => parse_fields(fields.named.iter_mut())?,
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
        &class_attrs.docs,
    );

    Ok(quote! {
        #input
        #class_impl

        ::ext_php_rs::class_derives!(#ident);
    })
}

fn parse_fields<'a>(fields: impl Iterator<Item = &'a mut syn::Field>) -> Result<Vec<Property<'a>>> {
    #[derive(Debug, Default, FromMeta)]
    #[darling(default)]
    struct FieldAttr {
        rename: Option<String>,
    }

    let mut result = vec![];
    for field in fields {
        let mut docs = vec![];
        let mut property = None;
        let mut unparsed = vec![];
        unparsed.append(&mut field.attrs);

        for attr in unparsed {
            if let Some(parsed) = parse_attribute(&attr)? {
                match parsed {
                    ParsedAttribute::Property(prop) => {
                        let ident = field
                            .ident
                            .as_ref()
                            .ok_or_else(|| err!(attr => "Only named fields can be properties."))?;

                        property = Some((ident, prop));
                    }
                    ParsedAttribute::Comment(doc) => docs.push(doc),
                }
            } else {
                field.attrs.push(attr);
            }
        }

        if let Some((ident, prop)) = property {
            result.push(Property {
                ident,
                attr: prop,
                docs,
            });
        }
    }

    Ok(result)
}

#[derive(Debug)]
pub struct Property<'a> {
    pub ident: &'a syn::Ident,
    pub attr: PropertyAttr,
    pub docs: Vec<String>,
}

impl Property<'_> {
    pub fn name(&self) -> String {
        self.attr
            .rename
            .to_owned()
            .unwrap_or_else(|| self.ident.to_string())
    }
}

#[derive(Debug, Default)]
pub struct PropertyAttr {
    pub rename: Option<String>,
    pub flags: Option<Expr>,
}

impl syn::parse::Parse for PropertyAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut this = Self::default();
        while !input.is_empty() {
            let field = input.parse::<Ident>()?.to_string();
            input.parse::<Token![=]>()?;

            match field.as_str() {
                "rename" => {
                    this.rename.replace(input.parse::<LitStr>()?.value());
                }
                "flags" => {
                    this.flags.replace(input.parse::<Expr>()?);
                }
                _ => return Err(input.error("invalid attribute field")),
            }

            let _ = input.parse::<Token![,]>();
        }

        Ok(this)
    }
}

#[derive(Debug)]
pub enum ParsedAttribute {
    Property(PropertyAttr),
    Comment(String),
}

pub fn parse_attribute(attr: &Attribute) -> Result<Option<ParsedAttribute>> {
    let name = attr.path().to_token_stream().to_string();

    Ok(match name.as_ref() {
        "doc" => {
            struct DocComment(pub String);

            impl syn::parse::Parse for DocComment {
                fn parse(input: ParseStream) -> syn::Result<Self> {
                    input.parse::<Token![=]>()?;
                    let comment: LitStr = input.parse()?;
                    Ok(Self(comment.value()))
                }
            }

            let comment: DocComment = syn::parse2(attr.to_token_stream())
                .map_err(|e| err!(attr => "Failed to parse doc comment {}", e))?;
            Some(ParsedAttribute::Comment(comment.0))
        }
        "prop" | "property" => {
            let attr = match attr.meta {
                Meta::Path(_) => PropertyAttr::default(),
                Meta::List(_) => attr
                    .parse_args()
                    .map_err(|e| err!(attr => "Unable to parse `#[{}]` attribute: {}", name, e))?,
                _ => {
                    bail!(attr => "Invalid attribute format for `#[{}]`", name);
                }
            };

            Some(ParsedAttribute::Property(attr))
        }
        _ => None,
    })
}

/// Generates an implementation of `RegisteredClass` for struct `ident`.
#[allow(clippy::too_many_arguments)]
fn generate_registered_class_impl(
    ident: &syn::Ident,
    class_name: Option<&str>,
    modifier: Option<&syn::Ident>,
    extends: Option<&syn::Expr>,
    implements: &[syn::Expr],
    fields: &[Property],
    flags: Option<&syn::Expr>,
    docs: &[String],
) -> TokenStream {
    let ident_str = ident.to_string();
    let class_name = match class_name {
        Some(class_name) => class_name,
        None => &ident_str,
    };
    let modifier = modifier.option_tokens();
    let extends = extends.option_tokens();

    let fields = fields.iter().map(|prop| {
        let name = prop.name();
        let ident = prop.ident;
        let flags = prop
            .attr
            .flags
            .as_ref()
            .map(|flags| flags.to_token_stream())
            .unwrap_or(quote! { ::ext_php_rs::flags::PropertyFlags::Public });
        let docs = &prop.docs;

        quote! {
            (#name, ::ext_php_rs::internal::property::PropertyInfo {
                prop: ::ext_php_rs::props::Property::field(|this: &mut Self| &mut this.#ident),
                flags: #flags,
                docs: &[#(#docs,)*]
            })
        }
    });

    let flags = match flags {
        Some(flags) => flags.to_token_stream(),
        None => quote! { ::ext_php_rs::flags::ClassFlags::empty() }.to_token_stream(),
    };

    let docs = quote! {
        #(#docs)*
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
            const DOC_COMMENTS: &'static [&'static str] = &[
                #docs
            ];

            #[inline]
            fn get_metadata() -> &'static ::ext_php_rs::class::ClassMetadata<Self> {
                static METADATA: ::ext_php_rs::class::ClassMetadata<#ident> =
                    ::ext_php_rs::class::ClassMetadata::new();
                &METADATA
            }

            fn get_properties<'a>() -> ::std::collections::HashMap<
                &'static str, ::ext_php_rs::internal::property::PropertyInfo<'a, Self>
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
            fn constants() -> &'static [(&'static str, &'static dyn ::ext_php_rs::convert::IntoZvalDyn, &'static [&'static str])] {
                use ::ext_php_rs::internal::class::PhpClassImpl;
                ::ext_php_rs::internal::class::PhpClassImplCollector::<Self>::default().get_constants()
            }
        }
    }
}
