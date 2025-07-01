# Contributing to `ext-php-rs`

Thank you for your interest in contributing to `ext-php-rs`! We welcome contributions of all kinds:

- Documentation expansion (examples in particular!)
- Safety reviews (especially if you have experience with Rust and the Zend API).
- Bug fixes and features.
- Feature requests.
- And more!

If you have bigger changes in mind, please open an issue first to discuss the change with us.

## Git Hooks

To catch common mistakes, we use git hooks to run checks before commits and pushes.
For this we use [lefthook](https://github.com/evilmartians/lefthook). See
[the installation docs](https://lefthook.dev/installation/) for instructions on how to install it on
your system.

After installing lefthook, you can run the following command to install the hooks:

```bash
lefthook install
```

### Dependencies

- When changing the `allowed_bindings.rs`, you need to have `docker` and `buildx` installed. This is required to
  build the bindings in a consistent environment. See the [installation guide](https://docs.docker.com/engine/install/)
  for instructions on how to install Docker.
- When updating the macro guides (`guide/src/macros`), you need to have the `nightly` toolchain installed. This is required
  to have the proper formatting in the documentation.

## Testing

We have both unit and integration tests. When contributing, please ensure that your changes are at least
covered by an integration test. If possible, add unit tests as well. This might not always be possible
due to the need of a running PHP interpreter.

### State of unit tests
There are still large parts of the library that are not covered by unit tests. We strive to cover
as much as possible, but this is a work in progress. If you make changes to untested code, we would
appreciate it if you could add tests for the code you changed.

If this is not possible, or requires a lot of unrelated changes, you don't have to add tests. However,
we would appreciate it if you are able to add those tests in a follow-up PR.

## Documentation

Our documentation is located in the `guide` directory.
If you update functionality, please ensure that the documentation is updated accordingly.

### Breaking Changes
If you make a breaking change, please
If your change is a [breaking change](https://semver.org) a migration guide MUST be included. This
MUST be placed in the `guide/src/migration-guides` directory and named `v<next-version>.md` (e.g. `v0.14.md`).
This guide MUST also be linked in the `guide/src/SUMMARY.md` file under the `Migration Guides` section.

## Commit Messages

We are using [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/) to generate our changelogs.
For that reason our tooling ensures that all commit messages adhere to that standard.

To make this easier you can use [convco](https://convco.github.io) to generate commit messages.

## Use of AI tools

- Using AI tools to generate Issues is NOT allowed. AI issues will be closed without comment.
- Using AI tools to generate entire PRs is NOT allowed.
- Using AI tools to generate short code snippets is allowed, but the contributor must review and understand
  the generated code. Think of it as a code completion tool.

This is to ensure that the contributor has a good understanding of the code and its purpose.

## License

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as described in the [README](README.md#license), without any additional terms or conditions.
