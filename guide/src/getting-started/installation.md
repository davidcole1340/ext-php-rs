# Installation

To get started using `ext-php-rs` you will need both a Rust toolchain
and a PHP development environment. We'll cover each of these below.

## Rust toolchain

First, make sure you have rust installed on your system.
If you haven't already done so you can do so by following the instructions [here](https://www.rust-lang.org/tools/install).
`ext-php-rs` runs on both the stable and nightly versions so you can choose whichever one fits you best.

## PHP development environment

In order to develop PHP extensions, you'll need the following installed on your system:

1. The PHP CLI executable itself
2. The PHP development headers
3. The `php-config` binary

While the easiest way to get started is to use the packages provided by your distribution,
we recommend building PHP from source.

**NB:** To use `ext-php-rs` you'll need at least PHP 8.0.

### Using a package manager

```sh
# Debian and derivatives
apt install php-dev
# Arch Linux
pacman -S php
# Fedora
dnf install php-devel
# Homebrew
brew install php
```

### Compiling PHP from source

Please refer to this [PHP internals book chapter](https://www.phpinternalsbook.com/php7/build_system/building_php.html)
for an in-depth guide on how to build PHP from source.

**TL;DR;** use the following commands to build a minimal development version
with debug symbols enabled.

```sh
# clone the php-src repository
git clone https://github.com/php/php-src.git
cd php-src
# by default you will be on the master branch, which is the current
# development version. You can check out a stable branch instead:
git checkout PHP-8.1
./buildconf
PREFIX="${HOME}/build/php"
.configure --prefix="${PREFIX}" \
    --enable-debug \
    --disable-all --disable-cgi
make -j "$(nproc)"
make install
```

The PHP CLI binary should now be located at `${PREFIX}/bin/php`
and the `php-config` binary at `${PREFIX}/bin/php-config`.

## Next steps

Now that we have our development environment in place,
let's go [build an extension](./hello_world.md) !
