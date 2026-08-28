# `pb login` and Build Workflow Design

## Goal

Add a local `pb login` flow so a human can generate a PebbleHost API key in the
panel once, enter it without exposing it in the terminal, and then let an
agent run authenticated `pb` commands without carrying the key on every
invocation. Make the repository status badge represent a real Build workflow
and make that workflow explicitly compile the CLI.

## Verified PebbleHost flow

PebbleHost does not document an OAuth, device-code, or browser callback flow.
The supported flow is to sign in to the panel and use the account page's
**Generate API Key** action. `pb login` therefore opens the user-provided
canonical page:

`https://panel.pebblehost.com/account/api`

The CLI does not scrape the browser, read cookies, automate account login, or
put a credential in a URL.

## CLI interaction

`pb login` is a local command and does not require an existing API key.

1. Print the API-key page URL on stderr so it is visible even when browser
   launching is unavailable.
2. Best-effort launch the exact URL with the platform-native opener:
   `xdg-open` on Linux, `open` on macOS, and the Windows URL launcher. A failed
   opener is informational; the command continues to the prompt.
3. Prompt for the generated key with terminal echo disabled. The key is never
   accepted as a positional argument or option, printed, logged, or included in
   a user-facing error.
4. Trim surrounding whitespace and reject an empty value.
5. Validate the key with `GET /api/client/account` using the configured base
   URL and the supplied bearer credential.
6. Persist the key only after validation succeeds, then print a token-free
   success message.

The login command is excluded from the periodic version reminder so secret
input is not preceded by unrelated stderr output. `operations` and `update`
remain token-free commands.

## Credential resolution

Authenticated commands resolve credentials in this order:

1. `PEBBLEHOST_API_KEY`, when present and non-empty after trimming.
2. The locally stored key file.
3. `MissingToken` if neither source is available.

If `PEBBLEHOST_API_KEY` is explicitly set to an empty or whitespace-only value,
the command fails instead of silently falling back to the stored credential.
This preserves an explicit environment override and makes CI misconfiguration
visible.

## Local storage

Use the platform configuration directory with an application-specific
subdirectory:

- Linux: `$XDG_CONFIG_HOME/pebblehost-cli`, or `$HOME/.config/pebblehost-cli`.
- macOS: the platform configuration directory under the user home directory.
- Windows: `%APPDATA%/pebblehost-cli`.

Store the raw key plus a trailing newline in `api-key`. Do not use a JSON file,
command-line argument, process environment mutation, or shell history for
persistence.

The directory is created with owner-only permissions (`0700` where supported)
and the key file with owner read/write permissions (`0600` where supported).
Existing paths are checked: symlinks, non-regular files, and files with group or
world permissions are rejected rather than overwritten. Writes use a unique
temporary file in the same directory and an atomic rename, preventing partial
credentials and avoiding backup copies. Existing valid credentials are
replaced only after the new key validates.

A small cross-platform config-directory dependency and a terminal password
prompt dependency are acceptable; no OS keychain integration is required for
this iteration. The file-based approach keeps headless Linux and release
binaries usable while still preventing ordinary process-list, shell-history,
and accidental file-sharing exposure.

## Errors and output

Browser-launch failure does not fail login. Prompt, empty-key, validation, and
storage failures return a non-zero exit status without exposing the key. API
validation uses the existing request path and bearer handling; a non-success
response prevents persistence.

The stored credential is consumed transparently by every authenticated
command. Existing `--json`/`--verbose` response formatting is unchanged.

## Build workflow and badge

Rename `.github/workflows/lint.yml` to `.github/workflows/build.yml` and rename
the workflow and job to **Build**. Preserve the existing formatting, Clippy,
test, and shell-script checks, then add an explicit locked build:

```yaml
- name: Build
  run: cargo build --all-features --locked
```

Change the README Shields badge to the actual `build.yml` workflow URL. Update
the release workflow's `workflow_run` dependency and status messages from
`Lint and Test` to `Build`; otherwise releases would stop waiting on the
renamed workflow. Historical planning documents remain unchanged.

## Testing strategy

Add focused inline tests in `src/main.rs` for:

- parsing `login` and preserving token-free commands;
- empty/whitespace key rejection;
- credential file creation, loading, replacement, permissions, symlink/path
  rejection, and atomic-write failure behavior;
- environment precedence, including explicit empty environment values;
- successful and failed account validation using Wiremock;
- browser-launch failure continuing to prompt, using an injectable opener
  boundary or a platform-neutral helper;
- token-free login success/error output where output capture is available.

Run `cargo fmt -- --check`, `cargo clippy --all-targets --all-features --
-D warnings`, `cargo test --all-features`, and `cargo build --all-features
--locked`. Smoke-test `pb login --help`, the browser-unavailable path, and an
authenticated command using a temporary config directory without printing the
credential.

## Non-goals

- OAuth, device-code, browser callbacks, browser scraping, or cookie reuse.
- Remote key creation, rotation, or revocation.
- A `logout` command.
- A `--token` option or positional token input.
- OS keychain integration.
- Rewriting historical design/plan documents.
