# Changelog

## [0.11.0](https://github.com/davidcole1340/ext-php-rs/compare/ext-php-rs-derive-v0.10.2...ext-php-rs-derive-v0.11.0) - 2025-05-10

### BREAKING CHANGES

- *(class)* [**breaking**] Generate correct stubs for extends and implements (by @Xenira) [[#326](https://github.com/davidcole1340/ext-php-rs/issues/326)] 
> `extends` and `implements` attributes now require the `stub` property containing the class/interface name to be used in stubs.
- *(macro)* [**breaking**] Uinify attributes in `#[php]` attribute (by @Xenira) [[#391](https://github.com/davidcole1340/ext-php-rs/issues/391)] 
> Attributes like `#[prop]`, `#[rename]`, etc. have been moved to `#[php]` attributes like `#[php(prop)]`,  have been moved to `#[php]` attributes like `#[php(prop)]`, `#[php(name = "Foo")]`, `#[php(rename = CamelCase)]`, etc.
- *(macro)* [**breaking**] Switch to builder pattern (by @Xenira) [[#99](https://github.com/davidcole1340/ext-php-rs/issues/99)] [[#131](https://github.com/davidcole1340/ext-php-rs/issues/131)] [[#327](https://github.com/davidcole1340/ext-php-rs/issues/327)] [[#174](https://github.com/davidcole1340/ext-php-rs/issues/174)] [[#335](https://github.com/davidcole1340/ext-php-rs/issues/335)] 
> The old macros were dependent on execution order and have been causing trouble with language servers. They are replaced by a builder. See the migration guide at https://davidcole1340.github.io/ext-php-rs/migration-guides/v0.14.html for information on how to migrate.

### Fixed
- *(args)* Fix variadic args (by @Xenira) [[#337](https://github.com/davidcole1340/ext-php-rs/issues/337)] 
- *(macro)* Add missing static flags in `php_impl` macro (by @Norbytus) [[#419](https://github.com/davidcole1340/ext-php-rs/issues/419)] 
- *(macro)* Add missing separator pipe in flags (by @Norbytus) [[#412](https://github.com/davidcole1340/ext-php-rs/issues/412)] 

### Other
- *(cargo-php)* Add locked option to install guide ([#370](https://github.com/davidcole1340/ext-php-rs/pull/370)) (by @Xenira) [[#370](https://github.com/davidcole1340/ext-php-rs/issues/370)] [[#314](https://github.com/davidcole1340/ext-php-rs/issues/314)] 
- *(clippy)* Apply pedantic rules (by @Xenira) [[#418](https://github.com/davidcole1340/ext-php-rs/issues/418)] 
- *(coverage)* Add coverage badge (by @Xenira)
- *(deps)* Update syn and darling ([#400](https://github.com/davidcole1340/ext-php-rs/pull/400)) (by @Xenira) [[#400](https://github.com/davidcole1340/ext-php-rs/issues/400)] 
- *(guide)* Directly include doc comments (by @Xenira)
- *(macro)* Use `#[php]` attribute for startup function (by @Xenira) [[#423](https://github.com/davidcole1340/ext-php-rs/issues/423)] 
- *(macro)* Trait rename for general and method names (by @Norbytus) [[#420](https://github.com/davidcole1340/ext-php-rs/issues/420)] 
- *(macro)* Update documentation for builder pattern (by @Xenira)
- *(macro)* Add stubs for new builder pattern (by @Xenira) [[#183](https://github.com/davidcole1340/ext-php-rs/issues/183)] 

## [0.10.2](https://github.com/davidcole1340/ext-php-rs/compare/ext-php-rs-derive-v0.10.1...ext-php-rs-derive-v0.10.2) - 2025-02-06

### Other
- Typo when error for #[defaults] macro (by @yoramdelangen)
- Don't use symbolic links for git. (by @faassen)
- Fix pipeline (#320) (by @Xenira) [[#320](https://github.com/davidcole1340/ext-php-rs/issues/320)] 
- Support for variadic functions (by @joehoyle)