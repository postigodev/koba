# Changelog

## Unreleased

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
