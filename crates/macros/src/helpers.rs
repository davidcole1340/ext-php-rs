use crate::class::{parse_attribute, ParsedAttribute};
use syn::Attribute;

pub type Docs = Vec<String>;

/// Takes a list of attributes and returns a list of doc comments retrieved from
/// the attributes.
pub fn get_docs(attrs: &[Attribute]) -> Vec<String> {
    let mut docs = vec![];

    for attr in attrs {
        if let Ok(Some(ParsedAttribute::Comment(doc))) = parse_attribute(attr) {
            docs.push(doc);
        }
    }

    docs
}

pub trait GetDocs {
    fn get_docs(&self) -> Docs;
}

impl GetDocs for &[Attribute] {
    fn get_docs(&self) -> Docs {
        get_docs(self)
    }
}
