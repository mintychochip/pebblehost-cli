# PebbleHost CLI

[![build](https://github.com/mintychochip/pebblehost-cli/actions/workflows/build.yml/badge.svg)](https://github.com/mintychochip/pebblehost-cli/actions/workflows/build.yml)
[![Release](https://img.shields.io/github/v/release/mintychochip/pebblehost-cli?logo=github&label=release)](https://github.com/mintychochip/pebblehost-cli/releases)
[![Platforms](https://img.shields.io/badge/platforms-linux%20%7C%20macos%20%7C%20windows-lightgrey)](https://github.com/mintychochip/pebblehost-cli/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

An unofficial Rust command-line interface for the PebbleHost client API.

> Unofficial project: this CLI is not affiliated with, endorsed by, sponsored by, or otherwise associated with the PebbleHost brand.

## API coverage

The CLI includes convenient commands for common operations and an escape hatch for the complete published API:

```bash
# List the 141 operations in the bundled API inventory
pb operations

# Call any documented endpoint directly
pb api-call GET /api/client/servers/SERVER_ID/resources
pb api-call POST /api/client/servers/SERVER_ID/command \
  --body '{"command":"say hello"}'
```

`api-call` accepts `GET`, `POST`, `PUT`, `PATCH`, and `DELETE`, repeatable `--query KEY=VALUE` parameters, and a raw JSON `--body`. The default request base URL follows the published OpenAPI server, `https://panel.pebblehost.com`; override it with `--base-url`.

For a compact, agent-friendly response payload, use the global `--json` flag
or its `--verbose` alias with any command:

```bash
pb --verbose account
pb servers --verbose
```

Both spellings print successful JSON responses as one sorted line without
request metadata. The default output is parsed text rather than JSON:
JSON:API `attributes` are unwrapped, list items are indented, and nested
objects are rendered as `key: value` fields. For example, `pb account` prints:

```text
admin: false
email: user@example.com
id: null
language: en
object: user
```

Use `--json`/`--verbose` when another program needs JSON. Raw text responses,
such as `file`, remain unchanged, and diagnostics continue to go to stderr.

## Install

The easiest way to install on Linux, macOS, or WSL is with the one-line installer:

```bash
curl -sSL https://raw.githubusercontent.com/mintychochip/pebblehost-cli/master/scripts/install.sh | sh
```

This detects your OS and architecture, downloads the latest release, and places the `pb` binary in `~/.local/bin` (or `/usr/local/bin` if that is not writable). Make sure the install directory is on your `PATH`.

For a different install location:

```bash
curl -sSL https://raw.githubusercontent.com/mintychochip/pebblehost-cli/master/scripts/install.sh | sh -s -- --prefix /usr/local/bin
```

You can also pin a specific release:

```bash
curl -sSL https://raw.githubusercontent.com/mintychochip/pebblehost-cli/master/scripts/install.sh | sh -s -- --tag v2026.8.15.3
```

> Security note: piping scripts directly from the internet is convenient but risky. If you prefer, download `scripts/install.sh`, review it, and run it locally with `sh scripts/install.sh`.

Supported platforms: x86_64/aarch64 Linux, x86_64/aarch64 macOS, and armv7 Linux.

## Update

To update to the latest release, run:

```bash
pb update
```

This fetches the current updater script and re-runs the installer for the `pb` binary that is first on your `PATH`.

## Usage

```bash
pb --help
pb --version
pb update
```

### Login

Run `pb login` to open `https://panel.pebblehost.com/account/api`. Generate an
API key there and paste it into the hidden terminal prompt. The validated key
is stored in the per-user CLI config and is used by authenticated commands.
`PEBBLEHOST_API_KEY` remains the higher-priority override for scripts and CI.

Browser launch is best-effort. The API key is never printed and is not accepted
as a command-line argument.

Use `--base-url` to point at a different panel.

The implementation follows the published OpenAPI document at https://api.pebblehost.com/api.yaml and uses documented bearer-token authentication.
