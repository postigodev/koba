<p align="center">
  <img src="assets/brand/koba-logo-stacked.png" alt="Koba logo" width="420">
</p>

<p align="center">
  Local-first Git workflow configuration for real repositories.
</p>

# Koba

Koba is a local-first Git workflow configurator for real repositories. It helps you inspect workflow infrastructure, draft a repo workflow contract, run configured checks, connect hooks, generate PR templates, and prepare commit/PR text without taking over Git.

Koba is built around a simple safety model: read first, recommend next, preview writes, and apply only when explicitly requested.

> [!IMPORTANT]
> Koba does not commit, push, rebase, rewrite history, store GitHub tokens, call GitHub APIs, or open pull requests. It shells out to local tools such as `git` and your configured check commands.

## What Koba Can Do Today

- Inspect repository workflow files with `scan`.
- Diagnose missing or risky workflow setup with `doctor`.
- Preview or write a minimal `koba.yml` workflow contract.
- Run `pre-commit` and `pre-push` checks from `koba.yml`.
- Preview or install native Git hook and Husky hook files that call `koba run`.
- Preview or generate a GitHub pull request template.
- Suggest Conventional Commit messages and safe Git commands.
- Draft a local PR title/body without opening a PR.

## Install

Koba is early MVP software. The CLI is useful for local dogfooding today, but the command surface and config schema may still change.

Install the CLI locally from this workspace:

```sh
cargo install --path crates/koba
```

Or run it directly from the workspace:

```sh
cargo run -p koba -- scan
```

Prebuilt GitHub Release binaries are produced from version tags once releases are cut. See [docs/distribution.md](docs/distribution.md) for the staged distribution plan.

## Quickstart

From a Git repository:

```sh
koba scan
koba doctor
koba init
koba init --apply
koba run pre-commit --dry-run
koba hooks install --adapter native --dry-run
koba github template pr --dry-run
koba suggest-commit
koba pr --dry-run
```

The default mode for generators is preview-only. Commands such as `init`, `hooks install`, `github template pr`, and `pr` only write files when passed `--apply`, and they refuse to overwrite existing user files.

## Commands

| Command | What it does | Writes files? |
| --- | --- | --- |
| `koba scan` | Shows what workflow infrastructure exists. | No |
| `koba doctor` | Interprets scan results into diagnostics and next steps. | No |
| `koba init` | Prints a proposed `koba.yml`. | No |
| `koba init --apply` | Writes `koba.yml` if it does not already exist. | Yes |
| `koba run pre-commit` | Runs `checks.preCommit` from `koba.yml`. | No |
| `koba run pre-push` | Runs `checks.prePush` from `koba.yml`. | No |
| `koba run <stage> --dry-run` | Lists checks without executing them. | No |
| `koba hooks install --adapter native` | Previews native hook files for `.git/hooks/`. | No |
| `koba hooks install --adapter native --apply` | Writes missing native hook files. | Yes |
| `koba hooks install --adapter husky` | Previews Husky hook files for `.husky/`. | No |
| `koba hooks install --adapter husky --apply` | Writes missing Husky hook files. | Yes |
| `koba github template pr` | Previews `.github/pull_request_template.md`. | No |
| `koba github template pr --apply` | Writes the PR template if it does not already exist. | Yes |
| `koba suggest-commit` | Suggests a Conventional Commit message and Git commands. | No |
| `koba pr` | Previews a PR title and body from local Git state. | No |
| `koba pr --apply` | Writes `.koba/pr-body.md` if it does not already exist. | Yes |

## Example `koba.yml`

```yaml
version: 1
profile: rust-cli

commits:
  convention: conventional
  requireScope: true

checks:
  preCommit:
    - cargo fmt --check
  prePush:
    - cargo test
```

`koba run pre-commit` executes each command under `checks.preCommit`. `koba run pre-push` executes each command under `checks.prePush`. For now, checks are simple shell commands.

## Development

Run the scoped checks:

```sh
cargo fmt --check
cargo check -p koba
cargo test -p koba
git diff --check
```

Koba is intentionally conservative. When adding new behavior, prefer structured results, preview/apply flows, and tests that use temporary repositories instead of mutating the real workspace.
