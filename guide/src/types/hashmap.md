# `HashMap`

`HashMap`s are represented as associative arrays in PHP.

| `T` parameter | `&T` parameter | `T` Return type | `&T` Return type | PHP representation |
| ------------- | -------------- | --------------- | ---------------- | ------------------ |
| Yes           | No             | Yes             | No               | `ZendHashTable`    |

Converting from a zval to a `HashMap` is valid when the key is a `String`, and
the value implements `FromZval`. The key and values are copied into Rust types
before being inserted into the `HashMap`. If one of the key-value pairs has a
numeric key, the key is represented as a string before being inserted.

Converting from a `HashMap` to a zval is valid when the key implements
`AsRef<str>`, and the value implements `IntoZval`.

## Rust example

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
#[php_module]
mod module {
    # use std::collections::HashMap;
    #[php_function]
    pub fn test_hashmap(hm: HashMap<String, String>) -> Vec<String> {
        for (k, v) in hm.iter() {
            println!("k: {} v: {}", k, v);
        }

        hm.into_iter()
            .map(|(_, v)| v)
            .collect::<Vec<_>>()
    }
}
# fn main() {}
```

## PHP example

```php
<?php

var_dump(test_hashmap([
    'hello' => 'world',
    'rust' => 'php',
    'okk',
]));
```

Output:

```text
k: hello v: world
k: rust v: php
k: 0 v: okk
array(3) {
    [0] => string(5) "world",
    [1] => string(3) "php",
    [2] => string(3) "okk"
}
```
