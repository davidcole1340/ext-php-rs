use std::path::PathBuf;

use crate::describe::Module;
use anyhow::{Context, Result};
use libloading::os::unix::{Library, Symbol};

pub struct Ext {
    // These need to be here to keep the libraries alive. The PHP library needs to be alive to
    // retrieve the describe function, while the extension library needs to be alive to access the
    // describe function. Missing here is the lifetime on `Symbol<'a, fn() -> Module>` where
    // `ext_lib: 'a`.
    #[allow(dead_code)]
    php_lib: Library,
    #[allow(dead_code)]
    ext_lib: Library,
    describe_fn: Symbol<fn() -> Module>,
}

impl Ext {
    /// Loads an extension.
    pub fn load(ext_path: PathBuf, php_path: PathBuf) -> Result<Self> {
        let php_lib = dlopen_php(php_path)?;
        let ext_lib = unsafe { Library::new(ext_path) }
            .with_context(|| "Failed to load extension library")?;

        let describe_fn = unsafe {
            ext_lib
                .get(b"ext_php_rs_describe_module")
                .with_context(|| "Failed to load describe function symbol from extension library")?
        };

        Ok(Self {
            php_lib,
            ext_lib,
            describe_fn,
        })
    }

    /// Describes the extension.
    pub fn describe(&self) -> Module {
        (self.describe_fn)()
    }
}

/// Attempts to load the PHP executable with `dlopen`, making the required
/// symbols available to the next library loaded with `dlopen`.
fn dlopen_php(path: PathBuf) -> Result<Library> {
    unsafe {
        let php_lib = Library::open(
            Some(path),
            libloading::os::unix::RTLD_GLOBAL | libloading::os::unix::RTLD_LAZY,
        )
        .with_context(|| "Failed to open PHP executable")?;

        // Attempts to get a symbol from the PHP library. The given symbol `sym`
        // **MUST** be valid UTF-8.
        let get = |sym| {
            php_lib.get::<*mut ()>(sym).with_context(|| {
                format!(
                    "Failed to retrieve symbol `{}` from PHP",
                    std::str::from_utf8_unchecked(sym)
                )
            })
        };

        get(b"std_object_handlers")?;
        get(b"zend_ce_exception")?;
        get(b"zend_class_serialize_deny")?;
        get(b"zend_class_unserialize_deny")?;
        get(b"zend_string_init_interned")?;

        Ok(php_lib)
    }
}
