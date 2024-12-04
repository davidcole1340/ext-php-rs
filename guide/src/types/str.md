# `&str`

A borrowed string. When this type is encountered, you are given a reference to
the actual zend string memory, rather than copying the contents like if you were
taking an owned `String` argument.

| `T` parameter | `&T` parameter | `T` Return type | `&T` Return type | PHP representation       |
| ------------- | -------------- | --------------- | ---------------- | ------------------------ |
| No            | Yes            | No              | Yes              | `zend_string` (C-string) |

Note that you cannot expect the function to operate the same by swapping out
`String` and `&str` - since the zend string memory is read directly, this
library does not attempt to parse `double` types as strings.

See the [`String`](./string.md) for a deeper dive into the internal structure of
PHP strings.

## Rust example

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
#[php_module]
mod module {
    #[php_function]
    pub fn str_example(input: &str) -> String {
        format!("Hello {}", input)
    }

    #[php_function]
    pub fn str_return_example() -> &'static str {
        "Hello from Rust"
    }
}
# fn main() {}
```

## PHP example

```php
<?php

var_dump(str_example("World")); // string(11) "Hello World"
var_dump(str_example(5)); // Invalid

var_dump(str_return_example());
```
