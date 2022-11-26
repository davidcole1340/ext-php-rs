#[cfg_attr(windows, path = "windows_build.rs")]
#[cfg_attr(not(windows), path = "unix_build.rs")]
mod impl_;

use std::{
    env,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use anyhow::{anyhow, bail, Context, Result};
use bindgen::RustTarget;
use impl_::Provider;
use php_discovery::build::Build as PhpBuild;

const MIN_PHP_API_VER: u32 = 20200930;
const MAX_PHP_API_VER: u32 = 20210902;

pub trait PHPProvider<'a>: Sized {
    /// Create a new PHP provider.
    fn new(info: &'a PhpBuild) -> Result<Self>;

    /// Retrieve a list of absolute include paths.
    fn get_includes(&self) -> Result<Vec<PathBuf>>;

    /// Retrieve a list of macro definitions to pass to the compiler.
    fn get_defines(&self) -> Result<Vec<(&'static str, &'static str)>>;

    /// Writes the bindings to a file.
    fn write_bindings(&self, bindings: String, writer: &mut impl Write) -> Result<()> {
        for line in bindings.lines() {
            writeln!(writer, "{}", line)?;
        }

        Ok(())
    }

    /// Prints any extra link arguments.
    fn print_extra_link_args(&self) -> Result<()> {
        Ok(())
    }
}

/// Finds the location of the PHP executable.
fn find_php() -> Result<PhpBuild> {
    php_discovery::discover()
        .map_err(|e| anyhow!("failed to discover available PHP builds: {:?}", e))
        .and_then(|builds| {
            if builds.is_empty() {
                bail!("Could not find any PHP builds in the system, please ensure that PHP is installed.")
            }

            Ok(builds)
        })
        .and_then(|builds| {
            let mut available = Vec::new();
            let mut matching = Vec::new();

            for build in builds {
                available.push(build.php_api.to_string());
                if build.php_api >= MIN_PHP_API_VER && build.php_api <= MAX_PHP_API_VER {
                    matching.push(build);
                }
            }

            if matching.is_empty() {
                bail!(
                    "Unable to find matching PHP binary, available PHP API version(s): '{}', requires a version between {} and {}", 
                    available.join(", "),
                    MIN_PHP_API_VER,
                    MAX_PHP_API_VER,
                )
            }

            let mut index = 0;
            if let Ok(version) = env::var("RUST_PHP_VERSION") {
                for (i, build) in matching.iter().enumerate() {
                    if build.version.to_string() == version {
                        index = i;
                        break;
                    }
                }
            }

            Ok(matching.remove(index))
        })
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

/// Generates bindings to the Zend API.
fn generate_bindings(defines: &[(&str, &str)], includes: &[PathBuf]) -> Result<String> {
    let mut bindgen = bindgen::Builder::default()
        .header("src/wrapper.h")
        .clang_args(
            includes
                .iter()
                .map(|inc| format!("-I{}", inc.to_string_lossy())),
        )
        .clang_args(
            defines
                .iter()
                .map(|(var, val)| format!("-D{}={}", var, val)),
        )
        .rustfmt_bindings(true)
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

fn main() -> Result<()> {
    let out_dir = env::var_os("OUT_DIR").context("Failed to get OUT_DIR")?;
    let out_path = PathBuf::from(out_dir).join("bindings.rs");
    let manifest: PathBuf = std::env::var("CARGO_MANIFEST_DIR").unwrap().into();
    for path in [
        manifest.join("src").join("wrapper.h"),
        manifest.join("src").join("wrapper.c"),
        manifest.join("allowed_bindings.rs"),
        manifest.join("windows_build.rs"),
        manifest.join("unix_build.rs"),
    ] {
        println!("cargo:rerun-if-changed={}", path.to_string_lossy());
    }

    // docs.rs runners only have PHP 7.4 - use pre-generated bindings
    if env::var("DOCS_RS").is_ok() {
        println!("cargo:warning=docs.rs detected - using stub bindings");
        println!("cargo:rustc-cfg=php_debug");
        println!("cargo:rustc-cfg=php81");
        std::fs::copy("docsrs_bindings.rs", out_path)
            .expect("failed to copy docs.rs stub bindings to out directory");
        return Ok(());
    }

    let php_build = find_php()?;
    let provider = Provider::new(&php_build)?;

    let includes = provider.get_includes()?;
    let defines = provider.get_defines()?;

    build_wrapper(&defines, &includes)?;
    let bindings = generate_bindings(&defines, &includes)?;

    let out_file =
        File::create(&out_path).context("Failed to open output bindings file for writing")?;
    let mut out_writer = BufWriter::new(out_file);
    provider.write_bindings(bindings, &mut out_writer)?;

    if php_build.version.major == 8 && php_build.version.minor == 1 {
        println!("cargo:rustc-cfg=php81");
    }
    if php_build.is_debug {
        println!("cargo:rustc-cfg=php_debug");
    }
    if php_build.is_thread_safety_enabled {
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
