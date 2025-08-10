# `#[php_interface]` Attribute

You can export an entire `Trait` block to PHP. This exports all methods as well
as constants to PHP on the interface. Trait method SHOULD NOT contain default implementation

## Options

By default all constants are renamed to `UPPER_CASE` and all methods are renamed to
camelCase. This can be changed by passing the `change_method_case` and
`change_constant_case` as `#[php]` attributes on the `impl` block. The options are:

- `#[php(change_method_case = "snake_case")]` - Renames the method to snake case.
- `#[php(change_constant_case = "snake_case")]` - Renames the constant to snake case.

See the [`name` and `change_case`](./php.md#name-and-change_case) section for a list of all
available cases.

## Methods

See the [php_impl](./impl.md#)

## Constants

See the [php_impl](./impl.md#)

## Example

Define trait example with few methods and constant, and try implement this interface
in php

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
use ext_php_rs::{prelude::*, types::ZendClassObject};


#[php_interface]
#[php(name = "Rust\\TestInterface")]
trait Test {
    const TEST: &'static str = "TEST";

    fn co();

    #[php(defaults(value = 0))]
    fn set_value(&mut self, value: i32);
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .interface::<PhpInterfaceTest>()
}

# fn main() {}
```

Using our newly created interface in PHP:

```php
<?php

assert(interface_exists("Rust\TestInterface"));

class B implements Rust\TestInterface {

    public static function co() {}

    public function setValue(?int $value = 0) {

    }
}

```
