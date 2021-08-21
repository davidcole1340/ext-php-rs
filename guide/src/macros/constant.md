# `#[php_const]`

Exports a Rust constant as a global PHP constant. The constant can be any type
that implements `IntoConst`.

## Examples

```rust
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
#[php_const]
const TEST_CONSTANT: i32 = 100;

#[php_const]
const ANOTHER_STRING_CONST: &'static str = "Hello world!";
```

## PHP usage

```php
<?php

var_dump(TEST_CONSTANT); // int(100)
var_dump(ANOTHER_STRING_CONST); // string(12) "Hello world!"
```
