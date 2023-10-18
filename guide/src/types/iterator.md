# `ZendIterator`

`ZendIterator`s are represented by the `Traversable` type.

| `T` parameter | `&T` parameter | `T` Return type | `&T` Return type | PHP representation |
|---------------| -------------- |-----------------| ---------------- | ------------------ |
| No            | Yes            | No              | No               | `ZendIterator`    |

Converting from a zval to a `ZendIterator` is valid when there is an associated iterator to 
the variable. This means that any value, at the exception of an `array`, that can be used in 
a `foreach` loop can be converted into a `ZendIterator`. As an example, a `Generator` can be
used but also a the result of a `query` call with `PDO`.

## Rust example

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
# use ext_php_rs::types::ZendIterator;
#[php_function]
pub fn test_iterator(iterator: &mut ZendIterator) {
    for (k, v) in iterator.iter() {
        println!("k: {} v: {}", k, v.string().unwrap());
    }
}
# fn main() {}
```

## PHP example

```php
<?php

$generator = function() {
    yield 'hello' => 'world';
    yield 'rust' => 'php';
    yield 'okk';
};

test_iterator($generator());
```

Output:

```text
k: hello v: world
k: rust v: php
k: 0 v: okk
```
