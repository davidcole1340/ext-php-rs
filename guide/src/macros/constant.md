# `wrap_constant!()`

Exports a Rust constant as a global PHP constant. The constant can be any type
that implements `IntoConst`.

## Examples

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
use ext_php_rs::prelude::*;

const TEST_CONSTANT: i32 = 100;
const ANOTHER_STRING_CONST: &'static str = "Hello world!";

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .constant(wrap_constant!(TEST_CONSTANT))
        .constant(wrap_constant!(ANOTHER_STRING_CONST))
}
# fn main() {}
```

## PHP usage

```php
<?php

var_dump(TEST_CONSTANT); // int(100)
var_dump(ANOTHER_STRING_CONST); // string(12) "Hello world!"
```
