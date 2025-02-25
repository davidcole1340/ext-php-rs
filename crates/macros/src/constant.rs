use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::ItemConst;

use crate::helpers::get_docs;
use crate::prelude::*;

const INTERNAL_CONST_DOC_PREFIX: &str = "_internal_const_docs_";

pub fn parser(item: ItemConst) -> TokenStream {
    let docs = get_docs(&item.attrs);
    let docs_ident = format_ident!("{INTERNAL_CONST_DOC_PREFIX}{}", item.ident);

    quote! {
        #item
        #[allow(non_upper_case_globals)]
        const #docs_ident: &[&str] = &[#(#docs),*];
    }
}

pub fn wrap(input: syn::Path) -> Result<TokenStream> {
    let Some(const_name) = input.get_ident().map(|i| i.to_string()) else {
        bail!(input => "Pass a PHP const into `wrap_constant!()`.");
    };
    let doc_const = format_ident!("{INTERNAL_CONST_DOC_PREFIX}{const_name}");

    Ok(quote! {
        (#const_name, #input, #doc_const)

    })
}
