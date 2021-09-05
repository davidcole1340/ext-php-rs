# `#[php_impl]`

You can export an entire `impl` block to PHP. This exports all methods as well
as constants to PHP on the class that it is implemented on. This requires the
`#[php_class]` macro to already be used on the underlying struct. Trait
implementations cannot be exported to PHP.

If you do not want a function exported to PHP, you should place it in a seperate
`impl` block.

## Methods

Methods basically follow the same rules as functions, so read about the
[`php_function`] macro first. The primary difference between functions and
methods is they are bounded by their class object.

Class methods can take a `&self` or `&mut self` parameter. They cannot take a
consuming `self` parameter. Static methods can omit this `self` parameter.

As there is no attribute directly on the method, options are passed as separate
attributes:

- `#[defaults(i = 5, b = "hello")]` - Sets the default value for parameter(s).
- `#[optional(i)]` - Sets the first optional parameter. Note that this also sets
  the remaining parameters as optional, so all optional parameters must be a
  variant of `Option<T>`.
- `#[public]`, `#[protected]` and `#[private]` - Sets the visibility of the
  method.

The `#[defaults]` and `#[optional]` attributes operate the same as the
equivalent function attribute parameters.

## Constants

Constants are defined as regular Rust `impl` constants. Any type that implements
`IntoZval` can be used as a constant. Constant visibility is not supported at
the moment, and therefore no attributes are valid on constants.

## Example

Continuing on from our `Human` example in the structs section, we will define a
constructor, as well as getters for the properties. We will also define a
constant for the maximum age of a `Human`.

```rust
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
# #[php_class]
# #[derive(Default)]
# pub struct Human {
#     name: String,
#     age: i32
# }
#[php_impl]
impl Human {
    const MAX_AGE: i32 = 100;

    pub fn __construct(&mut self, name: String, age: i32) {
        self.name = name;
        self.age = age;
    }

    pub fn get_name(&self) -> String {
        self.name.to_string()
    }

    pub fn get_age(&self) -> i32 {
        self.age
    }

    pub fn introduce(&self) {
        use ext_php_rs::php::types::object::RegisteredClass;
        
        // SAFETY: The `Human` struct is only constructed from PHP.
        let address: String = unsafe { self.get_property("address") }.unwrap();
        println!("My name is {} and I am {} years old. I live at {}.", self.name, self.age, address);
    }

    pub fn get_max_age() -> i32 {
        Self::MAX_AGE
    }
}
```

Using our newly created class in PHP:

```php
<?php

$me = new Human('David', 20);

$me->introduce(); // My name is David and I am 20 years old.
var_dump(Human::get_max_age()); // int(100)
var_dump(Human::MAX_AGE); // int(100)
```

[`php_function`]: ./function.md
