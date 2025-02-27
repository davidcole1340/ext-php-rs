# `Binary`

Binary data is represented as a string in PHP. The most common source of this
data is from the [`pack`] and [`unpack`] functions. It allows you to transfer
arbitrary binary data between Rust and PHP.

| `T` parameter | `&T` parameter | `T` Return type | `&T` Return type | PHP representation |
| ------------- | -------------- | --------------- | ---------------- | ------------------ |
| Yes           | No             | Yes             | No               | `zend_string`      |

The binary type is represented as a string in PHP. Although not encoded, the
data is converted into an array and then the pointer to the data is set as the
string pointer, with the length of the array being the length of the string.

`Binary<T>` is valid when `T` implements `Pack`. This is currently implemented
on most primitive numbers (i8, i16, i32, i64, u8, u16, u32, u64, isize, usize,
f32, f64).

[`pack`]: https://www.php.net/manual/en/function.pack.php
[`unpack`]: https://www.php.net/manual/en/function.unpack.php

## Rust Usage

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
use ext_php_rs::prelude::*;

#[php_module]
mod module {
    use ext_php_rs::binary::Binary;

    #[php_function]
    pub fn test_binary(input: Binary<u32>) -> Binary<u32> {
        for i in input.iter() {
            println!("{}", i);
        }

        vec![5, 4, 3, 2, 1]
            .into_iter()
            .collect::<Binary<_>>()
    }
}
# fn main() {}
```

## PHP Usage

```php
<?php

$data = pack('L*', 1, 2, 3, 4, 5);
$output = unpack('L*', test_binary($data));
var_dump($output); // array(5) { [0] => 5, [1] => 4, [2] => 3, [3] => 2, [4] => 1 }
```
