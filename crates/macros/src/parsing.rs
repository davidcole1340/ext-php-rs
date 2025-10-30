use convert_case::{Case, Casing};
use darling::FromMeta;
use quote::{ToTokens, quote};

const MAGIC_METHOD: [&str; 17] = [
    "__construct",
    "__destruct",
    "__call",
    "__call_static",
    "__get",
    "__set",
    "__isset",
    "__unset",
    "__sleep",
    "__wakeup",
    "__serialize",
    "__unserialize",
    "__to_string",
    "__invoke",
    "__set_state",
    "__clone",
    "__debug_info",
];

#[derive(Debug, FromMeta)]
pub enum Visibility {
    #[darling(rename = "public")]
    Public,
    #[darling(rename = "private")]
    Private,
    #[darling(rename = "protected")]
    Protected,
}

impl ToTokens for Visibility {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Visibility::Public => quote! { ::ext_php_rs::flags::MethodFlags::Public },
            Visibility::Protected => quote! { ::ext_php_rs::flags::MethodFlags::Protected },
            Visibility::Private => quote! { ::ext_php_rs::flags::MethodFlags::Private },
        }
        .to_tokens(tokens);
    }
}

pub trait Rename {
    fn rename(&self, rule: RenameRule) -> String;
}

pub trait MethodRename: Rename {
    fn rename_method(&self, rule: RenameRule) -> String;
}

#[derive(FromMeta, Debug, Default)]
#[darling(default)]
pub struct PhpRename {
    name: Option<String>,
    change_case: Option<RenameRule>,
}

impl PhpRename {
    pub fn rename(&self, name: impl AsRef<str>, default: RenameRule) -> String {
        if let Some(name) = self.name.as_ref() {
            name.clone()
        } else {
            name.as_ref().rename(self.change_case.unwrap_or(default))
        }
    }

    pub fn rename_method(&self, name: impl AsRef<str>, default: RenameRule) -> String {
        if let Some(name) = self.name.as_ref() {
            name.clone()
        } else {
            name.as_ref()
                .rename_method(self.change_case.unwrap_or(default))
        }
    }
}

#[derive(Debug, Copy, Clone, FromMeta, Default)]
pub enum RenameRule {
    /// Methods won't be renamed.
    #[darling(rename = "none")]
    None,
    /// Methods will be converted to `camelCase`.
    #[darling(rename = "camelCase")]
    #[default]
    Camel,
    /// Methods will be converted to `snake_case`.
    #[darling(rename = "snake_case")]
    Snake,
    /// Methods will be converted to `PascalCase`.
    #[darling(rename = "PascalCase")]
    Pascal,
    /// Renames to `UPPER_SNAKE_CASE`.
    #[darling(rename = "UPPER_CASE")]
    ScreamingSnake,
}

impl RenameRule {
    fn rename(self, value: impl AsRef<str>) -> String {
        match self {
            Self::None => value.as_ref().to_string(),
            Self::Camel => value.as_ref().to_case(Case::Camel),
            Self::Pascal => value.as_ref().to_case(Case::Pascal),
            Self::Snake => value.as_ref().to_case(Case::Snake),
            Self::ScreamingSnake => value.as_ref().to_case(Case::Constant),
        }
    }
}

impl<T> Rename for T
where
    T: ToString,
{
    fn rename(&self, rule: RenameRule) -> String {
        rule.rename(self.to_string())
    }
}

