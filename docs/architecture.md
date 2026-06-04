# Architecture

Koba starts as a small Rust workspace with one CLI crate. The structure should make the command surface easy to extend without locking the project into premature abstractions.

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
- `config`: future `koba.yml` loading and validation.
- `repo`: future repository discovery and file scanning.
- `git`: future narrow shell-out wrapper around the `git` executable.
- `diagnostics`: future findings, severities, and recommendations.
- `workflow`: future check and hook model.

## Design Decisions

- Use `clap` derive for a clear CLI surface and generated help.
- Keep `main.rs` tiny so the binary only handles process exit behavior.
- Prefer shelling out to `git` later instead of adding `git2` immediately.
- Keep configuration YAML-based eventually, but avoid implementing parsing before the schema is useful.
- Keep commands placeholder-only until scan and diagnostics behavior is designed.
- Add integration tests around the binary for user-visible behavior.

## Safety Model

Koba should treat writes as deliberate operations. Read-only commands can run by default. Any command that writes files should first show what will change and require either an explicit apply flag or an interactive confirmation.

Koba must never rewrite history, auto-commit, push, or hide file changes from the user.
