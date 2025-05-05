# `#[php]` Attributes

There are a number of attributes that can be used to annotate elements in your
extension.

Multiple `#[php]` attributes will be combined. For example, the following will
be identical:

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
#[php_function]
#[php(name = "hi_world")]
#[php(defaults(a = 1, b = 2))]
fn hello_world(a: i32, b: i32) -> i32 {
    a + b
}
# fn main() {}
```

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
#[php_function]
#[php(name = "hi_world", defaults(a = 1, b = 2))]
fn hello_world(a: i32, b: i32) -> i32 {
    a + b
}
# fn main() {}
```

Which attributes are available depends on the element you are annotating:

| Attribute        | `const` | `fn` | `struct` | `struct` Field | `impl` | `impl` `const` | `impl` `fn` |
| ---------------- | ------- | ---- | -------- | -------------- | ------ | -------------- | ----------- |
| name             | ✅      | ✅   | ✅       | ✅             | ❌     | ✅             | ✅          |
| rename           | ✅      | ✅   | ✅       | ✅             | ❌     | ✅             | ✅          |
| rename_methods   | ❌      | ❌   | ❌       | ❌             | ✅     | ❌             | ❌          |
| rename_constants | ❌      | ❌   | ❌       | ❌             | ✅     | ❌             | ❌          |
| flags            | ❌      | ❌   | ✅       | ✅             | ❌     | ❌             | ❌          |
| prop             | ❌      | ❌   | ❌       | ✅             | ❌     | ❌             | ❌          |
| extends          | ❌      | ❌   | ✅       | ❌             | ❌     | ❌             | ❌          |
| implements       | ❌      | ❌   | ✅       | ❌             | ❌     | ❌             | ❌          |
| modifier         | ❌      | ❌   | ✅       | ❌             | ❌     | ❌             | ❌          |
| defaults         | ❌      | ✅   | ❌       | ❌             | ❌     | ❌             | ✅          |
| optional         | ❌      | ✅   | ❌       | ❌             | ❌     | ❌             | ✅          |
| vis              | ❌      | ✅   | ❌       | ❌             | ❌     | ❌             | ✅          |
| getter           | ❌      | ❌   | ❌       | ❌             | ❌     | ❌             | ✅          |
| setter           | ❌      | ❌   | ❌       | ❌             | ❌     | ❌             | ✅          |
| constructor      | ❌      | ❌   | ❌       | ❌             | ❌     | ❌             | ✅          |
| abstract_method  | ❌      | ❌   | ❌       | ❌             | ❌     | ❌             | ✅          |

## `name` and `rename`

`name` and `rename` are mutually exclusive. The `name` attribute is used to set the name of
an item to a string literal. The `rename` attribute is used to change the case of the name.

```rs
#[php(name = "NEW_NAME")]
#[php(rename = snake_case)]]
```

Available cases are:
- `snake_case`
- `PascalCase`
- `camelCase`
- `UPPER_CASE`
- `none` - No change
