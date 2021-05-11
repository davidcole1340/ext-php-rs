# Changelog

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
