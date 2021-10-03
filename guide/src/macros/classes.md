# Classes

Structs can be exported to PHP as classes with the `#[php_class]` attribute
macro. This attribute derives the `RegisteredClass` trait on your struct, as
well as registering the class to be registered with the `#[php_module]` macro.

The implementation of `RegisteredClass` requires the implementation of `Default`
on the struct. This is because the struct is initialized before the constructor
is called, therefore it must have default values for all properties.

Note that Rust struct properties **are not** PHP properties, so if you want the
user to be able to access these, you must provide getters and/or setters.
Properties are supported internally, however, they are not usable through the
automatic macros. Support for properties is planned.

## Options

The attribute takes some options to modify the output of the class:

- `name` - Changes the name of the class when exported to PHP. The Rust struct
  name is kept the same. If no name is given, the name of the struct is used.
  Useful for namespacing classes.

There are also additional macros that modify the class. These macros **must** be
placed underneath the `#[php_class]` attribute.

- `#[extends(ce)]` - Sets the parent class of the class. Can only be used once.
  `ce` must be a valid Rust expression when it is called inside the
  `#[php_module]` function.
- `#[implements(ce)]` - Implements the given interface on the class. Can be used
  multiple times. `ce` must be a valid Rust expression when it is called inside
  the `#[php_module]` function.

You may also use the `#[prop]` attribute on a struct field to use the field as a
PHP property. By default, the field will be accessible from PHP publically with
the same name as the field. You can rename the property with options:

- `rename` - Allows you to rename the property, e.g.
  `#[prop(rename = "new_name")]`

## Example

This example creates a PHP class `Human`, adding a PHP property `address` with
an empty string as the default value.

```rust
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
#[php_class]
#[derive(Default)]
pub struct Human {
    name: String,
    age: i32,
    #[prop]
    address: String,
}
# #[php_module]
# pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
#     module
# }
```

Create a custom exception `RedisException`, which extends `Exception`, and put
it in the `Redis\Exception` namespace:

```rust
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
use ext_php_rs::php::{class::ClassEntry, exceptions::PhpException};

#[php_class(name = "Redis\\Exception\\RedisException")]
#[extends(ClassEntry::exception())]
#[derive(Default)]
pub struct RedisException;

// Throw our newly created exception
#[php_function]
pub fn throw_exception() -> PhpResult<i32> {
    Err(PhpException::from_class::<RedisException>("Not good!".into()))
}
# #[php_module]
# pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
#     module
# }
```
