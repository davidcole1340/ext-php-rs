#[cfg_attr(windows, path = "windows_build.rs")]
#[cfg_attr(not(windows), path = "unix_build.rs")]
mod impl_;

use std::{
    env,
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use anyhow::{anyhow, bail, Context, Result};
use bindgen::RustTarget;
use impl_::Provider;

const MIN_PHP_API_VER: u32 = 20200930;
const MAX_PHP_API_VER: u32 = 20230831;

pub trait PHPProvider<'a>: Sized {
    /// Create a new PHP provider.
    fn new(info: &'a PHPInfo) -> Result<Self>;

    /// Retrieve a list of absolute include paths.
    fn get_includes(&self) -> Result<Vec<PathBuf>>;

    /// Retrieve a list of macro definitions to pass to the compiler.
    fn get_defines(&self) -> Result<Vec<(&'static str, &'static str)>>;

    /// Writes the bindings to a file.
    fn write_bindings(&self, bindings: String, writer: &mut impl Write) -> Result<()> {
        for line in bindings.lines() {
            writeln!(writer, "{line}")?;
        }
        Ok(())
    }

    /// Prints any extra link arguments.
    fn print_extra_link_args(&self) -> Result<()> {
        Ok(())
    }
}

/// Finds the location of an executable `name`.
pub fn find_executable(name: &str) -> Option<PathBuf> {
    const WHICH: &str = if cfg!(windows) { "where" } else { "which" };
    let cmd = Command::new(WHICH).arg(name).output().ok()?;
    if cmd.status.success() {
        let stdout = String::from_utf8_lossy(&cmd.stdout);
        Some(stdout.trim().into())
    } else {
        None
    }
}

/// Returns an environment variable's value as a PathBuf
pub fn path_from_env(key: &str) -> Option<PathBuf> {
    std::env::var_os(key).map(PathBuf::from)
}

/// Finds the location of the PHP executable.
fn find_php() -> Result<PathBuf> {
    // If path is given via env, it takes priority.
    if let Some(path) = path_from_env("PHP") {
        if !path.try_exists()? {
            // If path was explicitly given and it can't be found, this is a hard error
            bail!("php executable not found at {:?}", path);
        }
        return Ok(path);
    }
    find_executable("php").with_context(|| {
        "Could not find PHP executable. \
        Please ensure `php` is in your PATH or the `PHP` environment variable is set."
    })
}

pub struct PHPInfo(String);

impl PHPInfo {
    pub fn get(php: &Path) -> Result<Self> {
        let cmd = Command::new(php)
            .arg("-i")
            .output()
            .context("Failed to call `php -i`")?;
        if !cmd.status.success() {
            bail!("Failed to call `php -i` status code {}", cmd.status);
        }
        let stdout = String::from_utf8_lossy(&cmd.stdout);
        Ok(Self(stdout.to_string()))
    }

    // Only present on Windows.
    #[cfg(windows)]
    pub fn architecture(&self) -> Result<impl_::Arch> {
        use std::convert::TryInto;

        self.get_key("Architecture")
            .context("Could not find architecture of PHP")?
            .try_into()
    }

    pub fn thread_safety(&self) -> Result<bool> {
        Ok(self
            .get_key("Thread Safety")
            .context("Could not find thread safety of PHP")?
            == "enabled")
    }

    pub fn debug(&self) -> Result<bool> {
        Ok(self
            .get_key("Debug Build")
            .context("Could not find debug build of PHP")?
            == "yes")
    }

    pub fn version(&self) -> Result<&str> {
        self.get_key("PHP Version")
            .context("Failed to get PHP version")
    }

    pub fn zend_version(&self) -> Result<u32> {
        self.get_key("PHP API")
            .context("Failed to get Zend version")
            .and_then(|s| u32::from_str(s).context("Failed to convert Zend version to integer"))
    }

    fn get_key(&self, key: &str) -> Option<&str> {
        let split = format!("{key} => ");
        for line in self.0.lines() {
            let components: Vec<_> = line.split(&split).collect();
            if components.len() > 1 {
                return Some(components[1]);
            }
        }
        None
    }
}

/// Builds the wrapper library.
fn build_wrapper(defines: &[(&str, &str)], includes: &[PathBuf]) -> Result<()> {
    let mut build = cc::Build::new();
    for (var, val) in defines {
        build.define(var, *val);
    }
    build
        .file("src/wrapper.c")
        .includes(includes)
        .try_compile("wrapper")
        .context("Failed to compile ext-php-rs C interface")?;
    Ok(())
}

#[cfg(feature = "embed")]
/// Builds the embed library.
fn build_embed(defines: &[(&str, &str)], includes: &[PathBuf]) -> Result<()> {
    let mut build = cc::Build::new();
    for (var, val) in defines {
        build.define(var, *val);
    }
    build
        .file("src/embed/embed.c")
        .includes(includes)
        .try_compile("embed")
        .context("Failed to compile ext-php-rs C embed interface")?;
    Ok(())
}

/// Generates bindings to the Zend API.
fn generate_bindings(defines: &[(&str, &str)], includes: &[PathBuf]) -> Result<String> {
    let mut bindgen = bindgen::Builder::default();

    #[cfg(feature = "embed")]
    {
        bindgen = bindgen.header("src/embed/embed.h");
    }

    bindgen = bindgen
        .header("src/wrapper.h")
        .clang_args(
            includes
                .iter()
                .map(|inc| format!("-I{}", inc.to_string_lossy())),
        )
        .clang_args(defines.iter().map(|(var, val)| format!("-D{var}={val}")))
        .formatter(bindgen::Formatter::Rustfmt)
        .no_copy("_zval_struct")
        .no_copy("_zend_string")
        .no_copy("_zend_array")
        .no_debug("_zend_function_entry") // On Windows when the handler uses vectorcall, Debug cannot be derived so we do it in code.
        .layout_tests(env::var("EXT_PHP_RS_TEST").is_ok())
        .rust_target(RustTarget::Nightly);

    for binding in ALLOWED_BINDINGS.iter() {
        bindgen = bindgen
            .allowlist_function(binding)
            .allowlist_type(binding)
            .allowlist_var(binding);
    }

    let bindings = bindgen
        .generate()
        .map_err(|_| anyhow!("Unable to generate bindings for PHP"))?
        .to_string();

    Ok(bindings)
}

/// Checks the PHP Zend API version for compatibility with ext-php-rs, setting
/// any configuration flags required.
fn check_php_version(info: &PHPInfo) -> Result<()> {
    let version = info.zend_version()?;

    if !(MIN_PHP_API_VER..=MAX_PHP_API_VER).contains(&version) {
        bail!("The current version of PHP is not supported. Current PHP API version: {}, requires a version between {} and {}", version, MIN_PHP_API_VER, MAX_PHP_API_VER);
    }

    // Infra cfg flags - use these for things that change in the Zend API that don't
    // rely on a feature and the crate user won't care about (e.g. struct field
    // changes). Use a feature flag for an actual feature (e.g. enums being
    // introduced in PHP 8.1).
    //
    // PHP 8.0 is the baseline - no feature flags will be introduced here.
    //
    // The PHP version cfg flags should also stack - if you compile on PHP 8.2 you
    // should get both the `php81` and `php82` flags.
    const PHP_81_API_VER: u32 = 20210902;

    const PHP_82_API_VER: u32 = 20220829;

    const PHP_83_API_VER: u32 = 20230831;

    println!("cargo::rustc-check-cfg=cfg(php80, php81, php82, php83, php_zts, php_debug, docs)");
    println!("cargo:rustc-cfg=php80");

    if (PHP_81_API_VER..PHP_82_API_VER).contains(&version) {
        println!("cargo:rustc-cfg=php81");
    }

    if version >= PHP_82_API_VER {
        println!("cargo:rustc-cfg=php82");
    }

    if version >= PHP_83_API_VER {
        println!("cargo:rustc-cfg=php83");
    }

    Ok(())
}

fn main() -> Result<()> {
    let out_dir = env::var_os("OUT_DIR").context("Failed to get OUT_DIR")?;
    let out_path = PathBuf::from(out_dir).join("bindings.rs");
    let manifest: PathBuf = std::env::var("CARGO_MANIFEST_DIR").unwrap().into();
    for path in [
        manifest.join("src").join("wrapper.h"),
        manifest.join("src").join("wrapper.c"),
        manifest.join("src").join("embed").join("embed.h"),
        manifest.join("src").join("embed").join("embed.c"),
        manifest.join("allowed_bindings.rs"),
        manifest.join("windows_build.rs"),
        manifest.join("unix_build.rs"),
    ] {
        println!("cargo:rerun-if-changed={}", path.to_string_lossy());
    }
    for env_var in ["PHP", "PHP_CONFIG", "PATH"] {
        println!("cargo:rerun-if-env-changed={env_var}");
    }

    println!("cargo:rerun-if-changed=build.rs");

    // docs.rs runners only have PHP 7.4 - use pre-generated bindings
    if env::var("DOCS_RS").is_ok() {
        println!("cargo:warning=docs.rs detected - using stub bindings");
        println!("cargo:rustc-cfg=php_debug");
        println!("cargo:rustc-cfg=php82");
        std::fs::copy("docsrs_bindings.rs", out_path)
            .expect("failed to copy docs.rs stub bindings to out directory");
        return Ok(());
    }

    let php = find_php()?;
    let info = PHPInfo::get(&php)?;
    let provider = Provider::new(&info)?;

    let includes = provider.get_includes()?;
    let defines = provider.get_defines()?;

    check_php_version(&info)?;
    build_wrapper(&defines, &includes)?;

    #[cfg(feature = "embed")]
    build_embed(&defines, &includes)?;

    let bindings = generate_bindings(&defines, &includes)?;

    let out_file =
        File::create(&out_path).context("Failed to open output bindings file for writing")?;
    let mut out_writer = BufWriter::new(out_file);
    provider.write_bindings(bindings, &mut out_writer)?;

    if info.debug()? {
        println!("cargo:rustc-cfg=php_debug");
    }
    if info.thread_safety()? {
        println!("cargo:rustc-cfg=php_zts");
    }
    provider.print_extra_link_args()?;

    // Generate guide tests
    let test_md = skeptic::markdown_files_of_directory("guide");
    #[cfg(not(feature = "closure"))]
    let test_md: Vec<_> = test_md
        .into_iter()
        .filter(|p| p.file_stem() != Some(std::ffi::OsStr::new("closure")))
        .collect();
    skeptic::generate_doc_tests(&test_md);

    Ok(())
}

// Mock macro for the `allowed_bindings.rs` script.
macro_rules! bind {
    ($($s: ident),*) => {
        &[$(
            stringify!($s),
        )*]
    }
}

/// Array of functions/types used in `ext-php-rs` - used to allowlist when
/// generating bindings, as we don't want to generate bindings for everything
/// (i.e. stdlib headers).
const ALLOWED_BINDINGS: &[&str] = include!("allowed_bindings.rs");
