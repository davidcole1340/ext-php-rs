# `bool`

A boolean. Not much else to say here.

| `T` parameter | `&T` parameter | `T` Return type | PHP representation |
| ------------- | -------------- | --------------- | ------------------ |
| Yes           | No             | Yes             | Union flag         |

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

```rust
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
#[php_function]
pub fn test_bool(input: bool) -> String {
    if input {
        "Yes!".into()
    } else {
        "No!".into()
    }
}
```

## PHP example

```php
<?php

var_dump(test_bool(true)); // string(4) "Yes!"
var_dump(test_bool(false)); // string(3) "No!"
```
