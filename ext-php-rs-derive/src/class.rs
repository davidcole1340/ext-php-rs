use std::collections::HashMap;

use crate::STATE;
use anyhow::{anyhow, bail, Result};
use darling::{FromMeta, ToTokens};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Attribute, AttributeArgs, Expr, ItemStruct, Token};

#[derive(Debug, Default)]
pub struct Class {
    pub class_name: String,
    pub parent: Option<String>,
    pub interfaces: Vec<String>,
    pub methods: Vec<crate::method::Method>,
    pub constants: Vec<crate::constant::Constant>,
    pub properties: HashMap<String, (String, Option<String>)>,
}

#[derive(Debug)]
pub enum ParsedAttribute {
    Extends(Expr),
    Implements(Expr),
    Property(Box<PropertyAttr>),
}

#[derive(Default, Debug, FromMeta)]
#[darling(default)]
pub struct AttrArgs {
    name: Option<String>,
}

pub fn parser(args: AttributeArgs, mut input: ItemStruct) -> Result<TokenStream> {
    let args = AttrArgs::from_list(&args)
        .map_err(|e| anyhow!("Unable to parse attribute arguments: {:?}", e))?;

    let mut parent = None;
    let mut interfaces = vec![];
    let mut properties = HashMap::<String, (String, Option<String>)>::new();

    input.attrs = {
        let mut unused = vec![];
        for attr in input.attrs.into_iter() {
            match parse_attribute(&attr)? {
                Some(parsed) => match parsed {
                    ParsedAttribute::Extends(class) => {
                        parent = Some(class.to_token_stream().to_string());
                    }
                    ParsedAttribute::Implements(class) => {
                        interfaces.push(class.to_token_stream().to_string());
                    }
                    ParsedAttribute::Property(attr) => {
                        properties.insert(
                            attr.name.to_string(),
                            (
                                attr.default.to_token_stream().to_string(),
                                attr.flags.map(|flags| flags.to_token_stream().to_string()),
                            ),
                        );
                    }
                },
                None => unused.push(attr),
            }
        }
        unused
    };

    let ItemStruct { ident, .. } = &input;
    let class_name = args.name.unwrap_or_else(|| ident.to_string());
    let meta = Ident::new(&format!("_{}_META", ident.to_string()), Span::call_site());

    let output = quote! {
        #input

        static #meta: ::ext_php_rs::php::types::object::ClassMetadata<#ident> = ::ext_php_rs::php::types::object::ClassMetadata::new();

        impl ::ext_php_rs::php::types::object::RegisteredClass for #ident {
            fn get_metadata() -> &'static ::ext_php_rs::php::types::object::ClassMetadata<Self> {
                &#meta
            }
        }
    };

    let class = Class {
        class_name,
        parent,
        interfaces,
        properties,
        ..Default::default()
    };

    let mut state = STATE.lock();

    if state.built_module {
        bail!("The `#[php_module]` macro must be called last to ensure functions and classes are registered.");
    }

    if state.startup_function.is_some() {
        bail!("The `#[php_startup]` macro must be called after all the classes have been defined.");
    }

    state.classes.insert(ident.to_string(), class);

    Ok(output)
}

#[derive(Debug)]
pub struct PropertyAttr {
    pub name: Ident,
    pub default: Expr,
    pub flags: Option<Expr>,
}

impl syn::parse::Parse for PropertyAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let _: Token![=] = input.parse()?;
        let default: Expr = input.parse()?;
        let flags = input
            .parse::<Token![,]>()
            .and_then(|_| input.parse::<Expr>())
            .ok();

        Ok(PropertyAttr {
            name,
            default,
            flags,
        })
    }
}

fn parse_attribute(attr: &Attribute) -> Result<Option<ParsedAttribute>> {
    let name = attr.path.to_token_stream().to_string();

    Ok(match name.as_ref() {
        "extends" => {
            let meta: Expr = attr
                .parse_args()
                .map_err(|_| anyhow!("Unable to parse `#[{}]` attribute.", name))?;
            Some(ParsedAttribute::Extends(meta))
        }
        "implements" => {
            let meta: Expr = attr
                .parse_args()
                .map_err(|_| anyhow!("Unable to parse `#[{}]` attribute.", name))?;
            Some(ParsedAttribute::Implements(meta))
        }
        "property" => {
            let attr: PropertyAttr = attr
                .parse_args()
                .map_err(|_| anyhow!("Unable to parse `#[{}]` attribute.", name))?;

            Some(ParsedAttribute::Property(Box::new(attr)))
        }
        _ => None,
    })
}
