# Changelog

## [0.11.3](https://github.com/extphprs/ext-php-rs/compare/ext-php-rs-derive-v0.11.2...ext-php-rs-derive-v0.11.3) - 2025-10-28

### Added
- *(enum)* Add basic enum support (by @Xenira, @joehoyle) [[#178](https://github.com/extphprs/ext-php-rs/issues/178)] [[#302](https://github.com/extphprs/ext-php-rs/issues/302)] 
- Add constructor visibility (by @Norbytus) [[#542](https://github.com/extphprs/ext-php-rs/issues/542)] 

### Other
- *(clippy)* Fix new clippy findings (by @Xenira) [[#543](https://github.com/extphprs/ext-php-rs/issues/543)] 
- *(deps)* Upgrade bindgen to 0.72 (by @ptondereau)
- *(deps)* Remove unused deps (by @robem)
- *(deps)* Update darling requirement from 0.20 to 0.21 (by @dependabot[bot])
- *(macro)* Add test infrastructure for macro crate (by @Xenira) [[#530](https://github.com/extphprs/ext-php-rs/issues/530)] 
- *(readme)* Update example in readme (by @joehoyle) [[#539](https://github.com/extphprs/ext-php-rs/issues/539)] 
- Update guide url and authors (by @Xenira) [[#500](https://github.com/extphprs/ext-php-rs/issues/500)] 
## [0.11.2](https://github.com/extphprs/ext-php-rs/compare/ext-php-rs-derive-v0.11.1...ext-php-rs-derive-v0.11.2) - 2025-07-13

### Fixed
- *(macro)* Allow multiple refs with `self_` (by @Xenira) [[#506](https://github.com/extphprs/ext-php-rs/issues/506)] 
- *(macro)* Fix `self_` reference when multiple method arguments supplied (by @Xenira) [[#504](https://github.com/extphprs/ext-php-rs/issues/504)] 
- *(stubs)* Include doc comments in `__construct()` stubs (by @Xenira) [[#497](https://github.com/extphprs/ext-php-rs/issues/497)] 

## [0.11.1](https://github.com/extphprs/ext-php-rs/compare/ext-php-rs-derive-v0.11.0...ext-php-rs-derive-v0.11.1) - 2025-07-06

### Other
- Add missing parenthesis (by @Stranger6667) [[#486](https://github.com/extphprs/ext-php-rs/issues/486)] 

## [0.11.0](https://github.com/extphprs/ext-php-rs/compare/ext-php-rs-derive-v0.10.2...ext-php-rs-derive-v0.11.0) - 2025-07-04

### BREAKING CHANGES

- *(macro)* [**breaking**] Change rename defaults to match psr (by @Xenira) [[#189](https://github.com/extphprs/ext-php-rs/issues/189)] [[#436](https://github.com/extphprs/ext-php-rs/issues/436)] 
> Methods and Properties are renamed to camelCase by default. Classes to PascalCase, constants to UPPER_CASE and functions to snake_case
- *(class)* [**breaking**] Generate correct stubs for extends and implements (by @Xenira) [[#326](https://github.com/extphprs/ext-php-rs/issues/326)] 
> `extends` and `implements` attributes now require the `stub` property containing the class/interface name to be used in stubs.
- *(macro)* [**breaking**] Uinify attributes in `#[php]` attribute (by @Xenira) [[#391](https://github.com/extphprs/ext-php-rs/issues/391)] 
> Attributes like `#[prop]`, `#[rename]`, etc. have been moved to `#[php]` attributes like `#[php(prop)]`, `#[php(name = "Foo")]`, `#[php(change_case = CamelCase)]`, etc.
- *(macro)* [**breaking**] Switch to builder pattern (by @davidcole1340, @danog, @ptondereau, @Xenira) [[#99](https://github.com/extphprs/ext-php-rs/issues/99)] [[#131](https://github.com/extphprs/ext-php-rs/issues/131)] [[#327](https://github.com/extphprs/ext-php-rs/issues/327)] [[#174](https://github.com/extphprs/ext-php-rs/issues/174)] [[#335](https://github.com/extphprs/ext-php-rs/issues/335)] 
> The old macros were dependent on execution order and have been causing trouble with language servers. They are replaced by a builder. See the migration guide at https://davidcole1340.github.io/ext-php-rs/migration-guides/v0.14.html for information on how to migrate.

### Added
- Argument defaults can be any expr valid in const scope (by @alekitto) [[#433](https://github.com/extphprs/ext-php-rs/issues/433)] 

### Fixed
- *(args)* Fix variadic args (by @Xenira) [[#337](https://github.com/extphprs/ext-php-rs/issues/337)] 
- *(macro)* Add missing static flags in `php_impl` macro (by @Norbytus) [[#419](https://github.com/extphprs/ext-php-rs/issues/419)] 
- *(macro)* Add missing separator pipe in flags (by @Norbytus) [[#412](https://github.com/extphprs/ext-php-rs/issues/412)] 

### Other
- *(bindings)* Add tooling to generate `docsrs_bindings.rs` (by @Xenira) [[#443](https://github.com/extphprs/ext-php-rs/issues/443)] 
- *(cargo-php)* Add locked option to install guide ([#370](https://github.com/extphprs/ext-php-rs/pull/370)) (by @Xenira) [[#370](https://github.com/extphprs/ext-php-rs/issues/370)] [[#314](https://github.com/extphprs/ext-php-rs/issues/314)] 
- *(clippy)* Apply pedantic rules (by @Xenira) [[#418](https://github.com/extphprs/ext-php-rs/issues/418)] 
- *(coverage)* Add coverage badge (by @Xenira)
- *(deps)* Update syn and darling ([#400](https://github.com/extphprs/ext-php-rs/pull/400)) (by @Xenira) [[#400](https://github.com/extphprs/ext-php-rs/issues/400)] 
- *(guide)* Directly include doc comments (by @Xenira)
- *(macro)* Change `rename` to `change_case` (by @Xenira)
- *(macro)* Use `#[php]` attribute for startup function (by @Xenira) [[#423](https://github.com/extphprs/ext-php-rs/issues/423)] 
- *(macro)* Trait rename for general and method names (by @Norbytus) [[#420](https://github.com/extphprs/ext-php-rs/issues/420)] 
- *(macro)* Update documentation for builder pattern (by @Xenira)
- *(macro)* Add stubs for new builder pattern (by @Xenira) [[#183](https://github.com/extphprs/ext-php-rs/issues/183)] 
- Add git hooks and `CONTRIBUTING.md` (by @Xenira) [[#475](https://github.com/extphprs/ext-php-rs/issues/475)] 
- Typo in README.md (by @kakserpom)

## [0.10.2](https://github.com/extphprs/ext-php-rs/compare/ext-php-rs-derive-v0.10.1...ext-php-rs-derive-v0.10.2) - 2025-02-06

### Other
- Typo when error for #[defaults] macro (by @yoramdelangen)
- Don't use symbolic links for git. (by @faassen)
- Fix pipeline (#320) (by @Xenira) [[#320](https://github.com/extphprs/ext-php-rs/issues/320)] 
- Support for variadic functions (by @joehoyle)
