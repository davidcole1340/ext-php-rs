use crate::prelude::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::TraitItem;

pub fn parser(args: TokenStream, input: TraitItem) -> Result<TokenStream> {
    Ok(quote! {()})
}
