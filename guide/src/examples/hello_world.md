# Hello World

Let's create a basic PHP extension. We will start by creating a new Rust library
crate:

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

[dependencies]
ext-php-rs = "*"

[lib]
crate-type = ["cdylib"]
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

### `src/lib.rs`

Let's actually write the extension code now. We start by importing the
`ext-php-rs` prelude, which contains most of the imports required to make a
basic extension. We will then write our basic `hello_world` function, which will
take a string argument for the callers name, and we will return another string.
Finally, we write a `get_module` function which is used by PHP to find out about
your module. The `#[php_module]` attribute automatically registers your new
function so we don't need to do anything except return the `ModuleBuilder` that
we were given.

We also need to enable the `abi_vectorcall` feature when compiling for Windows.
This is a nightly-only feature so it is recommended to use the `#[cfg_attr]`
macro to not enable the feature on other operating systems.

```rust,ignore
#![cfg_attr(windows, feature(abi_vectorcall))]
use ext_php_rs::prelude::*;

#[php_function]
pub fn hello_world(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
```

### `test.php`

Let's make a test script.

```php
<?php

var_dump(hello_world("David"));
```

Now let's build our extension and run our test script. This is done through
`cargo` like any other Rust crate. It is required that the `php-config`
executable is able to be found by the `ext-php-rs` build script.

The extension is stored inside `target/debug` (if you did a debug build,
`target/release` for release builds). The file name will be based on your crate
name, so for us it will be `libhello_world`. The extension is based on your OS -
on Linux it will be `libhello_world.so`, on macOS it will be
`libhello_world.dylib` and on Windows it will be `hello_world.dll` (no `lib`
prefix).

```sh
$ cargo build
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
$ php -dextension=./target/debug/libhello_world.dylib test.php
string(13) "Hello, David!"
```
