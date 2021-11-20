use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
    str,
};

use regex::Regex;

const MIN_PHP_API_VER: u32 = 20200930;
const MAX_PHP_API_VER: u32 = 20200930;

fn main() {
    // rerun if wrapper header is changed
    println!("cargo:rerun-if-changed=src/wrapper.h");
    println!("cargo:rerun-if-changed=src/wrapper.c");

    let out_dir = env::var_os("OUT_DIR").expect("Failed to get OUT_DIR");
    let out_path = PathBuf::from(out_dir).join("bindings.rs");

    // check for docs.rs and use stub bindings if required
    if env::var("DOCS_RS").is_ok() {
        println!("cargo:warning=docs.rs detected - using stub bindings");
        println!("cargo:rustc-cfg=php_debug");

        std::fs::copy("docsrs_bindings.rs", out_path)
            .expect("Unable to copy docs.rs stub bindings to output directory.");
        return;
    }

    // use php-config to fetch includes
    let includes_cmd = Command::new("php-config")
        .arg("--includes")
        .output()
        .expect("Unable to run `php-config`. Please ensure it is visible in your PATH.");

    if !includes_cmd.status.success() {
        let stderr = String::from_utf8(includes_cmd.stderr)
            .unwrap_or_else(|_| String::from("Unable to read stderr"));
        panic!("Error running `php-config`: {}", stderr);
    }

    // Ensure the PHP API version is supported.
    // We could easily use grep and sed here but eventually we want to support
    // Windows, so it's easier to just use regex.
    let php_i_cmd = Command::new("php")
        .arg("-i")
        .output()
        .expect("Unable to run `php -i`. Please ensure it is visible in your PATH.");

    if !php_i_cmd.status.success() {
        let stderr = str::from_utf8(&includes_cmd.stderr).unwrap_or("Unable to read stderr");
        panic!("Error running `php -i`: {}", stderr);
    }

    let api_ver = Regex::new(r"PHP API => ([0-9]+)")
        .unwrap()
        .captures_iter(
            str::from_utf8(&php_i_cmd.stdout).expect("Unable to parse `php -i` stdout as UTF-8"),
        )
        .next()
        .and_then(|ver| ver.get(1))
        .and_then(|ver| ver.as_str().parse::<u32>().ok())
        .expect("Unable to retrieve PHP API version from `php -i`.");

    if api_ver < MIN_PHP_API_VER || api_ver > MAX_PHP_API_VER {
        panic!("The current version of PHP is not supported. Current PHP API version: {}, requires a version between {} and {}", api_ver, MIN_PHP_API_VER, MAX_PHP_API_VER);
    }

    let includes =
        String::from_utf8(includes_cmd.stdout).expect("unable to parse `php-config` stdout");

    // Build `wrapper.c` and link to Rust.
    cc::Build::new()
        .file("src/wrapper.c")
        .includes(
            str::replace(includes.as_ref(), "-I", "")
                .split(' ')
                .map(Path::new),
        )
        .compile("wrapper");

    let mut bindgen = bindgen::Builder::default()
        .header("src/wrapper.h")
        .clang_args(includes.split(' '))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .rustfmt_bindings(true)
        .no_copy("_zend_value")
        .no_copy("_zend_string")
        .no_copy("_zend_array")
        .layout_tests(env::var("EXT_PHP_RS_TEST").is_ok());

    for binding in ALLOWED_BINDINGS.iter() {
        bindgen = bindgen
            .allowlist_function(binding)
            .allowlist_type(binding)
            .allowlist_var(binding);
    }

    bindgen
        .generate()
        .expect("Unable to generate bindings for PHP")
        .write_to_file(out_path)
        .expect("Unable to write bindings file.");

    let configure = Configure::get();

    if configure.has_zts() {
        println!("cargo:rustc-cfg=php_zts");
    }

    if configure.debug() {
        println!("cargo:rustc-cfg=php_debug");
    }
}

struct Configure(String);

impl Configure {
    pub fn get() -> Self {
        let cmd = Command::new("php-config")
        .arg("--configure-options")
        .output()
        .expect("Unable to run `php-config --configure-options`. Please ensure it is visible in your PATH.");

        if !cmd.status.success() {
            let stderr = String::from_utf8(cmd.stderr)
                .unwrap_or_else(|_| String::from("Unable to read stderr"));
            panic!("Error running `php -i`: {}", stderr);
        }

        // check for the ZTS feature flag in configure
        let stdout =
            String::from_utf8(cmd.stdout).expect("Unable to read stdout from `php-config`.");
        Self(stdout)
    }

    pub fn has_zts(&self) -> bool {
        self.0.contains("--enable-zts")
    }

    pub fn debug(&self) -> bool {
        self.0.contains("--enable-debug")
    }
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
