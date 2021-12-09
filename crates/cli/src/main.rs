// Mock macro for the `allowed_bindings.rs` script.
macro_rules! bind {
    ($($s: ident),*) => {
        cargo_php::stub_symbols!($($s),*);
    }
}

include!("../allowed_bindings.rs");

fn main() -> cargo_php::CrateResult {
    cargo_php::run()
}
