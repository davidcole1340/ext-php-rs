# Macros

`ext-php-rs` comes with a set of macros that are used to annotate types which
are to be exported to PHP. This allows you to write Rust-like APIs that can be
used from PHP without fiddling around with zvals.

- [`php_module`] - Defines the function used by PHP to retrieve your extension.
- [`php_startup`] - Defines the extension startup function used by PHP to
  initialize your extension.
- [`php_function`] - Used to export a Rust function to PHP.
- [`php_class`] - Used to export a Rust struct or enum as a PHP class.
- [`php_impl`] - Used to export a Rust `impl` block to PHP, including all
  methods and constants.
- [`php_const`] - Used to export a Rust constant to PHP as a global constant.

These macros do abuse the fact that (at the moment) proc macro expansion _seems_
to happen orderly, on one single thread. It has been stated many times that this
order is undefined behaviour ([see here]), so these macros _could_ break at any
time with a `rustc` update (let's just keep our fingers crossed).

The macros abuse this fact by storing a global state, which stores information
about all the constants, functions, methods and classes you have registered
throughout your crate. It is then read out of the state in the function tagged
with the `#[php_module]` attribute. This is why this function **must** be the
last function in your crate.

In the case the ordering does change (or we find out that it already was not in
order), the most likely solution will be having to register your PHP exports
manually inside the `#[php_module]` function.

[`php_module`]: ./module.md
[`php_startup`]: ./module_startup.md
[`php_function`]: ./function.md
[`php_class`]: ./classes.md
[`php_impl`]: ./impl.md
[`php_const`]: ./constant.md
[see here]: https://github.com/rust-lang/reference/issues/578
