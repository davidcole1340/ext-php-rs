# ext-php-rs

[<img align="right" src="https://discord.com/api/guilds/115233111977099271/widget.png?style=banner2">](https://discord.gg/dphp)

Bindings for the Zend API to build PHP extensions natively in Rust. Inspired by [killertux/solder](https://github.com/killertux/solder) and its predecessors.

## Documentation

We are currently unable to deploy our documentation to `docs.rs` due to the crate requiring PHP 8.0, which is unavailable in the default Ubuntu repositories.
Documentation can be viewed [here](https://davidcole1340.github.io/ext-php-rs/). It is generated from the latest `master` branch. Documentation will be moved to `docs.rs` when Ubuntu updates its repositories
to PHP 8.0.

## Features

This is not a set feature list, but these are the features on my roadmap. Create an issue if there's something you'd like to see!

- [x] Module definitions
- [x] Function implementation
- [x] Class implementation
    - [x] Class methods
    - [x] Class properties
    - [x] Class constants
- [x] Module constants
- [x] Calling PHP functions

## Requirements

- PHP 8.0 or later
     - No support is planned for lower versions.
- Linux or Darwin-based OS
- Rust - no idea which version
- Clang 3.9 or greater

See the following links for the dependency crate requirements:

- [`cc`](https://github.com/alexcrichton/cc-rs#compile-time-requirements)
- [`bindgen`](https://rust-lang.github.io/rust-bindgen/requirements.html)


## Usage

This project only works for PHP >= 8.0 (for now). Due to the fact that the PHP extension system relies heavily on C macros (which cannot be exported to Rust easily), structs have to be hard coded in.

There is only inline documentation for the time being. Starting by creating a C extension is a good start as well.

Check out one of the example projects:

- [ext-skel](example/skel) - Testbed for testing the library. Check out previous commits as well to see what else is possible.
- [opus-php](https://github.com/davidcole1340/opus-php/tree/rewrite_rs) - Work-in-progress extension to use the Opus library in PHP.

## Contributions

Contributions are very much welcome. I am a novice Rust developer and any suggestions are wanted and welcome. Feel free to file issues and PRs through Github.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## Resources

- [PHP Internals Book](https://www.phpinternalsbook.com/)

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE_APACHE](LICENSE_APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE_MIT](LICENSE_MIT) or http://opensource.org/licenses/MIT)

at your option.
