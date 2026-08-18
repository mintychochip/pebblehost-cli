# .github Production-Ready Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add GitHub issue forms, a pull request template, a contributing guide, and an MIT license file so the repository looks and behaves like a production-ready open-source Rust CLI project.

**Architecture:** The work is entirely new files under `.github/`, `CONTRIBUTING.md` at the repo root, and `LICENSE` at the repo root. No source code is modified, so existing `cargo` tests and `clippy` checks should continue to pass.

**Tech Stack:** GitHub issue form YAML, Markdown, MIT license text.

## Global Constraints

- All new files use standard GitHub path conventions: `.github/ISSUE_TEMPLATE/*.yml`, `.github/PULL_REQUEST_TEMPLATE.md`.
- `CONTRIBUTING.md` lives at repo root, not `.github/CONTRIBUTING.md`.
- `LICENSE` is the standard MIT license with copyright holder `mintychochip`.
- Issue forms must be valid YAML and use fields the GitHub UI understands.
- PR and contribution language must match the existing project tone and conventions.
- Do not modify Rust source, existing workflows, or `Cargo.toml`.

---

### Task 1: Create bug report issue form

**Files:**
- Create: `.github/ISSUE_TEMPLATE/bug_report.yml`

**Interfaces:**
- Produces: a GitHub issue form rendered when users click "Bug report".

- [ ] **Step 1: Write the bug report form**

```yaml
name: Bug report
description: Report a problem with the PebbleHost CLI.
labels: ["bug"]
title: "[Bug] "
body:
  - type: markdown
    attributes:
      value: |
        Thanks for taking the time to report a bug.

        ⚠️ Do **not** paste your PebbleHost API token. Redact it from any command output you share.

  - type: textarea
    id: description
    attributes:
      label: What happened?
      description: A clear and concise description of the bug.
      placeholder: The `pb api-call` command returned an unexpected 401 error when I called ...
    validations:
      required: true

  - type: textarea
    id: reproduce
    attributes:
      label: Steps to reproduce
      description: Steps that someone else can follow to see the same behavior.
      placeholder: |
        1. Run `pb operations`
        2. Run `pb api-call GET /api/client/servers/.../resources`
        3. See error
    validations:
      required: true

  - type: textarea
    id: expected
    attributes:
      label: What did you expect to happen?

  - type: textarea
    id: actual
    attributes:
      label: What actually happened?

  - type: textarea
    id: environment
    attributes:
      label: Environment
      description: |
        OS and architecture, `pb --version` output, install method (curl, cargo, etc.), and shell.
      placeholder: |
        OS: macOS 14.5 (aarch64)
        pb version: v2026.8.16.2
        Installed via: curl installer
        Shell: zsh
    validations:
      required: true

  - type: textarea
    id: command
    attributes:
      label: Relevant command and output
      description: |
        Paste the exact `pb` command and any relevant output. Remember to redact your API token.

  - type: checkboxes
    id: terms
    attributes:
      label: Before submitting
      options:
        - label: I have searched existing issues and this is not a duplicate.
```

- [ ] **Step 2: Validate the YAML**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/ISSUE_TEMPLATE/bug_report.yml'))"`
Expected: command exits `0` with no output.

- [ ] **Step 3: Commit**

Do not commit in Task 1; hold until final commit phase.

---

### Task 2: Create feature request issue form

**Files:**
- Create: `.github/ISSUE_TEMPLATE/feature_request.yml`

**Interfaces:**
- Produces: a GitHub issue form rendered when users click "Feature request".

- [ ] **Step 1: Write the feature request form**

```yaml
name: Feature request
description: Suggest a new feature or improvement for the PebbleHost CLI.
labels: ["enhancement"]
title: "[Feature] "
body:
  - type: markdown
    attributes:
      value: |
        Thanks for taking the time to suggest an improvement.

  - type: textarea
    id: problem
    attributes:
      label: What problem are you trying to solve?
      description: Describe the use case or pain point, not the solution.
      placeholder: I often need to list all servers and their current power state in one command.
    validations:
      required: true

  - type: textarea
    id: solution
    attributes:
      label: What would you like to see?
      description: Describe the feature or command you want added.
      placeholder: |
        Add a `pb servers list --power` flag that prints a table of server IDs and power states.
    validations:
      required: true

  - type: textarea
    id: alternatives
    attributes:
      label: Alternatives considered
      description: Any other ways to solve the problem, or workarounds you use today.

  - type: textarea
    id: context
    attributes:
      label: Additional context
      description: |
        API endpoints, screenshots, mock command output, or anything else that helps.

  - type: checkboxes
    id: terms
    attributes:
      label: Before submitting
      options:
        - label: I have searched existing issues and this is not a duplicate.
```

