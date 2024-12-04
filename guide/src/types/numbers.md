# Primitive Numbers

Primitive integers include `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`,
`isize`, `usize`, `f32` and `f64`.

| `T` parameter | `&T` parameter | `T` Return type | `&T` Return type | PHP representation                                                               |
| ------------- | -------------- | --------------- | ---------------- | -------------------------------------------------------------------------------- |
| Yes           | No             | Yes             | No               | `i32` on 32-bit platforms, `i64` on 64-bit platforms, `f64` platform-independent |

Note that internally, PHP treats **all** of these integers the same (a 'long'),
and therefore it must be converted into a long to be stored inside the zval. A
long is always signed, and the size will be 32-bits on 32-bit platforms and
64-bits on 64-bit platforms.

Floating point numbers are always stored in a `double` type (`f64`), regardless
of platform. Note that converting a zval into a `f32` will lose accuracy.

This means that converting `i64`, `u32`, `u64`, `isize` and `usize` _can_ fail
depending on the value and the platform, which is why all zval conversions are
fallible.

## Rust example

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
#[php_module]
mod module {
    #[php_function]
    pub fn test_numbers(a: i32, b: u32, c: f32) -> u8 {
        println!("a {} b {} c {}", a, b, c);
        0
    }
}
# fn main() {}
```

## PHP example

```php
<?php

test_numbers(5, 10, 12.5); // a 5 b 10 c 12.5
```
