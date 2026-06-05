# Architecture

Koba is a Rust workspace with one CLI crate at `crates/koba`. Keep this single-crate shape until module boundaries become stable enough to justify splitting crates.

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
│       │   ├── config.rs
│       │   ├── doctor.rs
│       │   ├── executor.rs
│       │   ├── git.rs
│       │   ├── github.rs
│       │   ├── hooks.rs
│       │   ├── init.rs
│       │   ├── output.rs
│       │   ├── pr.rs
│       │   ├── repo.rs
│       │   ├── run_checks.rs
│       │   ├── scan.rs
│       │   ├── suggest_commit.rs
│       │   ├── lib.rs
│       │   └── main.rs
│       └── tests/
│           └── cli.rs
├── docs/
├── examples/
└── README.md
```

## Modules

- `cli`: `clap` command definitions and top-level dispatch.
- `commands`: thin user-command handlers that pass current directory/options into modules.
- `git`: narrow shell-out helpers for Git discovery, status, and simple branch/commit lookup.
- `repo`: file-tree discovery for workflow files, hooks, and `.github/` assets.
- `scan`: read-only workflow overview rendering.
- `doctor`: structured diagnostics and recommendations from scan data.
- `init`: preview/apply generation for `koba.yml`.
- `config`: minimal YAML config model and parser for `koba.yml`.
- `executor`: platform shell execution for configured checks.
- `run_checks`: stage selection and execution for `pre-commit` and `pre-push`.
- `hooks`: preview/apply plans for native Git hooks and Husky hooks.
- `github`: preview/apply generation for GitHub PR templates.
- `suggest_commit`: deterministic Conventional Commit recommendation heuristics.
- `pr`: local PR title/body drafting and optional `.koba/pr-body.md` output.
- `output`: centralized status line formatting.

## Design Decisions

- Use `clap` derive for a clear CLI surface.
- Keep `main.rs` tiny and return process exit codes from command results.
- Shell out to `git` instead of using `git2`.
- Keep file writes behind explicit `--apply` flows.
- Represent generated writes as simple plans with target path, contents, and existing-file state.
- Keep `commands.rs` thin so behavior is testable in modules.
- Keep config parsing minimal and forward-compatible with unknown fields.

## Safety Model

Koba separates read, preview, and write behavior.

- `scan`, `doctor`, `suggest-commit`, and default `pr` are read/recommend-only.
- `init`, `hooks install`, `github template pr`, and `pr` preview by default.
- `--apply` writes only the documented target file(s).
- Existing files are not overwritten.
- Koba does not commit, push, rewrite history, store GitHub tokens, call GitHub APIs, or open PRs.

## Testing Strategy

- Unit tests cover parser behavior, scanner fixtures, diagnostics, generation plans, and deterministic heuristics.
- CLI integration tests run the compiled binary in temporary directories and temporary Git repositories.
- Tests avoid mutating the real repository.
- Apply-plan tests assert both preview behavior and no-overwrite behavior.
