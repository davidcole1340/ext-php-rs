# `Iterable`

`Iterable`s are represented either by an `array` or `Traversable` type.

| `T` parameter | `&T` parameter | `T` Return type | `&T` Return type | PHP representation               |
|---------------|----------------|-----------------| ---------------- |----------------------------------|
| Yes           | No             | No              | No               | `ZendHashTable` or `ZendIterator` |

Converting from a zval to a `Iterable` is valid when the value is either an array or an object 
that implements the `Traversable` interface. This means that any value that can be used in a
`foreach` loop can be converted into a `Iterable`.

## Rust example

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
# use ext_php_rs::types::Iterable;
#[php_function]
pub fn test_iterable(mut iterable: Iterable) {
    for (k, v) in iterable.iter().expect("cannot get iterable") {
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

$array = [
    'hello' => 'world',
    'rust' => 'php',
    'okk',
];

test_iterable($generator());
test_iterable($array);
```

Output:

```text
k: hello v: world
k: rust v: php
k: 0 v: okk
k: hello v: world
k: rust v: php
k: 0 v: okk
```
