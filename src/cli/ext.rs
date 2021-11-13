use std::path::PathBuf;

use crate::describe::Module;
use anyhow::{anyhow, Result};
use dlopen::wrapper::{Container, WrapperApi};
use dlopen_derive::WrapperApi;

/// Represents an dynamically loaded extension.
#[derive(Debug, WrapperApi)]
pub struct Ext {
    /// Describe function, called to return the structure of the extension.
    ext_php_rs_describe_module: fn() -> Module,
}

impl Ext {
    /// Loads an `ext-php-rs` extension, including the
    /// `ext_php_rs_describe_module` symbol.
    ///
    /// # Parameters
    ///
    /// * `path` - Path to extension.
    pub fn load(path: PathBuf) -> Result<Container<Self>> {
        unsafe { Container::load(path) }
            .map_err(|e| match e {
                dlopen::Error::OpeningLibraryError(e) => anyhow!("Failed to open extension dynamic library: {}", e),
                dlopen::Error::SymbolGettingError(_) => anyhow!("Given extension is missing describe function. Only extensions utilizing ext-php-rs can be used with this application."),
                e => anyhow!("Unknown error: {}", e)
            })
    }
}
