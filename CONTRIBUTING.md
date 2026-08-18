# Contributing to PebbleHost CLI

Thanks for your interest in contributing. This is an unofficial community project; we welcome bug reports, feature suggestions, and pull requests.

## Before you start

- Check the [README](README.md) for project overview and install instructions.
- Search existing issues and pull requests to avoid duplicates.
- For questions or usage help, use [GitHub Discussions](https://github.com/mintychochip/pebblehost-cli/discussions).

## Development setup

You need a recent stable [Rust toolchain](https://rustup.rs/).

```bash
cargo build
cargo test --all-features
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

If you change the shell scripts, run `shellcheck scripts/install.sh scripts/update.sh` or `sh -n` if shellcheck is not installed.

## Project conventions

- Follow the existing command structure in `src/main.rs`.
- Keep the public CLI surface minimal. New one-off endpoints should usually be reachable through the existing `pb api-call` command rather than adding a dedicated subcommand.
- Use `clap` derive macros for new subcommands and arguments.
- Keep error handling idiomatic with `thiserror` and `Result` propagation.

## Commit style

Mirror the existing commit history. Common prefixes include:

- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation
- `ci:` for GitHub Actions or CI changes
- `chore:` for maintenance tasks
- `release:` for version bumps (usually automated)

## Pull request process

1. Open a pull request against the `master` branch.
2. Make sure the `Lint and Test` workflow passes.
3. Link any related issue in the PR description.
4. Request a review and respond to feedback.

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
