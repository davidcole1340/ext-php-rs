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

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
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
# fn main() {}
```

Default parameter values can also be set for optional parameters. This is done
through the `defaults` attribute option. When an optional parameter has a
default, it does not need to be a variant of `Option`:

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
#[php_function(defaults(offset = 0))]
pub fn rusty_strpos(haystack: &str, needle: &str, offset: i64) -> Option<usize> {
    let haystack: String = haystack.chars().skip(offset as usize).collect();
    haystack.find(needle)
}
# fn main() {}
```

Note that if there is a non-optional argument after an argument that is a
variant of `Option<T>`, the `Option<T>` argument will be deemed a nullable
argument rather than an optional argument.

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
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
# fn main() {}
```

You can also specify the optional arguments if you want to have nullable
arguments before optional arguments. This is done through an attribute
parameter:

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
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
# fn main() {}
```

## Variadic Functions

Variadic functions can be implemented by specifying the last argument in the Rust
function to the type `&[&Zval]`. This is the equivalent of a PHP function using
the `...$args` syntax.

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
# use ext_php_rs::types::Zval;
/// This can be called from PHP as `add(1, 2, 3, 4, 5)`
/// note: it requires to set numbers with one or arguments
#[php_function]
pub fn add(number: u32, numbers:&[&Zval]) -> u32 {
    println!("Extra numbers: {:?}", numbers);
    // numbers is a slice of 4 Zvals all of type long
    number
}

/// Having optional numbers can be done like:
/// This can be called from PHP as `add(1)`, with no addional numbers given
#[php_function(optional = "numbers")]
pub fn add_optional(number: u32, numbers:&[&Zval]) -> u32 {
    println!("Optional numbers: {:?}", numbers);
    // numbers is a slice of 4 Zvals all of type long
    number
}
# fn main() {}
```

Checkout more example in our [tests](https://github.com/davidcole1340/ext-php-rs/tree/master/tests/src/integration/variadic_args.php) location.

## Returning `Result<T, E>`

You can also return a `Result` from the function. The error variant will be
translated into an exception and thrown. See the section on
[exceptions](../exceptions.md) for more details.
