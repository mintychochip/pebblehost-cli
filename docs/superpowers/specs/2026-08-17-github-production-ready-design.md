# .github Production-Ready Community Health Design

## Goal

Make the repository's `.github` folder (and related top-level community files) look and behave like a production-ready open-source Rust CLI project, with a focus on orderly contributions and clear issue triage.

## Scope

The user selected the **core contributor set** plus two additions:

1. `.github/ISSUE_TEMPLATE/bug_report.yml` — GitHub form template for bug reports.
2. `.github/ISSUE_TEMPLATE/feature_request.yml` — GitHub form template for feature requests.
3. `.github/ISSUE_TEMPLATE/config.yml` — Points users to GitHub Discussions for questions and disables blank issue templates.
4. `.github/PULL_REQUEST_TEMPLATE.md` — PR checklist and description template.
5. `CONTRIBUTING.md` — Contributor guide at repo root, linked from the PR template.
6. `LICENSE` — MIT license file (project already declares `license = "MIT"` in `Cargo.toml`, but the file is missing).

Out of scope: `CODE_OF_CONDUCT.md`, `SECURITY.md`, `FUNDING.yml`, Dependabot, or additional CI workflows unless explicitly added later.

## Design

### Issue forms

Use GitHub's issue form syntax (`yml` files under `.github/ISSUE_TEMPLATE/`). This gives:

- Required fields where appropriate.
- Dropdowns and checkboxes for common questions.
- Better UX than markdown templates with placeholder comments.

#### `bug_report.yml`

Fields:

- `description` (textarea, required) — What is wrong?
- `reproduce` (textarea, required) — Step-by-step reproduction.
- `expected` (textarea) — Expected behavior.
- `actual` (textarea) — Actual behavior.
- `environment` (textarea, required) — OS/architecture, `pb --version` output, install method (`curl`, `cargo`, etc.), and shell.
- `command` (textarea) — The exact `pb` command and any relevant output or error. Include a note to redact the API token.
- `search` (checkbox) — "I have searched existing issues and this is not a duplicate."

Labels: `bug`.
Title prefix: `[Bug] `.

#### `feature_request.yml`

Fields:

- `problem` (textarea, required) — What problem are you solving?
- `solution` (textarea, required) — What would you like to see?
- `alternatives` (textarea) — Alternatives considered.
- `context` (textarea) — Extra context, mock commands, API endpoints, etc.

Labels: `enhancement`.
Title prefix: `[Feature] `.

#### `config.yml`

- `blank_issues_enabled: false` — Nudges users toward the templates.
- `contact_links` — One link to GitHub Discussions for questions/help. Only useful if Discussions is enabled; the link still works even if disabled (it will be a no-op or prompt to enable).

### Pull request template

`.github/PULL_REQUEST_TEMPLATE.md`:

- Short description section.
- "What changed" bullet list.
- "How to test" section.
- Checklist:
  - Tests added/updated and pass.
  - `cargo fmt` and `cargo clippy` pass.
  - `README.md` or docs updated if needed.
  - Self-review completed.
  - Issue linked if this PR fixes one.

### CONTRIBUTING.md

Placed at repo root so it is discoverable from the project page. Sections:

1. Welcome and code of conduct note (brief, no separate file).
2. Development setup:
   - Install Rust.
   - `cargo build`, `cargo test`.
   - `cargo fmt` / `cargo clippy`.
   - Shell script linting with `shellcheck` or `sh -n`.
3. Project conventions:
   - Follow existing command structure in `src/main.rs`.
   - Keep the CLI API surface minimal; prefer `api-call` for one-off endpoints.
   - Use `clap` derive for new subcommands.
4. Commit style: mirror existing history (`feat:`, `fix:`, `docs:`, `ci:`, `chore:`, `release:`).
5. Pull request process:
   - Open a PR against `master`.
   - Ensure the `Lint and Test` workflow passes.
   - Link related issues.
6. License: contributions are under the MIT license.

### LICENSE

MIT text, copyright holder as `mintychochip` to match the GitHub repository owner. Keep it identical to the standard MIT license so GitHub's license detection works.

## File layout

```text
.github/
  ISSUE_TEMPLATE/
    bug_report.yml
    feature_request.yml
    config.yml
  PULL_REQUEST_TEMPLATE.md
CONTRIBUTING.md
LICENSE
```

## Verification

1. `ls .github/ISSUE_TEMPLATE/` shows the three files.
2. `ls .github/PULL_REQUEST_TEMPLATE.md` exists.
3. `test -f CONTRIBUTING.md && test -f LICENSE`.
4. Optional: `find .github/ISSUE_TEMPLATE -name '*.yml' | xargs yq` or a Python YAML parser to validate syntax.
5. `cargo test` and `cargo clippy` still pass (no Rust code is changed).
6. `git status` shows only the intended new files.
