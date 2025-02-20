# Exceptions

Exceptions can be thrown from Rust to PHP. The inverse (catching a PHP exception
in Rust) is currently being worked on.

## Throwing exceptions

[`PhpException`] is the type that represents an exception. It contains the
message contained in the exception, the type of exception and a status code to
go along with the exception.

You can create a new exception with the `new()`, `default()`, or
`from_class::<T>()` methods. `Into<PhpException>` is implemented for `String`
and `&str`, which creates an exception of the type `Exception` with a code of 0.
It may be useful to implement `Into<PhpException>` for your error type.

Calling the `throw()` method on a `PhpException` attempts to throw the exception
in PHP. This function can fail if the type of exception is invalid (i.e. does
not implement `Exception` or `Throwable`). Upon success, nothing will be
returned.

`IntoZval` is also implemented for `Result<T, E>`, where `T: IntoZval` and
`E: Into<PhpException>`. If the result contains the error variant, the exception
is thrown. This allows you to return a result from a PHP function annotated with
the `#[php_function]` attribute.

### Examples

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
use ext_php_rs::prelude::*;

#[php_module]
mod module {
    use std::convert::TryInto;

    // Trivial example - PHP represents all integers as `u64` on 64-bit systems
    // so the `u32` would be converted back to `u64`, but that's okay for an example.
    #[php_function]
    pub fn something_fallible(n: u64) -> PhpResult<u32> {
        let n: u32 = n.try_into().map_err(|_| "Could not convert into u32")?;
        Ok(n)
    }
}
# fn main() {}
```

[`PhpException`]: https://docs.rs/ext-php-rs/0.5.0/ext_php_rs/php/exceptions/struct.PhpException.html
