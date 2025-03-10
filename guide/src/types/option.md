# `Option<T>`

Options are used for optional and nullable parameters, as well as null returns.
It is valid to be converted to/from a zval as long as the underlying `T` generic
is also able to be converted to/from a zval.

| `T` parameter | `&T` parameter | `T` Return type | `&T` Return type | PHP representation                 |
| ------------- | -------------- | --------------- | ---------------- | ---------------------------------- |
| Yes           | No             | Yes             | No               | Depends on `T`, `null` for `None`. |

Using `Option<T>` as a parameter indicates that the parameter is nullable. If
null is passed, a `None` value will be supplied. It is also used in the place of
optional parameters. If the parameter is not given, a `None` value will also be
supplied.

Returning `Option<T>` is a nullable return type. Returning `None` will return
null to PHP.

## Rust example

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
#[php_module]
mod module {
    #[php_function]
    pub fn test_option_null(input: Option<String>) -> Option<String> {
        input.map(|input| format!("Hello {}", input).into())
    }
}
# fn main() {}
```

## PHP example

```php
<?php

var_dump(test_option_null("World")); // string(11) "Hello World"
var_dump(test_option_null()); // null
```
