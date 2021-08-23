# `#[php_function]`

Used to annotate functions which should be exported to PHP. Note that this
should not be used on class methods - see the `#[php_impl]` macro for that.

See the [list of types](../types/index.md) that are valid as parameter and
return types.

## Optional parameters

Optional parameters can be used by setting the Rust parameter type to a variant
of `Option<T>`. The macro will then figure out which parameters are optional by
using the last consecutive arguments that are a variant of `Option<T>` or have a
default value.

```rust
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
#[php_function]
pub fn greet(name: String, age: Option<i32>) -> String {
    let mut greeting = format!("Hello, {}!", name);

    if let Some(age) = age {
        greeting += &format!(" You are {} years old.", age);
    }

    greeting
}
```

Default parameter values can also be set for optional parameters. This is done
through the `defaults` attribute option. When an optional parameter has a
default, it does not need to be a variant of `Option`:

```rust
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
#[php_function(defaults(offset = 0))]
pub fn rusty_strpos(haystack: &str, needle: &str, offset: i64) -> Option<usize> {
    let haystack: String = haystack.chars().skip(offset as usize).collect();
    haystack.find(needle)
}
```

Note that if there is a non-optional argument after an argument that is a
variant of `Option<T>`, the `Option<T>` argument will be deemed a nullable
argument rather than an optional argument.

```rust
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
/// `age` will be deemed required and nullable rather than optional.
#[php_function]
pub fn greet(name: String, age: Option<i32>, description: String) -> String {
    let mut greeting = format!("Hello, {}!", name);

    if let Some(age) = age {
        greeting += &format!(" You are {} years old.", age);
    }

    greeting += &format!(" {}.", description);
    greeting
}
```

You can also specify the optional arguments if you want to have nullable
arguments before optional arguments. This is done through an attribute
parameter:

```rust
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
/// `age` will be deemed required and nullable rather than optional,
/// while description will be optional.
#[php_function(optional = "description")]
pub fn greet(name: String, age: Option<i32>, description: Option<String>) -> String {
    let mut greeting = format!("Hello, {}!", name);

    if let Some(age) = age {
        greeting += &format!(" You are {} years old.", age);
    }

    if let Some(description) = description {
        greeting += &format!(" {}.", description);
    }

    greeting
}
```

## Throwing exceptions

Exceptions can be thrown from inside a function which returns a `Result<T, E>`,
where `E` implements `Into<PhpException>`. The `PhpException` class allows you
to customise the type of exception thrown, along with the exception code and
message.

By default, `String` and `&str` are both implemented with `Into<PhpException>`,
and in both cases a regular `Exception` is thrown.

```rust
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
#[php_function]
pub fn example_exception() -> Result<i64, &'static str> {
    Err("Bad!!!")
}
```
