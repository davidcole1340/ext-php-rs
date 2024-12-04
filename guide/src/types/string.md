# `String`

When a `String` type is encountered, the zend string content is copied to/from a
Rust `String` object. If the zval does not contain a string, it will attempt to
read a `double` from the zval and convert it into a `String` object.

| `T` parameter | `&T` parameter | `T` Return type | `&T` Return type | PHP representation       |
| ------------- | -------------- | --------------- | ---------------- | ------------------------ |
| Yes           | No             | Yes             | No               | `zend_string` (C-string) |

Internally, PHP stores strings in `zend_string` objects, which is a refcounted C
struct containing the string length with the content of the string appended to
the end of the struct based on how long the string is. Since the string is
NUL-terminated, you cannot have any NUL bytes in your string, and an error will
be thrown if one is encountered while converting a `String` to a zval.

## Rust example

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
#[php_module]
mod module {
    #[php_function]
    pub fn str_example(input: String) -> String {
        format!("Hello {}", input)
    }
}
# fn main() {}
```

## PHP example

```php
<?php

var_dump(str_example("World")); // string(11) "Hello World"
var_dump(str_example(5)); // string(7) "Hello 5"
```
