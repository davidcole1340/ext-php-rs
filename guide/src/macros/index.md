# Macros

`ext-php-rs` comes with a set of macros that are used to annotate types which
are to be exported to PHP. This allows you to write Rust-like APIs that can be
used from PHP without fiddling around with zvals.

- [`php_module`] - Defines the function used by PHP to retrieve your extension.
- [`php_function`] - Used to export a Rust function to PHP.
- [`php_class`] - Used to export a Rust struct or enum as a PHP class.
- [`php_impl`] - Used to export a Rust `impl` block to PHP, including all
  methods and constants.

[`php_module`]: ./module.md
[`php_function`]: ./function.md
[`php_class`]: ./structs.md
[`php_impl`]: ./impl.md