- [ ] **Step 2: Validate the YAML**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/ISSUE_TEMPLATE/feature_request.yml'))"`
Expected: command exits `0` with no output.

- [ ] **Step 3: Commit**

Do not commit in Task 2; hold until final commit phase.

---

### Task 3: Create issue template configuration

**Files:**
- Create: `.github/ISSUE_TEMPLATE/config.yml`

**Interfaces:**
- Produces: GitHub issue chooser configuration that links to Discussions and disables blank issues.

- [ ] **Step 1: Write the config file**

```yaml
blank_issues_enabled: false
contact_links:
  - name: Ask a question
    url: https://github.com/mintychochip/pebblehost-cli/discussions
    about: Please use GitHub Discussions for general questions, help, or usage guidance.
```

- [ ] **Step 2: Validate the YAML**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/ISSUE_TEMPLATE/config.yml'))"`
Expected: command exits `0` with no output.

- [ ] **Step 3: Commit**

Do not commit in Task 3; hold until final commit phase.

---

### Task 4: Create pull request template

**Files:**
- Create: `.github/PULL_REQUEST_TEMPLATE.md`

**Interfaces:**
- Produces: pre-filled body text for every new pull request.

- [ ] **Step 1: Write the PR template**

```markdown
## Summary

Briefly describe the change and why it is needed.

## What changed

- Bullet list of notable changes.

## How to test

- Steps a reviewer can run to verify the change.

## Checklist

- [ ] I have run `cargo test` and all tests pass.
- [ ] I have run `cargo fmt -- --check` and `cargo clippy --all-targets --all-features -- -D warnings` without errors.
- [ ] I have updated `README.md` or other docs if the change affects user-facing behavior.
- [ ] I have linked any related issue(s).
- [ ] I have reviewed my own diff.
```

- [ ] **Step 2: Commit**

Do not commit in Task 4; hold until final commit phase.

---

### Task 5: Create CONTRIBUTING.md

**Files:**
- Create: `CONTRIBUTING.md`

**Interfaces:**
- Produces: contributor documentation linked from the PR template and repo page.

- [ ] **Step 1: Write the guide**

```markdown
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
```

- [ ] **Step 2: Commit**

Do not commit in Task 5; hold until final commit phase.

---

### Task 6: Create LICENSE

**Files:**
- Create: `LICENSE`

**Interfaces:**
- Produces: the MIT license file declared in `Cargo.toml`.

- [ ] **Step 1: Write the MIT license**

```text
MIT License

Copyright (c) 2026 mintychochip

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

- [ ] **Step 2: Commit**

Do not commit in Task 6; hold until final commit phase.

---


### Task 7: Commit design artifacts

**Files:**


Run:

```bash
git add docs/superpowers/specs/2026-08-17-github-production-ready-design.md \
  docs/superpowers/plans/2026-08-17-github-production-ready.md
```


Run:

```bash
git commit -m "docs: add .github production-ready design and implementation plan"
```

Expected: commit created with the two design files.


### Task 8: Final verification and commit

**Files:**


Run:

```bash
ls -la .github/ISSUE_TEMPLATE/ && \
test -f .github/PULL_REQUEST_TEMPLATE.md && \
test -f CONTRIBUTING.md && \
test -f LICENSE
```

Expected: `bug_report.yml`, `feature_request.yml`, and `config.yml` are listed; PR template, CONTRIBUTING.md, and LICENSE exist.


Run:

```bash
python3 - <<'PY'
import yaml, glob
for f in glob.glob('.github/ISSUE_TEMPLATE/*.yml'):
    yaml.safe_load(open(f))
print('all valid')
PY
```

Expected: prints `all valid`.


Run: `cargo test --all-features && cargo clippy --all-targets --all-features -- -D warnings`
Expected: tests and clippy complete without errors.


Run: `cargo fmt -- --check`
Expected: exit `0`.


Run:

```bash
git add .github/ISSUE_TEMPLATE/bug_report.yml \
  .github/ISSUE_TEMPLATE/feature_request.yml \
  .github/ISSUE_TEMPLATE/config.yml \
  .github/PULL_REQUEST_TEMPLATE.md \
  CONTRIBUTING.md \
  LICENSE
```


Run:

```bash
git commit -m "docs: add GitHub issue/PR templates, contributing guide, and MIT license"
```

Expected: a single commit containing the issue templates, PR template, CONTRIBUTING.md, and LICENSE.
