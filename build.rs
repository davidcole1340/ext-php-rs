use std::{collections::{HashSet}, env, path::PathBuf, process::Command};

use bindgen::callbacks::{MacroParsingBehavior, ParseCallbacks};

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

fn main() {
    // rerun if wrapper header is changed
    println!("cargo:rerun-if-changed=wrapper.h");

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

    let includes = String::from_utf8(includes_cmd.stdout)
        .expect("unable to parse `php-config` stdout");

    let ignore_math_h_macros = IgnoreMacros(vec![
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
    ].into_iter().collect());

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args(includes.split(" "))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .parse_callbacks(Box::new(ignore_math_h_macros))
        .rustfmt_bindings(true)
        .generate()
        .expect("Unable to generate bindings for PHP")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Unable to write bindings file.");
}
