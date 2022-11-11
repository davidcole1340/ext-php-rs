use anyhow::Result;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{ItemFn, LitStr};

#[cfg(windows)]
const ABI: &str = "vectorcall";
#[cfg(not(windows))]
const ABI: &str = "C";

/// Parses a function and sets the correct ABI to interact with PHP depending
/// on the OS.
///
/// On Windows, this sets the extern ABI to vectorcall while on all other OS
/// it it to C.
pub fn parser(mut input: ItemFn) -> Result<TokenStream> {
    if let Some(abi) = &mut input.sig.abi {
        abi.name = Some(LitStr::new(ABI, Span::call_site()));
    }
    Ok(input.to_token_stream())
}
