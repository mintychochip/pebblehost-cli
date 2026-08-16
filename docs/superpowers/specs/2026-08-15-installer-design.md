# `pb` Installer Design

## Goal

Provide a one-line curl installer so Linux, macOS, and WSL users can install the matching `pb` release binary without manually picking an asset.

## Target

POSIX-compatible shell script at `scripts/install.sh`.

## Behavior

- Detect OS (`uname -s`) and architecture (`uname -m`):
  - Linux x86_64 → `x86_64-unknown-linux-gnu`
  - Linux aarch64 → `aarch64-unknown-linux-gnu`
  - Linux armv7l / armv7 → `armv7-unknown-linux-gnueabihf`
  - macOS x86_64 → `x86_64-apple-darwin`
  - macOS arm64 → `aarch64-apple-darwin`
- Resolve the requested release:
  - Default: latest GitHub release (`https://api.github.com/repos/mintychochip/pebblehost-cli/releases/latest`)
  - Optional `--tag vX.Y.Z.run` or `--version X.Y.Z.run`
- Download the matching `pebblehost-cli-<version>-<target>.tar.gz` asset.
- Verify the download succeeded (non-empty, HTTP 200).
- Extract the `pb` binary.
- Install it to a `bin/` directory:
  - Default prefix: first writable of `~/.local/bin`, `/usr/local/bin`.
  - Optional `--prefix <dir>`.
  - If `/usr/local/bin` is chosen and not writable, prompt to re-run with `sudo`.
- Run `pb --version` and print the install path.
- Exit with a clear message on unsupported platform or download failure.

## CLI flags

```text
-s, --skip-version-check   do not run pb --version after install
-f, --force                overwrite existing binary
-p, --prefix <dir>         install directory (default: first writable of ~/.local/bin, /usr/local/bin)
-t, --tag <tag>            install a specific release tag (e.g. v2026.8.15.3)
-v, --version <version>    install a specific release version (e.g. 2026.8.15.3)
-h, --help                 show usage
```

## One-line install command

```bash
curl -sSL https://raw.githubusercontent.com/mintychochip/pebblehost-cli/master/scripts/install.sh | sh
```

With a prefix:

```bash
curl -sSL https://raw.githubusercontent.com/mintychochip/pebblehost-cli/master/scripts/install.sh | sh -s -- --prefix /usr/local/bin
```

## README update

Add an "Install" section with the one-liner, supported platforms, and the manual `cargo install --path .` / `cargo install --git` fallback.

## Verification

- Manual: run the script in a clean container/VM for each supported target.
- CI: add a shell lint step (`shellcheck` if available).
- Smoke test: run `pb --version` after install.

## Out of scope

- Windows native installer (PowerShell). The release already ships a Windows `.tar.gz`, but the curl/sh installer is intentionally POSIX-only.
- Wizard that prompts for API token or base URL. The script is non-interactive; token setup remains in the README.
