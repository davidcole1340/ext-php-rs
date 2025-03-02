# `#[php_const]` Attribute

Exports a Rust constant as a global PHP constant. The constant can be any type
that implements `IntoConst`.

The `wrap_constant!()` macro can be used to simplify the registration of constants.
It sets the name and doc comments for the constant.

## Examples

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
use ext_php_rs::prelude::*;

#[php_const]
const TEST_CONSTANT: i32 = 100;

#[php_const]
const ANOTHER_STRING_CONST: &'static str = "Hello world!";

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .constant(wrap_constant!(TEST_CONSTANT))
        .constant(("MANUAL_CONSTANT", ANOTHER_STRING_CONST, &[]))
}
# fn main() {}
```

## PHP usage

```php
<?php

var_dump(TEST_CONSTANT); // int(100)
var_dump(MANUAL_CONSTANT); // string(12) "Hello world!"
```
