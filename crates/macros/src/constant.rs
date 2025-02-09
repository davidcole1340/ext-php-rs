use crate::helpers::get_docs;
use anyhow::Result;
use darling::ToTokens;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, ItemConst};

#[derive(Debug)]
pub struct Constant {
    pub name: String,
    // pub visibility: Visibility,
    pub docs: Vec<String>,
    pub value: String,
}

pub fn parser(input: &mut ItemConst) -> Result<(TokenStream, Constant)> {
    let constant = Constant {
        name: input.ident.to_string(),
        docs: get_docs(&input.attrs),
        value: input.expr.to_token_stream().to_string(),
    };

    input.attrs.remove(0);

    Ok((
        quote! {
            #[allow(dead_code)]
            #input
        },
        constant,
    ))
}

impl Constant {
    pub fn val_tokens(&self) -> TokenStream {
        let expr: Expr =
            syn::parse_str(&self.value).expect("failed to parse previously parsed expr");
        expr.to_token_stream()
    }

    // pub fn get_flags(&self) -> TokenStream {
    //     let flag = match self.visibility {
    //         Visibility::Public => quote! { Public },
    //         Visibility::Protected => quote! { Protected },
    //         Visibility::Private => quote! { Private },
    //     };

    //     quote! { ::ext_php_rs::flags::ConstantFlags}
    // }
}
