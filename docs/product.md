# Product Model

Koba is a local-first workflow configurator for Git repositories. Its core object is a repo workflow contract: the local files and commands that define how changes move from working tree to commit, push, and pull request.

The contract currently covers commit conventions, checks, hooks, GitHub PR template infrastructure, and repo hygiene signals. The product flow is discovery -> recommendation -> preview -> explicit apply.

## Implemented MVP

- `scan`: read-only overview of Git, workflow files, hooks, and `.github/` assets.
- `doctor`: diagnostics and next steps based on scan results.
- `init`: preview or write a minimal `koba.yml`.
- `run`: execute `pre-commit` and `pre-push` checks from `koba.yml`.
- `hooks install`: preview or write native Git hook and Husky hook files that delegate to `koba run`.
- `github template pr`: preview or write `.github/pull_request_template.md`.
- `suggest-commit`: recommend a Conventional Commit message and safe Git commands.
- `pr`: draft a PR title/body locally and optionally write `.koba/pr-body.md`.

## Core Use Cases

- Understand the current workflow setup in a repository.
- Make expected local checks explicit and runnable.
- Connect hooks to a single check runner instead of duplicating logic.
- Generate practical PR infrastructure without overwriting user files.
- Surface safe commands for humans and AI coding agents.
- Prepare commit and PR text without mutating Git state.

## Safety Boundaries

Koba must remain conservative by default.

- Read-only inspection should be easy.
- Generated files should preview before writing.
- `--apply` may write only the documented target file for that command.
- Existing user files must not be overwritten.
- Koba must not commit, push, rebase, rewrite history, store GitHub tokens, call GitHub APIs, or open pull requests.

## Current Non-Goals

- Replacing Git, Husky, GitHub Actions, GitHub CLI, or language-specific tooling.
- Package script discovery.
- Hook force-overwrite behavior.
- GitHub API or GitHub CLI integration.
- Remote services or AI-dependent core behavior.
- Studio-style UI workflows.

## Roadmap

- Tighten the `koba.yml` schema and validation.
- Add richer diagnostics for `.github/` and hook setups.
- Support more project profiles and check suggestions.
- Improve branch/base detection for PR drafts.
- Add optional integrations only after the local workflow model is solid.
