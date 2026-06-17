# Changelog

## Unreleased

## v0.1.6 - 2026-06-16

### Improved

- Centralized Git working-tree parsing with structured `git status --porcelain=v1 -z` handling.
- Centralized path classification across `changes`, `suggest-commit`, and PR draft generation.
- Improved consistency between `koba changes`, `koba suggest-commit`, and `koba pr --dry-run`.
- Improved detection of cross-cutting internal refactors so shared analysis changes are not split into misleading module-specific feature groups.
- Improved mixed-surface repository display, including Rust CLI repositories that also include Agent Skills.
- Clarified doctor output for recommended Rust checks that are not configured in `koba.yml`.

### Fixed

- Added regression coverage to ensure files shown by `suggest-commit` match files included in recommended staging commands.
- Added regression coverage to ensure PR draft generation includes all untracked files from structured Git status.

### Safety

- Preserved Koba's read-only behavior for analysis commands.
- No staging, committing, pushing, applying, or history rewriting behavior was added.

## v0.1.5 - 2026-06-16

### Improved

- Improved `koba init` preview output when `koba.yml` already exists.
- Clarified that `koba init --apply` refuses to overwrite existing workflow contracts.
- Reduced manifest noise in `koba scan` for Agent Skill repositories by omitting missing `package.json`, `Cargo.toml`, and `pyproject.toml` markers when they are not relevant.

### Safety

- Preserved the existing no-overwrite behavior for `koba init --apply`.

## v0.1.4 - 2026-06-16

### Added

- Added first-class Agent Skill repository detection from `skills/*/SKILL.md`.
- Added Agent Skill-aware scan output for skill slugs, references, examples, evals, smoke prompts, and README files.
- Added `agent-skill` workflow initialization with `git diff --check` and `npx skills add . --list` as default validation checks.

### Improved

- Improved `doctor` so Agent Skill repositories are treated as supported projects instead of unknown custom repos.
- Improved `suggest-commit` and `changes` for generic `skills/*/**`, `evals/**`, and `tests/smoke-prompts.md` changes.
- Improved `changes` grouping so skill examples, evals, and smoke prompts are treated as coherent skill enhancement work.
- Replaced Koba-specific skill docs wording with generic Agent Skill documentation messaging.

### Safety

- Agent Skill support remains read-only by default and does not run skill validation, stage files, commit, push, or write files unless explicitly requested through existing preview/apply flows.

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
