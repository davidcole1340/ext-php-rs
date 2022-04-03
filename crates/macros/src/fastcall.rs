use anyhow::Result;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{ItemFn, LitStr};

#[cfg(windows)]
const ABI: &str = "vectorcall";
#[cfg(not(windows))]
const ABI: &str = "C";

pub fn parser(mut input: ItemFn) -> Result<TokenStream> {
    if let Some(abi) = &mut input.sig.abi {
        abi.name = Some(LitStr::new(ABI, Span::call_site()));
    }
    Ok(input.to_token_stream())
}
