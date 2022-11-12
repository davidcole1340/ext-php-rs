# Hello World

## Project Setup

We will start by creating a new Rust library crate:

```sh
$ cargo new hello_world --lib
$ cd hello_world
```

### `Cargo.toml`

Let's set up our crate by adding `ext-php-rs` as a dependency and setting the
crate type to `cdylib`. Update the `Cargo.toml` to look something like so:

```toml
[package]
name = "hello_world"
version = "0.1.0"
edition = "2018"

[lib]
crate-type = ["cdylib"]

[dependencies]
ext-php-rs = "*"

[profile.release]
strip = "debuginfo"
```

### `.cargo/config.toml`

When compiling for Linux and macOS, we do not link directly to PHP, rather PHP
will dynamically load the library. We need to tell the linker it's ok to have
undefined symbols (as they will be resolved when loaded by PHP).

On Windows, we also need to switch to using the `rust-lld` linker.

> Microsoft Visual C++'s `link.exe` is supported, however you may run into
> issues if your linker is not compatible with the linker used to compile PHP.

We do this by creating a Cargo config file in `.cargo/config.toml` with the
following contents:

```toml
{{#include ../../../.cargo/config.toml}}
```

## Writing our extension

### `src/lib.rs`

Let's actually write the extension code now. We start by importing the
`ext-php-rs` prelude, which contains most of the imports required to make a
basic extension. We will then write our basic `hello_world` function, which will
take a string argument for the callers name, and we will return another string.
Finally, we write a `get_module` function which is used by PHP to find out about
your module. We must provide the defined function to the given `ModuleBuilder`
and then return the same object.

We also need to enable the `abi_vectorcall` feature when compiling for Windows
(the first line). This is a nightly-only feature so it is recommended to use
the `#[cfg_attr]` macro to not enable the feature on other operating systems.

```rust,ignore
#![cfg_attr(windows, feature(abi_vectorcall))]
use ext_php_rs::prelude::*;

#[php_function]
pub fn hello_world(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module.function(wrap_function!(hello_world))
}
```

## Building the extension

Now let's build our extension.
This is done through `cargo` like any other Rust crate.

If you installed php using a package manager in the previous chapter
(or if the `php` and `php-config` binaries are already in your `$PATH`),
then you can just run

```sh
cargo build
```

If you have multiple PHP versions in your PATH, or your installation
resides in a custom location, you can use the following environment variables:

```sh
# explicitly specifies the path to the PHP executable:
export PHP=/path/to/php
# explicitly specifies the path to the php-config executable:
export PHP_CONFIG=/path/to/php-config
```

As an alternative, if you compiled PHP from source and installed it under
it's own prefix (`configure --prefix=/my/prefix`), you can just put
this prefix in front of your PATH:

```sh
export PATH="/my/prefix:${PATH}"
```

Once you've setup these variables, you can just run

```sh
cargo build
```

Cargo will track changes to these environment variables and rebuild the library accordingly.

## Testing our extension

The extension we just built is stored inside the cargo target directory:
`target/debug` if you did a debug build, `target/release` for release builds.

The extension file name is OS-dependent. The naming works as follows:

- let `S` be the empty string
- append to `S` the value of [std::env::consts::DLL_PREFIX](https://doc.rust-lang.org/std/env/consts/constant.DLL_PREFIX.html)
  (empty on windows, `lib` on unixes)
- append to `S` the lower-snake-case version of your crate name
- append to `S` the value of [std::env::consts::DLL_SUFFIX](https://doc.rust-lang.org/std/env/consts/constant.DLL_SUFFIX.html)
  (`.dll` on windows, `.dylib` on macOS, `.so` on other unixes).
- set the filename to the value of `S`

Which in our case would give us:

- linux: `libhello_world.so`
- macOS: `libhello_world.dylib`
- windows: `hello_world.dll`

Now we need a way to tell the PHP CLI binary to load our extension.
There are [several ways to do that](https://www.phpinternalsbook.com/php7/build_system/building_extensions.html#loading-shared-extensions).
For now we'll simply pass the `-d extension=/path/to/extension` option to the PHP CLI binary.

Let's make a test script:

### `test.php`

```php
<?php

var_dump(hello_world("David"));
```

And run it:

```sh
$ php -d extension=./target/debug/libhello_world.so test.php
string(13) "Hello, David!"
```
