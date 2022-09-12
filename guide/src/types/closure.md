# Closure

Rust closures can be passed to PHP through a wrapper class `PhpClosure`. The
Rust closure must be static (i.e. can only reference things with a `'static`
lifetime, so not `self` in methods), and can take up to 8 parameters, all of
which must implement `FromZval`. The return type must implement `IntoZval`.

Passing closures from Rust to PHP is feature-gated behind the `closure` feature.
Enable it in your `Cargo.toml`:

```toml
ext-php-rs = { version = "...", features = ["closure"] }
```

PHP callables (which includes closures) can be passed to Rust through the
`Callable` type. When calling a callable, you must provide it with a `Vec` of
arguments, all of which must implement `IntoZval` and `Clone`.

| `T` parameter | `&T` parameter | `T` Return type                        | `&T` Return type | PHP representation                                                                         |
| ------------- | -------------- | -------------------------------------- | ---------------- | ------------------------------------------------------------------------------------------ |
| `Callable`    | No             | `Closure`, `Callable`for PHP functions | No               | Callables are implemented in PHP, closures are represented as an instance of `PhpClosure`. |

Internally, when you enable the `closure` feature, a class `PhpClosure` is
registered alongside your other classes:

```php
<?php

class PhpClosure
{
    public function __invoke(..$args): mixed;
}
```

This class cannot be instantiated from PHP. When the class is invoked, the
underlying Rust closure is called. There are three types of closures in Rust:

## `Fn` and `FnMut`

These closures can be called multiple times. `FnMut` differs from `Fn` in the
fact that it can modify variables in its scope.

### Example

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
use ext_php_rs::prelude::*;

#[php_function]
pub fn closure_get_string() -> Closure {
    // Return a closure which takes two integers and returns a string
    Closure::wrap(Box::new(|a, b| {
        format!("A: {} B: {}", a, b)
    }) as Box<dyn Fn(i32, i32) -> String>)
}

#[php_function]
pub fn closure_count() -> Closure {
    let mut count = 0i32;

    // Return a closure which takes an integer, adds it to a persistent integer,
    // and returns the updated value.
    Closure::wrap(Box::new(move |a: i32| {
        count += a;
        count
    }) as Box<dyn FnMut(i32) -> i32>)
}
# fn main() {}
```

## `FnOnce`

Closures that implement `FnOnce` can only be called once. They consume some sort
of value. Calling these closures more than once will cause them to throw an
exception. They must be wrapped using the `wrap_once` function instead of
`wrap`.

Internally, the `FnOnce` closure is wrapped again by an `FnMut` closure, which
owns the `FnOnce` closure until it is called. If the `FnMut` closure is called
again, the `FnOnce` closure would have already been consumed, and an exception
will be thrown.

### Example

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
use ext_php_rs::prelude::*;

#[php_function]
pub fn closure_return_string() -> Closure {
    let example: String = "Hello, world!".into();

    // This closure consumes `example` and therefore cannot be called more than once.
    Closure::wrap_once(Box::new(move || {
        example
    }) as Box<dyn FnOnce() -> String>)
}
# fn main() {}
```

Closures must be boxed as PHP classes cannot support generics, therefore trait
objects must be used. These must be boxed to have a compile time size.

## `Callable`

Callables are simply represented as zvals. You can attempt to get a callable
function by its name, or as a parameter. They can be called through the
`try_call` method implemented on `Callable`, which returns a zval in a result.

### Callable parameter

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
use ext_php_rs::prelude::*;

#[php_function]
pub fn callable_parameter(call: ZendCallable) {
    let val = call.try_call(vec![&0, &1, &"Hello"]).expect("Failed to call function");
    dbg!(val);
}
# fn main() {}
```
