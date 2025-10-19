fn main() {
    // On macOS, allow undefined symbols for PHP extensions
    // PHP symbols will be resolved at runtime when the extension is loaded by PHP
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-arg=-Wl,-undefined,dynamic_lookup");
    }
}
