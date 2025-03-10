# `Vec`

Vectors can contain any type that can be represented as a zval. Note that the
data contained in the array will be copied into Rust types and stored inside the
vector. The internal representation of a PHP array is discussed below.

| `T` parameter | `&T` parameter | `T` Return type | `&T` Return type | PHP representation |
| ------------- | -------------- | --------------- | ---------------- | ------------------ |
| Yes           | No             | Yes             | No               | `ZendHashTable`    |

Internally, PHP arrays are hash tables where the key can be an unsigned long or
a string. Zvals are contained inside arrays therefore the data does not have to
contain only one type.

When converting into a vector, all values are converted from zvals into the
given generic type. If any of the conversions fail, the whole conversion will
fail.

## Rust example

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
#[php_module]
mod module {
    #[php_function]
    pub fn test_vec(vec: Vec<String>) -> String {
        vec.join(" ")
    }
}
# fn main() {}
```

## PHP example

```php
<?php

var_dump(test_vec(['hello', 'world', 5])); // string(13) "hello world 5"
```
