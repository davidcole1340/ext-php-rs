use crate::STATE;
use anyhow::{anyhow, bail, Result};
use darling::{FromMeta, ToTokens};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Attribute, AttributeArgs, Expr, Fields, FieldsNamed, ItemStruct, LitStr, Token};

#[derive(Debug, Default)]
pub struct Class {
    pub class_name: String,
    pub parent: Option<String>,
    pub interfaces: Vec<String>,
    pub methods: Vec<crate::method::Method>,
    pub constants: Vec<crate::constant::Constant>,
    pub properties: Vec<Property>,
}

#[derive(Debug)]
pub enum ParsedAttribute {
    Extends(Expr),
    Implements(Expr),
    Property(PropertyAttr),
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
    let mut properties = vec![];

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
                    attr => bail!("Attribute `{:?}` is not valid for structs.", attr),
                },
                None => unused.push(attr),
            }
        }
        unused
    };

    if let Fields::Named(FieldsNamed {
        brace_token: _,
        named,
    }) = &mut input.fields
    {
        for field in named.iter_mut() {
            let mut attrs = vec![];
            attrs.append(&mut field.attrs);
            for attr in attrs.into_iter() {
                match parse_attribute(&attr)? {
                    Some(parsed) => match parsed {
                        ParsedAttribute::Property(prop) => {
                            let field_name = field
                                .ident
                                .as_ref()
                                .ok_or_else(|| anyhow!("Only named fields can be properties."))?
                                .to_string();
                            let prop_name = prop.rename.unwrap_or_else(|| field_name.clone());
                            properties.push(Property {
                                field_name,
                                prop_name,
                                flags: prop.flags.map(|flags| flags.to_token_stream().to_string()),
                            });
                        }
                        _ => bail!("Attribute {:?} is not valid for struct fields.", attr),
                    },
                    None => field.attrs.push(attr),
                }
            }
        }
    }

    let ItemStruct { ident, .. } = &input;
    let class_name = args.name.unwrap_or_else(|| ident.to_string());
    let meta = Ident::new(&format!("_{}_META", ident.to_string()), Span::call_site());
    let prop_tuples = properties.iter().map(
        |Property {
             field_name,
             prop_name,
             flags: _
         }| {
            let field_name = Ident::new(field_name, Span::call_site());
            quote! {
                (#prop_name, &mut self.#field_name as &mut dyn ::ext_php_rs::php::types::object::Prop),
            }
        },
    );

    let output = quote! {
        #input

        static #meta: ::ext_php_rs::php::types::object::ClassMetadata<#ident> = ::ext_php_rs::php::types::object::ClassMetadata::new();

        impl ::ext_php_rs::php::types::object::RegisteredClass for #ident {
            const CLASS_NAME: &'static str = #class_name;

            fn get_metadata() -> &'static ::ext_php_rs::php::types::object::ClassMetadata<Self> {
                &#meta
            }

            fn get_properties(&mut self) -> ::std::collections::HashMap<&'static str, &mut dyn ::ext_php_rs::php::types::object::Prop> {
                use ::std::iter::FromIterator;

                ::std::collections::HashMap::from_iter([
                    #(#prop_tuples)*
                ])
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
pub struct Property {
    field_name: String,
    prop_name: String,
    flags: Option<String>,
}

#[derive(Debug, Default)]
pub struct PropertyAttr {
    rename: Option<String>,
    flags: Option<Expr>,
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
        "prop" | "property" => {
            let attr = if attr.tokens.is_empty() {
                PropertyAttr::default()
            } else {
                attr.parse_args()
                    .map_err(|e| anyhow!("Unable to parse `#[{}]` attribute: {}", name, e))?
            };

            Some(ParsedAttribute::Property(attr))
        }
        _ => None,
    })
}
