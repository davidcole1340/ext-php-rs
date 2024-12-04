# `bool`

A boolean. Not much else to say here.

| `T` parameter | `&T` parameter | `T` Return type | `&T` Return type | PHP representation |
| ------------- | -------------- | --------------- | ---------------- | ------------------ |
| Yes           | No             | Yes             | No               | Union flag         |

Booleans are not actually stored inside the zval. Instead, they are treated as
two different union types (the zval can be in a true or false state). An
equivalent structure in Rust would look like:

```rs
enum Zval {
    True,
    False,
    String(&mut ZendString),
    Long(i64),
    // ...
}
```

## Rust example

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
#[php_module]
mod module {
    #[php_function]
    pub fn test_bool(input: bool) -> String {
        if input {
            "Yes!".into()
        } else {
            "No!".into()
        }
    }
}
# fn main() {}
```

## PHP example

```php
<?php

var_dump(test_bool(true)); // string(4) "Yes!"
var_dump(test_bool(false)); // string(3) "No!"
```

## Rust example, taking by reference

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
#[php_module]
mod module {
    # use ext_php_rs::types;

    #[php_function]
    pub fn test_bool(input: &mut types::Zval) {
        input.reference_mut().unwrap().set_bool(false);
    }
}
# fn main() {}
```
