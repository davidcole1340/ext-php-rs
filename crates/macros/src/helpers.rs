use crate::prelude::*;
use syn::{Attribute, Expr, Lit, Meta};

/// Takes a list of attributes and returns a list of doc comments retrieved from
/// the attributes.
pub fn get_docs(attrs: &[Attribute]) -> Result<Vec<String>> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .map(|attr| {
            let Meta::NameValue(meta) = &attr.meta else {
                bail!(attr.meta => "Invalid format for `#[doc]` attribute.");
            };

            let Expr::Lit(value) = &meta.value else {
                bail!(attr.meta => "Invalid format for `#[doc]` attribute.");
            };

            let Lit::Str(doc) = &value.lit else {
                bail!(value.lit => "Invalid format for `#[doc]` attribute.");
            };

            Ok(doc.value())
        })
        .collect::<Result<Vec<_>>>()
}
