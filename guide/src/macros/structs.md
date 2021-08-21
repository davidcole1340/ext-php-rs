# Structs

Structs can be exported to PHP as classes with the
`#[derive(ZendObjectHandler)]` macro.

The implementation of `ZendObjectOverride` requires the implementation of
`Default` on the struct. This is because the struct is initialized before the
constructor is called, therefore it must have default values for all properties.

This derive macro is likely to be changed to an attribute macro over time, as to
introduce class renaming as well as inheritance, which can't be done with a
derive macro.

This macro can also be used on enums.

Note that Rust struct properties **are not** PHP properties, so if you want the
user to be able to access these, you must provide getters and/or setters.
Properties are supported, however, they are not usable through the automatic
macros.

## Example

This example creates a PHP class `Human`:

```rust
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
#[derive(Default, ZendObjectHandler)]
pub struct Human {
    name: String,
    age: i32
}
```
