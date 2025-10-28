# cargo-php

Installs extensions and generates stub files for PHP extensions generated with
`ext-php-rs`.

- [`ext-php-rs`](https://github.com/extphprs/ext-php-rs)

## Installation

Install with Cargo: `cargo install cargo-php --locked`.

## Usage

```text
$ cargo php --help
cargo-php 0.1.0

David Cole <david.cole1340@gmail.com>

Installs extensions and generates stub files for PHP extensions generated with `ext-php-rs`.

USAGE:
    cargo-php <SUBCOMMAND>

OPTIONS:
    -h, --help
            Print help information

    -V, --version
            Print version information

SUBCOMMANDS:
    help
            Print this message or the help of the given subcommand(s)
    install
            Installs the extension in the current PHP installation
    remove
            Removes the extension in the current PHP installation
    stubs
            Generates stub PHP files for the extension

$ cargo php install --help
cargo-php-install

Installs the extension in the current PHP installation.

This copies the extension to the PHP installation and adds the extension to a PHP configuration
file.

USAGE:
    cargo-php install [OPTIONS]

OPTIONS:
        --disable
            Installs the extension but doesn't enable the extension in the `php.ini` file

    -h, --help
            Print help information

        --ini-path <INI_PATH>
            Path to the `php.ini` file to update with the new extension

        --install-dir <INSTALL_DIR>
            Changes the path that the extension is copied to. This will not activate the extension
            unless `ini_path` is also passed

        --manifest <MANIFEST>
            Path to the Cargo manifest of the extension. Defaults to the manifest in the directory
            the command is called

        --release
            Whether to install the release version of the extension

        --yes
            Bypasses the confirmation prompt

$ cargo php remove --help
cargo-php-remove

Removes the extension in the current PHP installation.

This deletes the extension from the PHP installation and also removes it from the main PHP
configuration file.

USAGE:
    cargo-php remove [OPTIONS]

OPTIONS:
    -h, --help
            Print help information

        --ini-path <INI_PATH>
            Path to the `php.ini` file to remove the extension from

        --install-dir <INSTALL_DIR>
            Changes the path that the extension will be removed from. This will not remove the
            extension from a configuration file unless `ini_path` is also passed

        --manifest <MANIFEST>
            Path to the Cargo manifest of the extension. Defaults to the manifest in the directory
            the command is called

        --yes
            Bypasses the confirmation prompt

$ cargo php stubs --help
cargo-php-stubs

Generates stub PHP files for the extension.

These stub files can be used in IDEs to provide typehinting for extension classes, functions and
constants.

USAGE:
    cargo-php stubs [OPTIONS] [EXT]

ARGS:
    <EXT>
            Path to extension to generate stubs for. Defaults for searching the directory the
            executable is located in

OPTIONS:
    -h, --help
            Print help information

        --manifest <MANIFEST>
            Path to the Cargo manifest of the extension. Defaults to the manifest in the directory
            the command is called.

            This cannot be provided alongside the `ext` option, as that option provides a direct
            path to the extension shared library.

    -o, --out <OUT>
            Path used to store generated stub file. Defaults to writing to `<ext-name>.stubs.php` in
            the current directory

        --stdout
            Print stubs to stdout rather than write to file. Cannot be used with `out`
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE_APACHE] or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE_MIT] or <http://opensource.org/licenses/MIT>)

at your option.

[LICENSE_APACHE]: https://github.com/extphprs/ext-php-rs/blob/master/LICENSE_APACHE
[LICENSE_MIT]: https://github.com/extphprs/ext-php-rs/blob/master/LICENSE_MIT
