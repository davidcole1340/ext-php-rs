# Introduction

`ext-php-rs` is a Rust library containing bindings and abstractions for the PHP
extension API, which allows users to build extensions natively in Rust.

## Features

- **Easy to use:** The built-in macros can abstract away the need to interact
  with the Zend API, such as Rust-type function parameter abstracting away
  interacting with Zend values.
- **Lightweight:** You don't have to use the built-in helper macros. It's
  possible to write your own glue code around your own functions.
- **Extensible:** Implement `IntoZval` and `FromZval` for your own custom types,
  allowing the type to be used as function parameters and return types.

## Goals

Our main goal is to **make extension development easier.**

- Writing extensions in C can be tedious, and with the Zend APIs limited
  documentation can be intimidating.
- Rust's modern language features and feature-full standard library are big
  improvements on C.
- Abstracting away the raw Zend APIs allows extensions to be developed faster
  and with more confidence.
- Abstractions also allow us to support future (and potentially past) versions
  of PHP without significant changes to extension code.

## Versioning

`ext-php-rs` follows semantic versioning, however, no backwards compatibility is
guaranteed while we are at major version `0`, which is for the forseeable
future. It's recommended to lock the version at the patch level.

## Documentation

- This guide!
- [Rust docs](https://davidcole1340.github.io/ext-php-rs)
