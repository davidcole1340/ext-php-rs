# `#[php_enum]` Attribute

Enums can be exported to PHP as enums with the `#[php_enum]` attribute macro.
This attribute derives the `RegisteredClass` and `PhpEnum` traits on your enum.
To register the enum use the `r#enum::<EnumName>()` method on the `ModuleBuilder`
in the `#[php_module]` macro.

## Options

tbd

## Restrictions

tbd

## Example

This example creates a PHP enum `Suit`.

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
use ext_php_rs::prelude::*;

#[php_enum]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module.r#enum::<Suit>()
}
# fn main() {}
```

TODO: Add backed enums example
