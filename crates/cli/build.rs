//! This could be a `.cargo/config.toml` file, however, when working in a
//! workspace only the top level config file is read. For development it's
//! easier to make this a build script, even though it does add to the compile
//! time.

fn main() {
    println!("cargo:rustc-link-arg-bins=-rdynamic");
}