impl<T> MethodRename for T
where
    T: ToString + Rename,
{
    fn rename_method(&self, rule: RenameRule) -> String {
        let original = self.to_string();
        match rule {
            RenameRule::None => original,
            _ => {
                if MAGIC_METHOD.contains(&original.as_str()) {
                    match original.as_str() {
                        "__to_string" => "__toString".to_string(),
                        "__debug_info" => "__debugInfo".to_string(),
                        "__call_static" => "__callStatic".to_string(),
                        _ => original,
                    }
                } else {
                    self.rename(rule)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parsing::{MethodRename, Rename};

    use super::{PhpRename, RenameRule};

    #[test]
    fn php_rename() {
        let rename = PhpRename {
            name: Some("test".to_string()),
            change_case: None,
        };
        assert_eq!(rename.rename("testCase", RenameRule::Snake), "test");
        assert_eq!(rename.rename("TestCase", RenameRule::Snake), "test");
        assert_eq!(rename.rename("TEST_CASE", RenameRule::Snake), "test");

        let rename = PhpRename {
            name: None,
            change_case: Some(RenameRule::ScreamingSnake),
        };
        assert_eq!(rename.rename("testCase", RenameRule::Snake), "TEST_CASE");
        assert_eq!(rename.rename("TestCase", RenameRule::Snake), "TEST_CASE");
        assert_eq!(rename.rename("TEST_CASE", RenameRule::Snake), "TEST_CASE");

        let rename = PhpRename {
            name: Some("test".to_string()),
            change_case: Some(RenameRule::ScreamingSnake),
        };
        assert_eq!(rename.rename("testCase", RenameRule::Snake), "test");
        assert_eq!(rename.rename("TestCase", RenameRule::Snake), "test");
        assert_eq!(rename.rename("TEST_CASE", RenameRule::Snake), "test");

        let rename = PhpRename {
            name: None,
            change_case: None,
        };
        assert_eq!(rename.rename("testCase", RenameRule::Snake), "test_case");
        assert_eq!(rename.rename("TestCase", RenameRule::Snake), "test_case");
        assert_eq!(rename.rename("TEST_CASE", RenameRule::Snake), "test_case");
    }

    #[test]
    fn php_rename_method() {
        let rename = PhpRename {
            name: Some("test".to_string()),
            change_case: None,
        };
        assert_eq!(rename.rename_method("testCase", RenameRule::Snake), "test");
        assert_eq!(rename.rename_method("TestCase", RenameRule::Snake), "test");
        assert_eq!(rename.rename_method("TEST_CASE", RenameRule::Snake), "test");

        let rename = PhpRename {
            name: None,
            change_case: Some(RenameRule::ScreamingSnake),
        };
        assert_eq!(
            rename.rename_method("testCase", RenameRule::Snake),
            "TEST_CASE"
        );
        assert_eq!(
            rename.rename_method("TestCase", RenameRule::Snake),
            "TEST_CASE"
        );
        assert_eq!(
            rename.rename_method("TEST_CASE", RenameRule::Snake),
            "TEST_CASE"
        );

        let rename = PhpRename {
            name: Some("test".to_string()),
            change_case: Some(RenameRule::ScreamingSnake),
        };
        assert_eq!(rename.rename_method("testCase", RenameRule::Snake), "test");
        assert_eq!(rename.rename_method("TestCase", RenameRule::Snake), "test");
        assert_eq!(rename.rename_method("TEST_CASE", RenameRule::Snake), "test");

        let rename = PhpRename {
            name: None,
            change_case: None,
        };
        assert_eq!(
            rename.rename_method("testCase", RenameRule::Snake),
            "test_case"
        );
        assert_eq!(
            rename.rename_method("TestCase", RenameRule::Snake),
            "test_case"
        );
        assert_eq!(
            rename.rename_method("TEST_CASE", RenameRule::Snake),
            "test_case"
        );
    }

    #[test]
    fn rename_magic_method() {
        for &(magic, expected) in &[
            ("__construct", "__construct"),
            ("__destruct", "__destruct"),
            ("__call", "__call"),
            ("__call_static", "__callStatic"),
            ("__get", "__get"),
            ("__set", "__set"),
            ("__isset", "__isset"),
            ("__unset", "__unset"),
            ("__sleep", "__sleep"),
            ("__wakeup", "__wakeup"),
            ("__serialize", "__serialize"),
            ("__unserialize", "__unserialize"),
            ("__to_string", "__toString"),
            ("__invoke", "__invoke"),
            ("__set_state", "__set_state"),
            ("__clone", "__clone"),
            ("__debug_info", "__debugInfo"),
        ] {
            assert_eq!(magic, magic.rename_method(RenameRule::None));
            assert_eq!(
                magic,
                PhpRename {
                    name: None,
                    change_case: Some(RenameRule::None)
                }
                .rename_method(magic, RenameRule::ScreamingSnake)
            );

            assert_eq!(expected, magic.rename_method(RenameRule::Camel));
            assert_eq!(
                expected,
                PhpRename {
                    name: None,
                    change_case: Some(RenameRule::Camel)
                }
                .rename_method(magic, RenameRule::ScreamingSnake)
            );

            assert_eq!(expected, magic.rename_method(RenameRule::Pascal));
            assert_eq!(
                expected,
                PhpRename {
                    name: None,
                    change_case: Some(RenameRule::Pascal)
                }
                .rename_method(magic, RenameRule::ScreamingSnake)
            );

            assert_eq!(expected, magic.rename_method(RenameRule::Snake));
            assert_eq!(
                expected,
                PhpRename {
                    name: None,
                    change_case: Some(RenameRule::Snake)
                }
                .rename_method(magic, RenameRule::ScreamingSnake)
            );

            assert_eq!(expected, magic.rename_method(RenameRule::ScreamingSnake));
            assert_eq!(
                expected,
                PhpRename {
                    name: None,
                    change_case: Some(RenameRule::ScreamingSnake)
                }
                .rename_method(magic, RenameRule::Camel)
            );
        }
    }

    #[test]
    fn rename_method() {
        let &(original, camel, snake, pascal, screaming_snake) =
            &("get_name", "getName", "get_name", "GetName", "GET_NAME");
        assert_eq!(original, original.rename_method(RenameRule::None));
        assert_eq!(camel, original.rename_method(RenameRule::Camel));
        assert_eq!(pascal, original.rename_method(RenameRule::Pascal));
        assert_eq!(snake, original.rename_method(RenameRule::Snake));
        assert_eq!(
            screaming_snake,
            original.rename_method(RenameRule::ScreamingSnake)
        );
    }

    #[test]
    fn rename() {
        let &(original, camel, snake, pascal, screaming_snake) =
            &("get_name", "getName", "get_name", "GetName", "GET_NAME");
        assert_eq!(original, original.rename(RenameRule::None));
        assert_eq!(camel, original.rename(RenameRule::Camel));
        assert_eq!(pascal, original.rename(RenameRule::Pascal));
        assert_eq!(snake, original.rename(RenameRule::Snake));
        assert_eq!(screaming_snake, original.rename(RenameRule::ScreamingSnake));
    }
}
