use std::{borrow::Cow, ops::Deref, path::PathBuf, str::FromStr};

use dlopen::wrapper::{Container, WrapperApi};
use dlopen_derive::WrapperApi;
use ext_php_rs_describe::Module;

/// Wrapper around library container, implements [`FromStr`] to be compatible
/// with clap.
pub struct Extension(pub Container<ExtInner>);

impl Deref for Extension {
    type Target = Container<ExtInner>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Extension {
    type Err = Cow<'static, str>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path = PathBuf::from(s);

        if !path.is_file() {
            return Err(format!("Given extension path `{}` is not a file.", s).into());
        }

        let container: Container<ExtInner> = match unsafe { Container::load(s) } {
            Ok(container) => container,
            Err(e) => {
                return Err(match e {
                    dlopen::Error::NullCharacter(_) => {
                        "Given extension path contained null characters.".into()
                    }
                    dlopen::Error::OpeningLibraryError(e) => {
                        format!("Failed to open extension library: {}", e).into()
                    }
                    dlopen::Error::SymbolGettingError(_) => {
                        "Given extension is missing describe function. Only extensions utilizing ext-php-rs can be used with this application.".into()
                    }
                    e => format!("Unknown error: {}", e).into(),
                })
            }
        };

        Ok(Self(container))
    }
}

/// Represents an dynamically loaded extension.
#[derive(Debug, WrapperApi)]
pub struct ExtInner {
    /// Describe function, called to return the structure of the extension.
    ext_php_rs_describe_module: fn() -> Module,
}
