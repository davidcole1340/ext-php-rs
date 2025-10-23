//! This could be a `.cargo/config.toml` file, however, when working in a
//! workspace only the top level config file is read. For development it's
//! easier to make this a build script, even though it does add to the compile
//! time.

fn main() {
    println!("cargo:rustc-link-arg-bins=-rdynamic");

    // On musl targets, allow undefined symbols for PHP runtime functions
    // sigsetjmp and PHP symbols will be resolved when the binary runs with PHP loaded
    #[cfg(target_env = "musl")]
    {
        println!("cargo:rustc-link-arg-bins=-Wl,--unresolved-symbols=ignore-all");
    }

    // On macOS, use dynamic lookup for undefined symbols
    // This allows PHP symbols to be resolved at runtime
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-arg-bins=-Wl,-undefined,dynamic_lookup");
    }
}
