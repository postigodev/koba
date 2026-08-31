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
│       │   ├── analysis.rs
│       │   ├── changes.rs
│       │   ├── cli.rs
│       │   ├── commands.rs
│       │   ├── config.rs
│       │   ├── doctor.rs
│       │   ├── executor.rs
│       │   ├── git.rs
│       │   ├── git_status.rs
│       │   ├── github.rs
│       │   ├── hooks.rs
│       │   ├── init.rs
│       │   ├── output.rs
│       │   ├── path_classification.rs
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
- `git_status`: structured parsing of Git working-tree status.
- `path_classification`: shared path-to-change-concept classification.
- `analysis`: current deterministic working-tree analysis, commit grouping, message heuristics, checks, and risk assessment.
- `changes`: read-only rendering of working-tree analysis and recommended commit plans.
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

## Intended Commit-First Architecture

The current modules have not yet been refactored into these final boundaries. The intended conceptual flow is:

```text
deterministic analysis / planning
    ↓
commit description generation
    ↓
preview / approval / execution
```

- Analysis owns Git-state inspection, coherent grouping, relevant checks, risks, and mutation-safety constraints.
- Description generation consumes an existing plan. Multiple generators may exist later, but none may select files, decide safety, or infer approval.
- Preview and approval expose the exact paths, final message, checks, and proposed mutation.
- Execution may eventually stage only the approved paths and create only the approved commit.
- Push is outside the default commit flow and must never happen automatically.

Today, `analysis::CommitPlan` still contains deterministic/path-driven message prose, and `suggest_commit` renders manual Git commands. Separating those responsibilities and adding approval-gated execution are roadmap work, not implemented modules.

## Safety Model

Koba separates read, preview, and write behavior.

- `scan`, `doctor`, `suggest-commit`, and default `pr` are read/recommend-only.
- `init`, `hooks install`, `github template pr`, and `pr` preview by default.
- `--apply` writes only the documented target file(s).
- Existing files are not overwritten.
- The current release does not create commits or push.
- Future commit execution must show the exact plan, require explicit approval, and limit mutation to that approved plan.
- The default commit flow must not use broad staging such as `git add .` or push automatically.
- Koba must not silently rewrite history. The current release also does not store GitHub tokens, call GitHub APIs, or open PRs.

## Testing Strategy

- Unit tests cover parser behavior, scanner fixtures, diagnostics, generation plans, and deterministic heuristics.
- CLI integration tests run the compiled binary in temporary directories and temporary Git repositories.
- Tests avoid mutating the real repository.
- Apply-plan tests assert both preview behavior and no-overwrite behavior.
