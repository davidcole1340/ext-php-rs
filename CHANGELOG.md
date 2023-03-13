# Changelog

## 0.10.0
- feat: Add PHP 8.2 support by @ptondereau [#212]

[#212]: https://github.com/davidcole1340/ext-php-rs/pull/212

## Version 0.9.0

- ci+docs: honour PHP_CONFIG & rebuild automatically when env vars change by @julius [#210]
- chore: Update generated FFI bindings with bindgen 0.63 by @ptondereau [#211]

**BC changes**
- feat: allows ZendStr to contain null bytes by @julius [#202]

**Migration**
See: [#202]

[#202]: https://github.com/davidcole1340/ext-php-rs/pull/202
[#210]: https://github.com/davidcole1340/ext-php-rs/pull/210
[#211]: https://github.com/davidcole1340/ext-php-rs/pull/211


## Version 0.8.3

- build: Check docs warnings in CI by @davidcole1340 in [#180]
- fix: Fixes infinite loop in ClassEntry::instance_of() by @ju1ius in [#188]
- fix: Fix binary slice lifetimes by @davidcole1340 in [#181]
- build: Fixes CI workflow configuration by @ju1ius in [#195]
- feat: Add get_id() and hash() methods on ZendObject by @ju1ius in [#196]
- docs: Describes restrictions on generic parameters for `php_class` by @ju1ius in [#194]
- feat: Add instance_of() and get_class_entry() methods on ZendObject by @ju1ius in [#197]

[#180]: https://github.com/davidcole1340/ext-php-rs/pull/180
[#188]: https://github.com/davidcole1340/ext-php-rs/pull/188
[#181]: https://github.com/davidcole1340/ext-php-rs/pull/181
[#195]: https://github.com/davidcole1340/ext-php-rs/pull/195
[#196]: https://github.com/davidcole1340/ext-php-rs/pull/196
[#194]: https://github.com/davidcole1340/ext-php-rs/pull/194
[#197]: https://github.com/davidcole1340/ext-php-rs/pull/197

## Version 0.8.2

- Update changelog for latest versions by @striezel in [#161]
- fix building docs on docs.rs by @davidcole1340 in [#165]
- Add some standard zend interfaces by @nikeee in [#164]
- Correct parameter name. by @denzyldick in [#168]
- fix describe when using `#[implements]` by @davidcole1340 in [#169]
- Add example that shows how to implement an interface by @nikeee in [#167]
- add `before` flag to `#[php_startup]` by @davidcole1340 in [#170]
- add ability to define abstract methods by @davidcole1340 in [#171]
- chore(cli): Bump Clap for CLI tool by @ptondereau in [#177]
- fix type links in docs.rs by @davidcole1340 in [#179]

[#161]: https://github.com/davidcole1340/ext-php-rs/pull/161
[#165]: https://github.com/davidcole1340/ext-php-rs/pull/165
[#164]: https://github.com/davidcole1340/ext-php-rs/pull/164
[#168]: https://github.com/davidcole1340/ext-php-rs/pull/168
[#169]: https://github.com/davidcole1340/ext-php-rs/pull/169
[#167]: https://github.com/davidcole1340/ext-php-rs/pull/167
[#170]: https://github.com/davidcole1340/ext-php-rs/pull/170
[#171]: https://github.com/davidcole1340/ext-php-rs/pull/171
[#177]: https://github.com/davidcole1340/ext-php-rs/pull/177
[#179]: https://github.com/davidcole1340/ext-php-rs/pull/179

## Version 0.8.1

- 404 /guide doesn't exists. by @denzyldick in [#149]
- Fixed some typos by @denzyldick in [#148]
- Fix a few typos by @striezel in [#150]
- fix causes of some clippy warnings by @striezel in [#152]
- fix more causes of clippy warnings by @striezel in [#157]
- attempt to fix errors related to clap by @striezel in [#158]
- ci: run clippy only on stable Rust channel by @striezel in [#159]
- update actions/checkout in GitHub Actions workflows to v3 by @striezel in
  [#151]
- Add ability to set function name on php_function macro by @joehoyle in [#153]
- Specify classes as fully-qualified names in stubs by @joehoyle in [#156]
- Support marking classes as interfaces by @joehoyle in [#155]
- Support marking methods as abstract by @joehoyle in [#154]
- Add php-scrypt as a example project by @PineappleIOnic in [#146]
- Fix ini file duplication and truncation when using cargo-php command by
  @roborourke in [#136]
- Allow passing --yes parameter to bypass prompts by @roborourke in [#135]

[#135]: https://github.com/davidcole1340/ext-php-rs/pull/135
[#136]: https://github.com/davidcole1340/ext-php-rs/pull/136
[#146]: https://github.com/davidcole1340/ext-php-rs/pull/146
[#148]: https://github.com/davidcole1340/ext-php-rs/pull/148
[#149]: https://github.com/davidcole1340/ext-php-rs/pull/149
[#150]: https://github.com/davidcole1340/ext-php-rs/pull/150
[#151]: https://github.com/davidcole1340/ext-php-rs/pull/151
[#152]: https://github.com/davidcole1340/ext-php-rs/pull/152
[#153]: https://github.com/davidcole1340/ext-php-rs/pull/153
[#154]: https://github.com/davidcole1340/ext-php-rs/pull/154
[#155]: https://github.com/davidcole1340/ext-php-rs/pull/155
[#156]: https://github.com/davidcole1340/ext-php-rs/pull/156
[#157]: https://github.com/davidcole1340/ext-php-rs/pull/157
[#158]: https://github.com/davidcole1340/ext-php-rs/pull/158
[#159]: https://github.com/davidcole1340/ext-php-rs/pull/159

## Version 0.8.0

- Windows support by @davidcole1340 in [#128]
- Support for binary slice to avoid extra allocation by @TobiasBengtsson in
  [#139]
- Bump dependencies by @ptondereau in [#144]

[#128]: https://github.com/davidcole1340/ext-php-rs/pull/128
[#139]: https://github.com/davidcole1340/ext-php-rs/pull/139
[#144]: https://github.com/davidcole1340/ext-php-rs/pull/144

## Version 0.7.4

- Fix is_true() / is_false() in Zval by @joehoyle in [#116]
- readme: fix link to guide by @TorstenDittmann in [#120]
- Fix request_(startup|shutdown)_function in ModuleBuilder by @glyphpoch in
  [#119]
- Fix CI on macOS by @davidcole1340 in [#126]
- Add ability to pass modifier function for classes by @davidcole1340 in [#127]

[#116]: https://github.com/davidcole1340/ext-php-rs/pull/116
[#119]: https://github.com/davidcole1340/ext-php-rs/pull/119
[#120]: https://github.com/davidcole1340/ext-php-rs/pull/120
[#126]: https://github.com/davidcole1340/ext-php-rs/pull/126
[#127]: https://github.com/davidcole1340/ext-php-rs/pull/127

## Version 0.7.3

- Upgrade `clap` to `3.0.0-rc3`. [#113]
- Build properties hashmap once and cache inside class metadata. [#114]
- Add `impl FromZval for &Zval` and `impl FromZvalMut for &mut Zval`.
- Add `has_numerical_keys` and `has_sequential_keys` to `ZendHashTable`. [#115]

Thanks to the following contributors:

- @davidcole1340
- @vkill

[#113]: https://github.com/davidcole1340/ext-php-rs/pull/113
[#114]: https://github.com/davidcole1340/ext-php-rs/pull/114
[#115]: https://github.com/davidcole1340/ext-php-rs/pull/115

## Version 0.7.2

- Add preliminary PHP 8.1 support. [#109]
  - Extensions should now compile for PHP 8.1. This doesn't implement any of the
    new PHP 8.1 features.
- Add `anyhow` cargo feature to implement
  `From<anyhow::Error> for PhpException`. [#110]
- Made `ClassMetadata: Send + Sync`. [#111]
- Fixed registering constants with expressions. [#112]

[#109]: https://github.com/davidcole1340/ext-php-rs/pull/109
[#110]: https://github.com/davidcole1340/ext-php-rs/pull/110
[#111]: https://github.com/davidcole1340/ext-php-rs/pull/111
[#112]: https://github.com/davidcole1340/ext-php-rs/pull/112

## Version 0.7.1

- Ensure stable ABI between `cargo-php` and downstream extensions. [#108]
  - `ext-php-rs` versions used when compiling CLI and extension are now
    compared.

[#108]: https://github.com/davidcole1340/ext-php-rs/pull/108

## Version 0.7.0

- Disabled serialization and unserialization of Rust structs exported as PHP
  classes. [#105]
  - You can't serialize an associated Rust struct so this would have never
    worked, but disabling them fixes crashes when running in an environment like
    psysh.
- Replaced boxed module inside `ModuleBuilder` with in-struct module.
- Fixed builds failing on Linux AArch64 systems. [#106]
- Added `cargo-php` for creating stubs, installing and uninstalling extensions.
  [#107]
  - Check out the guide for more information on this.

[#105]: https://github.com/davidcole1340/ext-php-rs/pull/105
[#106]: https://github.com/davidcole1340/ext-php-rs/pull/106
[#107]: https://github.com/davidcole1340/ext-php-rs/pull/107

## Version 0.6.0

- Reorganized project. [#101]
  - Changed (almost all) module paths. Too many changes to list them all, check
    out the docs.
  - Removed `skel` project.
- Allow methods to accept references to `ZendClassObject<T>` instead of `self`.
  [#103]

[#101]: https://github.com/davidcole1340/ext-php-rs/pull/101
[#103]: https://github.com/davidcole1340/ext-php-rs/pull/103

## Version 0.5.3

- Fixed docs.rs PHP bindings file.

## Version 0.5.2

- Constructors that return `Self` can now be added to classes. [#83]
  - `Default` is no longer required to be implemented on classes, however, a
    constructor must be specified if you want to construct the class from PHP.
  - Constructors can return `Self` or `Result<Self, E>`, where
    `E: Into<PhpException>`.
- Added `FromZendObject` and `IntoZendObject` traits. [#74]
- Added `#[derive(ZvalConvert)]` derive macro. Derives `IntoZval` and `FromZval`
  on arbitrary structs and enums. [#78]
- Added `ZBox<T>`, similar to `Box<T>`, to allocate on the Zend heap. [#94]
- Changed execution data functions to take mutable references. [#100]
- `&mut T` is now valid as a function parameter. [#100]

Thanks to the contributors for this release:

- @davidcole1340
- @vodik

[#74]: https://github.com/davidcole1340/ext-php-rs/pull/74
[#78]: https://github.com/davidcole1340/ext-php-rs/pull/78
[#83]: https://github.com/davidcole1340/ext-php-rs/pull/83
[#94]: https://github.com/davidcole1340/ext-php-rs/pull/94
[#100]: https://github.com/davidcole1340/ext-php-rs/pull/100

## Version 0.5.1

- `PhpException` no longer requires a lifetime [#80].
- Added `PhpException` and `PhpResult` to prelude [#80].
- Fixed `ZendString` missing last character [#82].

[#80]: https://github.com/davidcole1340/ext-php-rs/pull/80
[#82]: https://github.com/davidcole1340/ext-php-rs/pull/82

## Version 0.5.0

### Breaking changes

- Method names are now renamed to snake case by default [#63].
- Refactored `ZendHashTable` into an owned and borrowed variant [#76].
  - Creating a new hashtable is done through the `OwnedHashTable` struct, which
    is then dereferenced to `&HashTable`, as `String` is to `&str`.
- Refactored `ZendString` into an owned and borrowed variant [#77].
  - Creating a new Zend string is done through the `ZendString` struct, which is
    then dereferenced to `&ZendStr`, as `String` is to `&str`.
- Class properties are now defined as struct properties, removing the old
  property system in the process [#69].

### Enhancements

- Added interfaces and parent class to the `Debug` implementation for
  `ClassEntry` [@72b0491].
- Rust unit type `()` now has a datatype of `void` [@8b3ed08].
- Functions returning Rust objects will now display their full classname in
  reflection [#64].
- Fixed alignment of class objects in memory [#66].

Thanks to the contributors for this release:

- @davidcole1340
- @vodik

[#63]: https://github.com/davidcole1340/ext-php-rs/pull/63
[#76]: https://github.com/davidcole1340/ext-php-rs/pull/76
[#77]: https://github.com/davidcole1340/ext-php-rs/pull/77
[#69]: https://github.com/davidcole1340/ext-php-rs/pull/69
[#64]: https://github.com/davidcole1340/ext-php-rs/pull/64
[#66]: https://github.com/davidcole1340/ext-php-rs/pull/66
[@72b0491]: https://github.com/davidcole1340/ext-php-rs/commit/72b0491
[@8b3ed08]: https://github.com/davidcole1340/ext-php-rs/commit/8b3ed08

## Version 0.2.0 - 0.4.0

- Added macros!
- Missed a bit :(

## Version 0.1.0

- `Zval::reference()` returns a reference instead of a dereferenced pointer.
- Added `ZendHashTable::iter()` - note this is changing in a future version.
- `ClassBuilder::extends()` now takes a reference rather than a pointer to match
  the return type of `ClassEntry::exception()`.
- `ClassEntry::build()` now returns a reference - same reason as above.
- Improve library 'safety' by removing `unwrap` calls:
  - `.build()` returns `Result` on `FunctionBuilder`, `ClassBuilder` and
    `ModuleBuilder`.
  - `.property()` and `.constant()` return `Result` on `ClassBuilder`.
  - `.register_constant()` returns `Result`.
  - `.try_call()` on callables now return `Result` rather than `Option`.
  - `throw()` and `throw_with_code()` now returns `Result`.
  - `new()` and `new_interned()` on `ZendString` now returns a `Result`.
  - For `ZendHashTable`:
    - `insert()`, `insert_at_index()` now returns a
      `Result<HashTableInsertResult>`, where `Err` failed, `Ok(Ok)` inserts
      successfully without overwrite, and `Ok(OkWithOverwrite(&Zval))` inserts
      successfully with overwrite.
    - `push()` now returns a `Result`.
    - Converting from a `Vec` or `HashMap` to a `ZendHashTable` is fallible, so
      it now implements `TryFrom` as opposed to `From`.
  - For `Zval`:
    - `set_string()` now returns a `Result`, and takes a second parameter
      (persistent).
    - `set_persistent_string()` has now been removed in favour of
      `set_string()`.
    - `set_interned_string()` also returns a `Result`.
    - `set_array()` now only takes a `ZendHashTable`, you must convert your
      `Vec` or `HashMap` by calling `try_into()` and handling the error.

## Version 0.0.7

- Added support for thread-safe PHP (@davidcole1340) #37
- Added ability to add properties to classes (@davidcole1340) #39
- Added better interactions with objects (@davidcole1340) #41

## Version 0.0.6

- Fixed `panic!` when a PHP binary string was given to a function
  (@davidcole1340) [c:d73788e]
- Fixed memory leak when returning an array from Rust to PHP (@davidcole1340)
  #34
- Documentation is now deployed to
  [GitHub Pages](https://davidcol1340.github.io/ext-php-rs) (@davidcole1340) #35
- Added ability to unpack and pack binary strings similar to PHP
  (@davidcole1340) #32
- Allowed `default-features` to be true for Bindgen (@willbrowningme) #36

## Version 0.0.5

- Relicensed project under MIT or Apache 2.0 as per Rust crate guidelines
  (@davidcole1340) [c:439f2ae]
- Added `parse_args!` macro to simplify argument parsing (@davidcole1340)
  [c:45c7242]
- Added ability to throw exceptions from Rust to PHP (@davidcole1340)
  [c:45c7242]
- Added ability to register global constants (@davidcole1340) [c:472e26e]
- Implemented `From<ZendHashTable>` for `Vec` (@davidcole1340) [c:3917c41]
- Expanded implementations for converting to `Zval` from primitives
  (@davidcole1340) [c:d4c6aa2]
- Replaced unit errors with an `Error` enum (@davidcole1340) [c:f11451f]
- Added `Debug` and `Clone` implementations for most structs (@davidcole1340)
  [c:62a43e6]
