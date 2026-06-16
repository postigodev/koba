# Changelog

## Unreleased

## v0.1.3 - 2026-06-16

### Added

- Added `koba changes`, a read-only working-tree review command that summarizes changed, staged, unstaged, and untracked files.
- Added commit group planning for mixed working trees so Koba can recommend splitting unrelated changes before staging.
- Added check recommendations based on changed file types, including Rust, docs, skills, Scoop manifests, GitHub Actions, Node/TypeScript, and Python projects.

### Improved

- Improved Conventional Commit suggestions with sharper path-based scopes for agent skills, Scoop packaging, GitHub workflows, and Koba internals.
- Updated the Koba Agent Skill to prefer `koba changes` for working-tree review, commit preparation, and check planning.
- Documented the Koba source-workspace fallback for agents when the global `koba` executable is not on `PATH`.

### Safety

- `koba changes` is fully read-only and does not stage, commit, push, apply changes, or rewrite history.

## v0.1.2 - 2026-06-06

- Improved repository-wide terminal output consistency and legibility.
- Replaced Unicode-only status glyphs with ASCII status badges for cross-platform readability.
- Added aligned diagnostic and planning rows with section-local column widths.
- Clarified preview, execution, commit suggestion, and PR draft output structure.
- Preserved existing product behavior, safety rules, generated file contents, and exit-code semantics.

## v0.1.1 - 2026-06-06

- Clarified `suggest-commit` help text to describe working tree changes.
- Improved `scan` output outside Git repositories by omitting Git identity warnings.

## v0.1.0 - 2026-06-06

- Added read-only repository scanning with `koba scan`.
- Added workflow diagnostics with `koba doctor`.
- Added preview/apply workflow contract generation with `koba init`.
- Added check execution for `pre-commit` and `pre-push` stages with `koba run`.
- Added native Git hook and Husky hook preview/apply flows.
- Added GitHub pull request template generation.
- Added recommend-only Conventional Commit suggestions.
- Added local pull request draft generation.
