# `#[php_startup]`

Used to define the PHP extension startup function. This function is used to
register extension classes and constants with the PHP interpreter.

This function is automatically generated if you have registered classes or
constants and have not already used this macro. If you do use this macro, it
will be automatically registered in the `get_module` function when you use the
`#[php_module]` attribute.

Most of the time you won't need to use this macro as the startup function will
be automatically generated when required (if not already defined).

Read more about what the module startup function is used for
[here.](https://www.phpinternalsbook.com/php7/extensions_design/php_lifecycle.html#module-initialization-minit)

## Example

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
#[php_startup]
pub fn startup_function() {

}
# fn main() {}
```
