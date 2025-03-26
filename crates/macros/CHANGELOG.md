# Changelog

## [0.11.0](https://github.com/davidcole1340/ext-php-rs/compare/ext-php-rs-derive-v0.10.2...ext-php-rs-derive-v0.11.0) - 2025-03-26

### BREAKING CHANGES

- *(macro)* [**breaking**] Switch to builder pattern (by @Xenira) [[#99](https://github.com/davidcole1340/ext-php-rs/issues/99)] [[#131](https://github.com/davidcole1340/ext-php-rs/issues/131)] [[#327](https://github.com/davidcole1340/ext-php-rs/issues/327)] [[#174](https://github.com/davidcole1340/ext-php-rs/issues/174)] [[#335](https://github.com/davidcole1340/ext-php-rs/issues/335)] 
> The old macros were dependent on execution order and have been causing trouble with language servers. They are replaced by a builder. See the migration guide at https://davidcole1340.github.io/ext-php-rs/migration-guides/v0.14.html for information on how to migrate.

### Fixed
- *(args)* Fix variadic args (by @Xenira) [[#337](https://github.com/davidcole1340/ext-php-rs/issues/337)] 

### Other
- *(deps)* Update syn and darling ([#400](https://github.com/davidcole1340/ext-php-rs/pull/400)) (by @Xenira) [[#400](https://github.com/davidcole1340/ext-php-rs/issues/400)] 
- *(guide)* Directly include doc comments (by @Xenira)
- *(macro)* Update documentation for builder pattern (by @Xenira)
- *(macro)* Add stubs for new builder pattern (by @Xenira) [[#183](https://github.com/davidcole1340/ext-php-rs/issues/183)] 

## [0.10.2](https://github.com/davidcole1340/ext-php-rs/compare/ext-php-rs-derive-v0.10.1...ext-php-rs-derive-v0.10.2) - 2025-02-06

### Other
- Typo when error for #[defaults] macro (by @yoramdelangen)
- Don't use symbolic links for git. (by @faassen)
- Fix pipeline (#320) (by @Xenira) [[#320](https://github.com/davidcole1340/ext-php-rs/issues/320)] 
- Support for variadic functions (by @joehoyle)