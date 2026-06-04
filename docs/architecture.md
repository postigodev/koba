# Architecture

Koba starts as a small Rust workspace with one CLI crate. Keep that shape for now. The project should gain internal modules before it gains more crates, avoiding premature workspace abstraction while the product model is still settling.

## Current Structure

```text
.
├── Cargo.toml
├── crates/
│   └── koba/
│       ├── Cargo.toml
│       ├── src/
│       │   ├── cli.rs
│       │   ├── commands.rs
│       │   ├── lib.rs
│       │   └── main.rs
│       └── tests/
│           └── cli.rs
├── docs/
│   ├── architecture.md
│   └── product.md
└── examples/
    └── koba.yml
```

## Intended Module Direction

- `cli`: argument parsing and command dispatch.
- `commands`: user-visible command handlers.
- `repo`: repository discovery, root detection, and file scanning.
- `github`: `.github/` scanning for workflows, pull request templates, issue templates, and repository automation files.
- `diagnostics`: findings, severities, recommendations, and explainable output.
- `config`: `koba.yml` model, loading, validation, and defaults.
- `hooks`: adapters for native Git hooks and hook managers such as Husky.
- `exec`: command execution wrappers for scoped checks and external tools.
- `apply`: file writing plans, previews, diffs, and explicit apply behavior.
- `git`: future narrow shell-out wrapper around the `git` executable.
- `workflow`: future check and hook model.

## Design Decisions

- Use `clap` derive for a clear CLI surface and generated help.
- Keep `main.rs` tiny so the binary only handles process exit behavior.
- Prefer shelling out to `git` later instead of adding `git2` immediately.
- Keep configuration YAML-based eventually, but avoid implementing parsing before the schema is useful.
- Keep commands placeholder-only until scan and diagnostics behavior is designed.
- Add integration tests around the binary for user-visible behavior.
- Treat `.github/` as a first-class workflow surface, not a special case hidden inside generic file scanning.

## Safety Model

Koba should separate read, preview, and write behavior.

- Discovery commands read repository state and produce findings.
- Recommendation commands explain what could improve and why.
- Preview commands show planned file changes before writing.
- Apply commands write files only after an explicit flag or interactive confirmation.

Koba must never rewrite history, auto-commit, push, store GitHub tokens, or hide file changes from the user. Future GitHub CLI integration should call `gh` and inherit its existing authentication.

## Adapter Model

- Native Git hooks: inspect and plan hook files without assuming a package manager.
- Husky: treat Husky as an adapter for JavaScript repositories, not as a competitor.
- GitHub CLI: use `gh` for authenticated GitHub operations later, without storing tokens.
- `.github` generation: preview and apply workflow files, PR templates, issue templates, and related repository assets.

## Testing Strategy

- CLI smoke tests for command availability and placeholder behavior.
- Temp repo fixture tests for repository discovery and command behavior.
- Scanner tests using synthetic file trees for `.github/`, hooks, and config detection.
- No tests should mutate a real user repository.
- Apply-plan tests should assert planned writes before any file is written.
