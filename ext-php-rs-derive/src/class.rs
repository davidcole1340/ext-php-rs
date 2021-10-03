use std::collections::HashMap;

use crate::STATE;
use anyhow::{anyhow, bail, Result};
use darling::{FromMeta, ToTokens};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Attribute, AttributeArgs, Expr, Fields, FieldsNamed, ItemStruct, LitStr, Token};

#[derive(Debug, Default)]
pub struct Class {
    pub class_name: String,
    pub struct_path: String,
    pub parent: Option<String>,
    pub interfaces: Vec<String>,
    pub methods: Vec<crate::method::Method>,
    pub constructor: Option<crate::method::Method>,
    pub constants: Vec<crate::constant::Constant>,
    pub properties: HashMap<String, Property>,
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
    let mut properties = HashMap::new();

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
                            properties.insert(
                                prop_name,
                                Property::field(
                                    field_name,
                                    prop.flags.map(|flags| flags.to_token_stream().to_string()),
                                ),
                            );
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
    let struct_path = ident.to_string();
    let class = Class {
        class_name,
        struct_path,
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

    Ok(quote! { #input })
}

#[derive(Debug)]
pub struct Property {
    ty: PropertyType,
    #[allow(dead_code)]
    flags: Option<String>,
}

#[derive(Debug)]
pub enum PropertyType {
    Field {
        field_name: String,
    },
    Method {
        getter: Option<String>,
        setter: Option<String>,
    },
}

impl Property {
    pub fn add_getter(&mut self, new_getter: String) -> Result<()> {
        match &mut self.ty {
            PropertyType::Field { .. } => bail!("Cannot add getter to field property."),
            PropertyType::Method { getter, setter: _ } => match getter {
                Some(getter) => bail!(
                    "Attempted to add getter `{}` to property that already has a getter `{}`.",
                    new_getter,
                    getter
                ),
                None => {
                    getter.replace(new_getter);
                    Ok(())
                }
            },
        }
    }

    pub fn add_setter(&mut self, new_setter: String) -> Result<()> {
        match &mut self.ty {
            PropertyType::Field { .. } => bail!("Cannot add setter to field property."),
            PropertyType::Method { getter: _, setter } => match setter {
                Some(getter) => bail!(
                    "Attempted to add setter `{}` to property that already has a setter `{}`.",
                    new_setter,
                    getter
                ),
                None => {
                    setter.replace(new_setter);
                    Ok(())
                }
            },
        }
    }

    pub fn field(field_name: String, flags: Option<String>) -> Self {
        Self {
            ty: PropertyType::Field { field_name },
            flags,
        }
    }

    pub fn method(flags: Option<String>) -> Self {
        Self {
            ty: PropertyType::Method {
                getter: None,
                setter: None,
            },
            flags,
        }
    }

    pub fn as_prop_tuple(&self, name: &str) -> TokenStream {
        match &self.ty {
            PropertyType::Field { field_name } => {
                let field_name = Ident::new(field_name, Span::call_site());
                quote! {
                    (#name, ::ext_php_rs::php::types::props::Property::field(|obj: &mut Self| &mut obj.#field_name)),
                }
            }
            PropertyType::Method { getter, setter } => {
                let getter = if let Some(getter) = getter {
                    let ident = Ident::new(getter, Span::call_site());
                    quote! { Some(Self::#ident) }
                } else {
                    quote! { None }
                };
                let setter = if let Some(setter) = setter {
                    let ident = Ident::new(setter, Span::call_site());
                    quote! { Some(Self::#ident) }
                } else {
                    quote! { None }
                };
                quote! {
                    (#name, ::ext_php_rs::php::types::props::Property::method(#getter, #setter)),
                }
            }
        }
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
