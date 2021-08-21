# Types

In PHP, data is stored in containers called zvals (zend values). Internally,
these are effectively tagged unions (enums in Rust) without the safety that Rust
introduces. Passing data between Rust and PHP requires the data to become a
zval. This is done through two traits: `FromZval` and `IntoZval`. These traits
have been implemented on most regular Rust types:

- Primitive integers (`i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`,
  `usize`, `isize`).
- Double and single-precision floating point numbers (`f32`, `f64`).
- Booleans.
- Strings (`String` and `&str`)
- `Vec<T>` where T implements `IntoZval` and/or `FromZval`.
- `HashMap<String, T>` where T implements `IntoZval` and/or `FromZval`.
- `Binary<T>` where T implements `Pack`, used for transferring binary string
  data.
- `Option<T>` where T implements `IntoZval` and/or `FromZval`, and where `None`
  is converted to a PHP `null`.

There is one special case - `Result<T, E>`, where T implements `IntoZval` and
`E` implements `Into<PhpException>`. This can only be used as a function/method
return type. If the error variant is encountered, `E` is converted into a
`PhpException` and thrown.
