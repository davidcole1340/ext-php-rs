use std::str::FromStr;

use darling::{FromMeta, ToTokens};
use proc_macro2::{Ident, Literal, TokenStream};
use quote::quote;
use syn::ItemConst;

use crate::{error::Result, STATE};

#[derive(Debug)]
pub struct Constant {
    pub name: String,
    // pub visibility: Visibility,
    pub value: String,
}

pub fn parser(input: ItemConst) -> Result<TokenStream> {
    let mut state = STATE.lock()?;

    state.constants.push(Constant {
        name: input.ident.to_string(),
        value: input.expr.to_token_stream().to_string(),
    });

    Ok(quote! {
        #[allow(dead_code)]
        #input
    })
}

impl Constant {
    pub fn val_tokens(&self) -> TokenStream {
        Literal::from_str(&self.value)
            .map(|lit| lit.to_token_stream())
            .or_else(|_| Ident::from_string(&self.value).map(|ident| ident.to_token_stream()))
            .unwrap_or(quote! { Default::default() })
    }

    // pub fn get_flags(&self) -> TokenStream {
    //     let flag = match self.visibility {
    //         Visibility::Public => quote! { Public },
    //         Visibility::Protected => quote! { Protected },
    //         Visibility::Private => quote! { Private },
    //     };

    //     quote! { ::ext_php_rs::php::flags::ConstantFlags}
    // }
}
