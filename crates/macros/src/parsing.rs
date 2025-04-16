use darling::FromMeta;

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

pub trait Rename {
    fn rename(&self, rule: &RenameRule) -> String;
}

pub trait MethodRename: Rename {
    fn rename_method(&self, rule: &RenameRule) -> String;
}

#[derive(FromMeta, Debug, Default)]
#[darling(default)]
pub struct PhpRename {
    name: Option<String>,
    rename: Option<RenameRule>,
}

impl PhpRename {
    pub fn rename(&self, name: impl AsRef<str>) -> String {
        self.name.as_ref().map_or_else(
            || {
                let name = name.as_ref();
                self.rename
                    .as_ref()
                    .map_or_else(|| name.to_string(), |r| name.rename(r))
            },
            ToString::to_string,
        )
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
    ScreamingSnakeCase,
}

impl RenameRule {
    fn rename(&self, value: impl AsRef<str>) -> String {
        match *self {
            Self::None => value.as_ref().to_string(),
            Self::Camel => ident_case::RenameRule::CamelCase.apply_to_field(value.as_ref()),
            Self::Pascal => ident_case::RenameRule::PascalCase.apply_to_field(value.as_ref()),
            Self::Snake => ident_case::RenameRule::SnakeCase.apply_to_field(value.as_ref()),
            Self::ScreamingSnakeCase => {
                ident_case::RenameRule::ScreamingSnakeCase.apply_to_field(value.as_ref())
            }
        }
    }
}

impl Rename for &str {
    fn rename(&self, rule: &RenameRule) -> String {
        rule.rename(self)
    }
}

impl Rename for syn::Ident {
    fn rename(&self, rule: &RenameRule) -> String {
        let s = self.to_string();
        rule.rename(s)
    }
}

impl MethodRename for syn::Ident {
    fn rename_method(&self, rule: &RenameRule) -> String {
        self.to_string().as_str().rename_method(rule)
    }
}

impl MethodRename for &str {
    fn rename_method(&self, rule: &RenameRule) -> String {
        match rule {
            RenameRule::None => self.to_string(),
            _ => {
                if MAGIC_METHOD.contains(self) {
                    match *self {
                        "__to_string" => "__toString".to_string(),
                        "__debug_info" => "__debugInfo".to_string(),
                        "__call_static" => "__callStatic".to_string(),
                        _ => self.to_string(),
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

    use super::{PhpRename, RenameRule, MAGIC_METHOD};

    #[test]
    fn php_rename() {
        let rename = PhpRename {
            name: Some("test".to_string()),
            rename: None,
        };
        assert_eq!(rename.rename("test"), "test");
        assert_eq!(rename.rename("Test"), "test");
        assert_eq!(rename.rename("TEST"), "test");

        let rename = PhpRename {
            name: None,
            rename: Some(RenameRule::ScreamingSnakeCase),
        };
        assert_eq!(rename.rename("test"), "TEST");
        assert_eq!(rename.rename("Test"), "TEST");
        assert_eq!(rename.rename("TEST"), "TEST");

        let rename = PhpRename {
            name: Some("test".to_string()),
            rename: Some(RenameRule::ScreamingSnakeCase),
        };
        assert_eq!(rename.rename("test"), "test");
        assert_eq!(rename.rename("Test"), "test");
        assert_eq!(rename.rename("TEST"), "test");

        let rename = PhpRename {
            name: None,
            rename: None,
        };
        assert_eq!(rename.rename("test"), "test");
        assert_eq!(rename.rename("Test"), "Test");
        assert_eq!(rename.rename("TEST"), "TEST");
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
            assert_eq!(magic, magic.rename_method(&RenameRule::None));
            assert_eq!(expected, magic.rename_method(&RenameRule::Camel));
            assert_eq!(expected, magic.rename_method(&RenameRule::Pascal));
            assert_eq!(expected, magic.rename_method(&RenameRule::Snake));
            assert_eq!(
                expected,
                magic.rename_method(&RenameRule::ScreamingSnakeCase)
            );
        }
    }

    #[test]
    fn rename_method() {
        let &(original, camel, snake, pascal, screaming_snake) =
            &("get_name", "getName", "get_name", "GetName", "GET_NAME");
        assert_eq!(original, original.rename_method(&RenameRule::None));
        assert_eq!(camel, original.rename_method(&RenameRule::Camel));
        assert_eq!(pascal, original.rename_method(&RenameRule::Pascal));
        assert_eq!(snake, original.rename_method(&RenameRule::Snake));
        assert_eq!(
            screaming_snake,
            original.rename_method(&RenameRule::ScreamingSnakeCase)
        );
    }

    #[test]
    fn rename() {
        let &(original, camel, snake, pascal, screaming_snake) =
            &("get_name", "getName", "get_name", "GetName", "GET_NAME");
        assert_eq!(original, original.rename(&RenameRule::None));
        assert_eq!(camel, original.rename(&RenameRule::Camel));
        assert_eq!(pascal, original.rename(&RenameRule::Pascal));
        assert_eq!(snake, original.rename(&RenameRule::Snake));
        assert_eq!(
            screaming_snake,
            original.rename(&RenameRule::ScreamingSnakeCase)
        );
    }
}
