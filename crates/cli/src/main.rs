//! # `cargo-php` CLI
//!
//! Installs extensions and generates stub files for PHP extensions generated
//! with `ext-php-rs`. Use `cargo php --help` for more information.

/// Mock macro for the `allowed_bindings.rs` script.
#[cfg(not(windows))]
macro_rules! bind {
    ($($s: ident),*) => {
        cargo_php::stub_symbols!($($s),*);
    }
}

#[cfg(not(windows))]
include!("../allowed_bindings.rs");

fn main() -> cargo_php::CrateResult {
    cargo_php::run()
}
