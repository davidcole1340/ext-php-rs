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
- Rust - no idea which version

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

```
MIT License

Copyright (c) 2021 David Cole <david.cole1340@gmail.com>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
