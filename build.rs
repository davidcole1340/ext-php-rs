use std::{
    collections::HashSet,
    env,
    path::{Path, PathBuf},
    process::Command,
};

use bindgen::callbacks::{MacroParsingBehavior, ParseCallbacks};
use regex::{Captures, Regex};

extern crate bindgen;

// https://github.com/rust-lang/rust-bindgen/issues/687#issuecomment-450750547
#[derive(Debug)]
struct IgnoreMacros(HashSet<String>);

impl ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> MacroParsingBehavior {
        if self.0.contains(name) {
            MacroParsingBehavior::Ignore
        } else {
            MacroParsingBehavior::Default
        }
    }
}

const MIN_PHP_API_VER: u32 = 20200930;
const MAX_PHP_API_VER: u32 = 20200930;

fn main() {
    // rerun if wrapper header is changed
    println!("cargo:rerun-if-changed=src/wrapper/wrapper.h");
    println!("cargo:rerun-if-changed=src/wrapper/wrapper.c");

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
    // We could easily use grep and sed here but eventually we want to support Windows,
    // so it's easier to just use regex.
    let php_i_cmd = Command::new("php")
        .arg("-i")
        .output()
        .expect("Unable to run `php -i`. Please ensure it is visible in your PATH.");

    if !php_i_cmd.status.success() {
        let stderr = String::from_utf8(includes_cmd.stderr)
            .unwrap_or_else(|_| String::from("Unable to read stderr"));
        panic!("Error running `php -i`: {}", stderr);
    }

    let php_i = String::from_utf8(php_i_cmd.stdout).expect("unabel to parse `php -i` stdout");
    let php_api_regex = Regex::new(r"PHP API => ([0-9]+)").unwrap();
    let api_ver: Vec<Captures> = php_api_regex.captures_iter(php_i.as_ref()).collect();

    match api_ver.first() {
        Some(api_ver) => match api_ver.get(1) {
            Some(api_ver) => {
                let api_ver: u32 = api_ver.as_str().parse().unwrap();

                if api_ver < MIN_PHP_API_VER || api_ver > MAX_PHP_API_VER {
                    panic!("The current version of PHP is not supported. Current PHP API version: {}, requires a version between {} and {}", api_ver, MIN_PHP_API_VER, MAX_PHP_API_VER);
                }
            },
            None => panic!("Unable to retrieve PHP API version from `php -i`. Please check the installation and ensure it is callable.")
        },
        None => panic!("Unable to retrieve PHP API version from `php -i`. Please check the installation and ensure it is callable.")
    };

    let includes =
        String::from_utf8(includes_cmd.stdout).expect("unable to parse `php-config` stdout");

    // Build `wrapper.c` and link to Rust.
    cc::Build::new()
        .file("src/wrapper/wrapper.c")
        .includes(
            str::replace(includes.as_ref(), "-I", "")
                .split(" ")
                .map(|path| Path::new(path)),
        )
        .compile("wrapper");

    let ignore_math_h_macros = IgnoreMacros(
        vec![
            // math.h:914 - enum which uses #define for values
            "FP_NAN".into(),
            "FP_INFINITE".into(),
            "FP_ZERO".into(),
            "FP_SUBNORMAL".into(),
            "FP_NORMAL".into(),
            // math.h:237 - enum which uses #define for values
            "FP_INT_UPWARD".into(),
            "FP_INT_DOWNWARD".into(),
            "FP_INT_TOWARDZERO".into(),
            "FP_INT_TONEARESTFROMZERO".into(),
            "FP_INT_TONEAREST".into(),
        ]
        .into_iter()
        .collect(),
    );

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen::Builder::default()
        .header("src/wrapper/wrapper.h")
        .clang_args(includes.split(' '))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .parse_callbacks(Box::new(ignore_math_h_macros))
        .rustfmt_bindings(true)
        .generate()
        .expect("Unable to generate bindings for PHP")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Unable to write bindings file.");
}
