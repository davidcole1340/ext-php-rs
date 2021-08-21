# `#[php_function]`

Used to annotate functions which should be exported to PHP. Note that this
should not be used on class methods - see the `#[php_impl]` macro for that.

See the [list of types](../types/index.md) that are valid as parameter and
return types.

## Optional parameters

Optional parameters can be used by setting the Rust parameter type to
`Option<T>` and then passing the name of the first optional parameter into the
macro options. Note that all parameters after the given parameter will be
optional as well, and therefore must be of the type `Option<T>`.

```rust
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
#[php_function(optional = "age")]
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
#[php_function(optional = "offset", defaults(offset = 0))]
pub fn rusty_strpos(haystack: &str, needle: &str, offset: i64) -> Option<usize> {
    let haystack: String = haystack.chars().skip(offset as usize).collect();
    haystack.find(needle)
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
