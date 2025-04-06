use crate::class::{parse_attribute, ParsedAttribute};
use darling::FromMeta;
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

#[derive(Debug, Copy, Clone, FromMeta, Default)]
pub enum RenameRule {
    /// Methods won't be renamed.
    #[darling(rename = "none")]
    None,
    /// Methods will be converted to camelCase.
    #[darling(rename = "camelCase")]
    #[default]
    Camel,
    /// Methods will be converted to snake_case.
    #[darling(rename = "snake_case")]
    Snake,
}

pub trait Rename {
    fn renmae(self, rule: &RenameRule) -> Self;
}

impl Rename for String {
    fn renmae(self, rule: &RenameRule) -> Self {
        match *rule {
            RenameRule::None => self,
            RenameRule::Camel => ident_case::RenameRule::CamelCase.apply_to_field(self),
            RenameRule::Snake => ident_case::RenameRule::SnakeCase.apply_to_field(self),
        }
    }
}
