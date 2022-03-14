# `#[php_module]`

The module macro is used to annotate the `get_module` function, which is used by
the PHP interpreter to retrieve information about your extension, including the
name, version, functions and extra initialization functions. Regardless if you
use this macro, your extension requires a `extern "C" fn get_module()` so that
PHP can get this information.

Using the macro, any functions annotated with the `php_function` macro will be
automatically registered with the extension in this function. If you have
defined any constants or classes with their corresponding macros, a 'module
startup' function will also be generated if it has not already been defined.

Automatically registering these functions requires you to define the module
function **after** all other functions have been registered, as macros are
expanded in-order, therefore this macro will not know that other functions have
been used after.

The function is renamed to `get_module` if you have used another name. The
function is passed an instance of `ModuleBuilder` which allows you to register
the following (if required):

- Extension and request startup and shutdown functions.
  - Read more about the PHP extension lifecycle
    [here](https://www.phpinternalsbook.com/php7/extensions_design/php_lifecycle.html).
- PHP extension information function
  - Used by the `phpinfo()` function to get information about your extension.
- Functions not automatically registered

Classes and constants are not registered in the `get_module` function. These are
registered inside the extension startup function.

## Usage

```rust,ignore
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
# use ext_php_rs::{info_table_start, info_table_row, info_table_end};
# use ext_php_rs::php::module::ModuleEntry;
/// Used by the `phpinfo()` function and when you run `php -i`.
/// This will probably be simplified with another macro eventually!
pub extern "C" fn php_module_info(_module: *mut ModuleEntry) {
    info_table_start!();
    info_table_row!("my extension", "enabled");
    info_table_end!();
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module.info_function(php_module_info)
}
```
