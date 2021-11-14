use std::path::PathBuf;

use crate::describe::Module;
use anyhow::{Context, Result};
use libloading::os::unix::{Library, Symbol};

pub struct Ext {
    // These need to be here to keep the libraries alive. The extension library needs to be alive
    // to access the describe function. Missing here is the lifetime on `Symbol<'a, fn() ->
    // Module>` where `ext_lib: 'a`.
    #[allow(dead_code)]
    ext_lib: Library,
    describe_fn: Symbol<fn() -> Module>,
}

impl Ext {
    /// Loads an extension.
    pub fn load(ext_path: PathBuf) -> Result<Self> {
        let ext_lib = unsafe { Library::new(ext_path) }
            .with_context(|| "Failed to load extension library")?;

        let describe_fn = unsafe {
            ext_lib
                .get(b"ext_php_rs_describe_module")
                .with_context(|| "Failed to load describe function symbol from extension library")?
        };

        Ok(Self {
            ext_lib,
            describe_fn,
        })
    }

    /// Describes the extension.
    pub fn describe(&self) -> Module {
        (self.describe_fn)()
    }
}
