# Changelog

## Version 1.0.0

- `Zval::reference()` returns a reference instead of a dereferenced pointer.
- Added `ZendHashTable::iter()` - note this is changing in a future version.
- `ClassBuilder::extends()` now takes a reference rather than a pointer to match the
return type of `ClassEntry::exception()`.
- `ClassEntry::build()` now returns a reference - same reason as above.
- Improve library 'safety' by removing `unwrap` calls:
    - `.build()` returns `Result` on `FunctionBuilder`, `ClassBuilder` and `ModuleBuilder`.
    - `.property()` and `.constant()` return `Result` on `ClassBuilder`.
    - `.register_constant()` returns `Result`.
    - `.try_call()` on callables now return `Result` rather than `Option`.
    - `throw()` and `throw_with_code()` now returns `Result`.
    - `new()` and `new_interned()` on `ZendString` now returns a `Result`.
    - For `ZendHashTable`:
        - `insert()`, `insert_at_index()` now returns a `Result<HashTableInsertResult>`, where `Err` failed, 
        `Ok(Ok)` inserts successfully without overwrite, and `Ok(OkWithOverwrite(&Zval))` inserts successfully
        with overwrite.
        - `push()` now returns a `Result`.
        - Converting from a `Vec` or `HashMap` to a `ZendHashTable` is fallible, so it now implementes `TryFrom` as
        opposed to `From`.
    - For `Zval`:
        - `set_string()` now returns a `Result`, and takes a second parameter (persistent).
        - `set_persistent_string()` has now been removed in favour of `set_string()`.
        - `set_interned_string()` also returns a `Result`.
        - `set_array()` now only takes a `ZendHashTable`, you must convert your `Vec` or `HashMap`
        by calling `try_into()` and handling the error.

## Version 0.0.7

- Added support for thread-safe PHP (@davidcole1340) #37
- Added ability to add properties to classes (@davidcole1340) #39
- Added better interactions with objects (@davidcole1340) #41

## Version 0.0.6

- Fixed `panic!` when a PHP binary string was given to a function (@davidcole1340) [c:d73788e]
- Fixed memory leak when returning an array from Rust to PHP (@davidcole1340) #34
- Documentation is now deployed to [GitHub Pages](https://davidcol1340.github.io/ext-php-rs) (@davidcole1340) #35
- Added ability to unpack and pack binary strings similar to PHP (@davidcole1340) #32
- Allowed `default-features` to be true for Bindgen (@willbrowningme) #36

## Version 0.0.5

- Relicensed project under MIT or Apache 2.0 as per Rust crate guidelines (@davidcole1340) [c:439f2ae]
- Added `parse_args!` macro to simplify argument parsing (@davidcole1340) [c:45c7242]
- Added ability to throw exceptions from Rust to PHP (@davidcole1340) [c:45c7242]
- Added ability to register global constants (@davidcole1340) [c:472e26e]
- Implemented `From<ZendHashTable>` for `Vec` (@davidcole1340) [c:3917c41]
- Expanded implementations for converting to `Zval` from primitives (@davidcole1340) [c:d4c6aa2]
- Replaced unit errors with an `Error` enum (@davidcole1340) [c:f11451f]
- Added `Debug` and `Clone` implementations for most structs (@davidcole1340) [c:62a43e6]
