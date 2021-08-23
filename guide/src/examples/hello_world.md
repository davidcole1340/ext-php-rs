# Hello World

Let's create a basic PHP extension. We will start by creating a new Rust library
crate:

```sh
$ cargo new hello_world --lib
$ cd hello_world
```

Let's set up our crate by adding `ext-php-rs` as a dependency and setting the
crate type to `cdylib`. Update the `Cargo.toml` to look something like so:

### `Cargo.toml`

```toml
[package]
name = "hello_world"
version = "0.1.0"
edition = "2018"

[dependencies]
ext-php-rs = "0.2"

[lib]
crate-type = ["cdylib"]
```

As the linker will not be able to find the PHP installation that we are
dynamically linking to, we need to enable dynamic linking with undefined
symbols. We do this by creating a Cargo config file in `.cargo/config.toml` with
the following contents:

### `.cargo/config.toml`

```toml
[build]
rustflags = ["-C", "link-arg=-Wl,-undefined,dynamic_lookup"]
```

Let's actually write the extension code now. We start by importing the
`ext-php-rs` prelude, which contains most of the imports required to make a
basic extension. We will then write our basic `hello_world` function, which will
take a string argument for the callers name, and we will return another string.
Finally, we write a `get_module` function which is used by PHP to find out about
your module. The `#[php_module]` attribute automatically registers your new
function so we don't need to do anything except return the `ModuleBuilder` that
we were given.

### `src/lib.rs`

```rust
# extern crate ext_php_rs;
use ext_php_rs::prelude::*;

#[php_function]
pub fn hello_world(name: String) -> String {
    format!("Hello, {}!", name)
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
```

Let's make a test script.

### `test.php`

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
on Linux it will be `libhello_world.so` and on macOS it will be
`libhello_world.dylib`.

```
$ cargo build
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
$ php -dextension=./target/debug/libhello_world.dylib test.php
string(13) "Hello, David!"
```
