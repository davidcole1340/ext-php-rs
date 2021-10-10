use anyhow::{anyhow, bail, Result};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, ForeignItemFn, ItemForeignMod, ReturnType, Signature, Token};

pub fn parser(input: ItemForeignMod) -> Result<TokenStream> {
    input
        .items
        .into_iter()
        .map(|item| match item {
            syn::ForeignItem::Fn(func) => parse_function(func),
            _ => bail!("Only `extern` functions are supported by PHP."),
        })
        .collect::<Result<Vec<_>>>()
        .map(|vec| quote! { #(#vec)* })
}

fn parse_function(mut func: ForeignItemFn) -> Result<TokenStream> {
    let ForeignItemFn {
        attrs, vis, sig, ..
    } = &mut func;
    sig.unsafety = Some(Default::default()); // Function must be unsafe.

    let Signature { ident, .. } = &sig;

    let name = ident.to_string();
    let params = sig
        .inputs
        .iter()
        .map(|input| match input {
            syn::FnArg::Typed(arg) => {
                let pat = &arg.pat;
                Some(quote! { &#pat })
            }
            _ => None,
        })
        .collect::<Option<Punctuated<_, Token![,]>>>()
        .ok_or_else(|| {
            anyhow!("`self` parameters are not permitted inside `#[php_extern]` blocks.")
        })?;
    let ret = build_return(&name, &sig.output, params);

    Ok(quote! {
        #(#attrs)* #vis #sig {
            use ::std::convert::TryInto;

            let callable = ::ext_php_rs::types::ZendCallable::try_from_name(
                #name
            ).expect(concat!("Unable to find callable function `", #name, "`."));

            #ret
        }
    })
}

fn build_return(
    name: &str,
    return_type: &ReturnType,
    params: Punctuated<TokenStream, Token![,]>,
) -> TokenStream {
    match return_type {
        ReturnType::Default => quote! {
            callable.try_call(vec![ #params ]);
        },
        ReturnType::Type(_, _) => quote! {
            callable
                .try_call(vec![ #params ])
                .ok()
                .and_then(|zv| zv.try_into().ok())
                .expect(concat!("Failed to call function `", #name, "`."))
        },
    }
}
