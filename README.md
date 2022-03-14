# ext-php-rs

[<img align="right" src="https://discord.com/api/guilds/115233111977099271/widget.png?style=banner2">](https://discord.gg/dphp)

Bindings and abstractions for the Zend API to build PHP extensions natively in
Rust.

## Example

Export a simple function `function hello_world(string $name): string` to PHP:

```rust
#![cfg_attr(windows, feature(abi_vectorcall))]

use ext_php_rs::prelude::*;

/// Gives you a nice greeting!
/// 
/// @param string $name Your name.
/// 
/// @return string Nice greeting!
#[php_function]
pub fn hello_world(name: String) -> String {
    format!("Hello, {}!", name)
}

// Required to register the extension with PHP.
#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
```

Use [`cargo-php`] to build IDE stubs and install the extension:

```text
$ cargo install cargo-php
  Installing cargo-php v0.1.0
$ cargo php stubs --stdout
  Compiling example-ext v0.1.0
  Finished dev [unoptimized + debuginfo] target(s) in 3.57s
<?php

// Stubs for example-ext

/**
 * Gives you a nice greeting!
 *
 * @param string $name Your name.
 *
 * @return string Nice greeting!
 */
function hello_world(string $name): string {}
$ cargo php install --release
  Compiling example-ext v0.1.0
  Finished release [optimized] target(s) in 1.68s
Are you sure you want to install the extension `example-ext`? yes
$ php -m
[PHP Modules]
// ...
example-ext
// ...
```

Calling the function from PHP:

```php
var_dump(hello_world("David")); // string(13) "Hello, David!"
```

For more examples read the library
[guide](https://davidcole1340.github.io/ext-php-rs).

[`cargo-php`]: https://crates.io/crates/cargo-php

## Features

- **Easy to use:** The built-in macros can abstract away the need to interact
  with the Zend API, such as Rust-type function parameter abstracting away
  interacting with Zend values.
- **Lightweight:** You don't have to use the built-in helper macros. It's
  possible to write your own glue code around your own functions.
- **Extensible:** Implement `IntoZval` and `FromZval` for your own custom types,
  allowing the type to be used as function parameters and return types.

## Goals

Our main goal is to **make extension development easier.**

- Writing extensions in C can be tedious, and with the Zend APIs limited
  documentation can be intimidating.
- Rust's modern language features and feature-full standard library are big
  improvements on C.
- Abstracting away the raw Zend APIs allows extensions to be developed faster
  and with more confidence.
- Abstractions also allow us to support future (and potentially past) versions
  of PHP without significant changes to extension code.

## Documentation

The library guide can be read
[here](https://davidcole1340.github.io/ext-php-rs/guide).

The project is documented in-line, so viewing the `cargo` documentation is the
best resource at the moment. This can be viewed at [docs.rs].

## Requirements

- PHP 8.0 or later
  - No support is planned for lower versions.
- Linux, macOS or Windows-based operating system
- Rust - no idea which version
- Clang 3.9 or greater

See the following links for the dependency crate requirements:

- [`cc`](https://github.com/alexcrichton/cc-rs#compile-time-requirements)
- [`bindgen`](https://rust-lang.github.io/rust-bindgen/requirements.html)

### Windows Support

Windows has some extra requirements:

- Extensions can only be compiled for PHP installations sourced from
  [windows.php.net].
- Only PHP installations compiled with MSVC are supported (no support for
  `x86_64-pc-windows-gnu`).
- Microsoft Visual C++ must be installed. The compiler version must match or be
  older than the compiler that was used to compile your PHP installation (at the
  time of writing Visual Studio 2019 is supported).
- Extensions can only be compiled with nightly Rust, and the `abi_vectorcall`
  feature must be enabled in your crates's Cargo Features

## Cargo Features

All features are disabled by default.

- `closure` - Enables the ability to return Rust closures to PHP. Creates a new
  class type, `RustClosure`.
- `anyhow` - Implements `Into<PhpException>` for `anyhow::Error`, allowing you
  to return anyhow results from PHP functions. Supports anyhow v1.x.

## Usage

This project only works for PHP >= 8.0 (for now). Due to the fact that the PHP
extension system relies heavily on C macros (which cannot be exported to Rust
easily), structs have to be hard coded in.

Check out one of the example projects:

- [anonaddy-sequoia](https://gitlab.com/willbrowning/anonaddy-sequoia) - Sequoia
  encryption PHP extension.
- [opus-php](https://github.com/davidcole1340/opus-php) - Audio encoder for the
  Opus codec in PHP.

## Contributions

Contributions are very much welcome. I am a novice Rust developer and any
suggestions are wanted and welcome. Feel free to file issues and PRs through
Github.

Contributions welcome include:

- Documentation expansion (examples in particular!)
- Safety reviews (especially if you have experience with Rust and the Zend API).
- Bug fixes and features.
- Feature requests.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## Resources

- [PHP Internals Book](https://www.phpinternalsbook.com/)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE_APACHE] or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE_MIT] or <http://opensource.org/licenses/MIT>)

at your option.

[LICENSE_APACHE]: https://github.com/davidcole1340/ext-php-rs/blob/master/LICENSE_APACHE
[LICENSE_MIT]: https://github.com/davidcole1340/ext-php-rs/blob/master/LICENSE_MIT
[docs.rs]: https://docs.rs/ext-php-rs
