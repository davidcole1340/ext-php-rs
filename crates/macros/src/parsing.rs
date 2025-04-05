use darling::FromMeta;

#[derive(Debug, FromMeta)]
pub enum Visibility {
    #[darling(rename = "public")]
    Public,
    #[darling(rename = "private")]
    Private,
    #[darling(rename = "protected")]
    Protected,
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
                    .map_or_else(|| name.to_string(), |r| r.rename(name))
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
    /// Change case of an identifier.
    ///
    /// Magic methods are handled specially to make sure they're always cased
    /// correctly.
    pub fn rename(self, name: impl AsRef<str>) -> String {
        let name = name.as_ref();
        match self {
            RenameRule::None => name.to_string(),
            rule => match name {
                "__construct" => "__construct".to_string(),
                "__destruct" => "__destruct".to_string(),
                "__call" => "__call".to_string(),
                "__call_static" => "__callStatic".to_string(),
                "__get" => "__get".to_string(),
                "__set" => "__set".to_string(),
                "__isset" => "__isset".to_string(),
                "__unset" => "__unset".to_string(),
                "__sleep" => "__sleep".to_string(),
                "__wakeup" => "__wakeup".to_string(),
                "__serialize" => "__serialize".to_string(),
                "__unserialize" => "__unserialize".to_string(),
                "__to_string" => "__toString".to_string(),
                "__invoke" => "__invoke".to_string(),
                "__set_state" => "__set_state".to_string(),
                "__clone" => "__clone".to_string(),
                "__debug_info" => "__debugInfo".to_string(),
                field => match rule {
                    Self::Camel => ident_case::RenameRule::CamelCase.apply_to_field(field),
                    Self::Pascal => ident_case::RenameRule::PascalCase.apply_to_field(field),
                    Self::Snake => ident_case::RenameRule::SnakeCase.apply_to_field(field),
                    Self::ScreamingSnakeCase => {
                        ident_case::RenameRule::ScreamingSnakeCase.apply_to_field(field)
                    }
                    Self::None => unreachable!(),
                },
            },
        }
    }
}
