# Installer Implementation Plan

> REQUIRED SUB-SKILL: Use `superpowers:executing-plans` for inline execution.

**Goal:** Add a POSIX `scripts/install.sh` that downloads and installs the correct `pb` release binary for Linux, macOS, and WSL.

## Global Constraints

- No project binary behavior changes; only add `scripts/install.sh` and README docs.
- Script must be POSIX-ish, shellcheck-clean, and avoid non-standard dependencies beyond `curl`, `tar`, `uname`.
- README install section must mention one-liner, supported platforms, manual fallbacks, and security note (inspect before piping).
- CI lint: add `shellcheck scripts/install.sh` step if shellcheck is available; otherwise a basic `bash -n` syntax check.

---

### Task 1: Create `scripts/install.sh`

**Files:**
- Create: `scripts/install.sh`

**Interfaces:**
- Input: CLI flags `--help`, `--prefix`, `--tag`, `--version`, `--force`, `--skip-version-check`.
- Output: installs `pb` binary, prints version and path.

- [x] **Step 1: Detect target triple**
  Map `uname -s`/`-m` to the five supported Rust targets.

- [x] **Step 2: Resolve release tag**
  Default to `https://api.github.com/repos/mintychochip/pebblehost-cli/releases/latest`; parse `tag_name`; strip `v` to get version.
  Optional `--tag`/`--version` override.

- [x] **Step 3: Download and extract**
  Construct `https://github.com/mintychochip/pebblehost-cli/releases/download/${tag}/pebblehost-cli-${version}-${target}.tar.gz`.
  Download to a temp directory, extract `pb` (or `pb.exe` on Windows, but script targets POSIX).

- [x] **Step 4: Choose install prefix**
  Default: `~/.local/bin` if it exists or is creatable; else `/usr/local/bin` if writable; else print a `sudo` hint.
  Honor `--prefix`.

- [x] **Step 5: Install binary and verify**
  Use `cp`/`chmod +x` to place binary. Run `pb --version` unless `--skip-version-check`.

- [x] **Step 6: Error handling**
  Clear messages for unsupported platform, failed download, or missing `curl`/`tar`.

---

### Task 2: Update README

**Files:**
- Modify: `README.md`

- [x] **Step 1: Add "Install" section**
  Include one-liner, supported platforms, `--prefix` example, `cargo install` fallback, and the standard "download and inspect" security note.

---

### Task 3: Add CI lint for `scripts/install.sh`

**Files:**
- Modify: `.github/workflows/lint.yml`

- [x] **Step 1: Add shellcheck or syntax check job step**
  Install shellcheck if missing, run `shellcheck scripts/install.sh` or `sh -n scripts/install.sh`.

---

### Task 4: Manual verification

- [x] **Step 1: Syntax check**
  Run `shellcheck scripts/install.sh` locally if available; otherwise `sh -n scripts/install.sh`.

- [x] **Step 2: Dry-run test**
  Run the script with `--prefix /tmp/pb-test` and a specific tag, verify binary installs and `pb --version` works.

- [x] **Step 3: Run existing Rust tests/lint**
  `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.

---

### Task 5: Commit and push

- [x] **Step 1: Add `scripts/install.sh`, README, lint.yml, plan, spec**
  `git add` all modified/new files and commit.

- [x] **Step 2: Push to master**
