use crate::STATE;
use anyhow::{anyhow, bail, Context, Result};
use darling::{FromMeta, ToTokens};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, format_ident};
use syn::parse::ParseStream;
use syn::{Attribute, AttributeArgs, Expr, LitStr, Token, ItemEnum};

#[derive(Debug, Default)]
pub struct Enum {
    pub enum_name: String,
    pub struct_path: String,
    pub docs: Vec<String>,
    pub methods: Vec<crate::method::Method>,
    pub modifier: Option<String>,
    pub cases: Vec<String>,
    pub flags: Option<String>,
}

#[derive(Debug)]
pub enum ParsedAttribute {
    Comment(String),
}

#[derive(Default, Debug, FromMeta)]
#[darling(default)]
pub struct AttrArgs {
    name: Option<String>,
    modifier: Option<String>,
    flags: Option<Expr>,
}

pub fn parser(args: AttributeArgs, mut input: ItemEnum) -> Result<TokenStream> {
    let args = AttrArgs::from_list(&args)
        .map_err(|e| anyhow!("Unable to parse attribute arguments: {:?}", e))?;

    let mut cases = vec![];
    let mut comments = vec![];

    input.attrs = {
        let mut unused = vec![];
        for attr in input.attrs.into_iter() {
            match parse_attribute(&attr)? {
                Some(parsed) => match parsed {
                    ParsedAttribute::Comment(comment) => {
                        comments.push(comment);
                    }
                    attr => bail!("Attribute `{:?}` is not valid for enums.", attr),
                },
                None => unused.push(attr),
            }
        }
        unused
    };

    for variant in input.variants.iter_mut() {
        let mut attrs = vec![];
        attrs.append(&mut variant.attrs);
        cases.push(variant.ident.to_string());
    }

    let ItemEnum { ident, .. } = &input;
    let enum_name = args.name.unwrap_or_else(|| input.ident.to_string());
    let struct_path = input.ident.to_string();
    let flags = args.flags.map(|flags| flags.to_token_stream().to_string());
    let enum_ = Enum {
        enum_name,
        struct_path,
        docs: comments,
        modifier: args.modifier,
        flags,
        cases,
        ..Default::default()
    };

    let mut state = STATE.lock();

    if state.startup_function.is_some() {
       // bail!("The `#[php_startup]` macro must be called after all the enums have been defined.");
    }

    let cases = enum_.cases.clone();
    let cases = cases.iter().map(|case| {
        let name = case;
        let ident = format_ident!("{}", name);
        quote! { #name => Ok(Self::#ident), }
    });


    state.enums.insert(input.ident.to_string(), enum_);

    let name = stringify!(#ident);
    Ok(quote! {
        #input

        impl ext_php_rs::convert::FromZendObject<'_> for #ident {
            fn from_zend_object(object: &ext_php_rs::types::ZendObject) -> Result<Self, ext_php_rs::error::Error> {
                let name = &object
                    .get_properties()?
                    .get("name")
                    .ok_or(ext_php_rs::error::Error::InvalidProperty)?
                    .indirect()
                    .ok_or(ext_php_rs::error::Error::InvalidProperty)?
                    .string()
                    .ok_or(ext_php_rs::error::Error::InvalidProperty)?;

                match name.as_str() {
                    #(#cases)*
                    _ => Err(ext_php_rs::error::Error::InvalidProperty),
                }
            }
        }

        impl ext_php_rs::convert::FromZval<'_> for #ident {
            const TYPE: ext_php_rs::flags::DataType = ext_php_rs::flags::DataType::Object(Some(#name));
            fn from_zval(zval: &ext_php_rs::types::Zval) -> Option<Self> {
                zval.object()
                    .and_then(|o| Self::from_zend_object(o).ok())
            }
        }

        impl ext_php_rs::convert::IntoZendObject for #ident {
            fn into_zend_object(self) -> ext_php_rs::error::Result<ext_php_rs::boxed::ZBox<ext_php_rs::types::ZendObject>> {
                let mut obj = ext_php_rs::types::ZendObject::new(#ident::get_metadata().ce());
                let name = ext_php_rs::types::ZendStr::new("name", false);
                let mut zval = ext_php_rs::types::Zval::new();
                zval.set_zend_string(name);
                obj.properties_table[0] = zval;
                Ok(obj)
            }
        }

        impl ext_php_rs::convert::IntoZval for #ident {
            const TYPE: ext_php_rs::flags::DataType = ext_php_rs::flags::DataType::Object(Some(#name));

            fn set_zval(self, zv: &mut ext_php_rs::types::Zval, _persistent: bool) -> ext_php_rs::error::Result<()> {
                let obj = self.into_zend_object()?;
                zv.set_object(obj.into_raw());
                Ok(())
            }
        }
    })
}

#[derive(Debug)]
pub struct Property {
    pub ty: PropertyType,
    pub docs: Vec<String>,
    #[allow(dead_code)]
    pub flags: Option<String>,
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

pub fn parse_attribute(attr: &Attribute) -> Result<Option<ParsedAttribute>> {
    let name = attr.path.to_token_stream().to_string();

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

            let comment: DocComment =
                syn::parse2(attr.tokens.clone()).with_context(|| "Failed to parse doc comment")?;
            Some(ParsedAttribute::Comment(comment.0))
        }
        _ => None,
    })
}
