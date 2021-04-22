# ext-php-rs

Bindings for the Zend API to build PHP extensions natively in Rust. Inspired by [killertux/solder](https://github.com/killertux/solder) and its predecessors.

[![PHP Discorders](https://discord.com/api/guilds/115233111977099271/widget.png?style=banner1)](https://discord.gg/dphp)

## Features

This is not a set feature list, but these are the features on my roadmap. Create an issue if there's something you'd like to see!

- [x] Module definitions
- [x] Function implementation
- [x] Class implementation
    - [x] Class methods
    - [ ] Class properties
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

## Resources

- [PHP Internals Book](https://www.phpinternalsbook.com/)

## License

This software is dual-licensed under the MIT license and the Apache license (Version 2.0) at your preference.

See [LICENSE_MIT](LICENSE_MIT) and [LICENSE_APACHE](LICENCE_APACHE) for details.
