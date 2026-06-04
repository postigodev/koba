# AGENTS.md

Guidance for AI coding agents working in this repository.

## Project Intent

Koba is a local-first Rust CLI for making Git workflow infrastructure explicit, inspectable, and reproducible. It should integrate with existing tools rather than replace them.

## Working Rules

- Inspect the repository state before editing.
- Keep changes scoped to the requested task.
- Do not commit, push, rewrite history, or stage changes unless explicitly asked.
- Default product behavior to recommend-only. File mutations must be explicit and reviewable.
- Prefer small, testable modules over dumping logic into `main.rs`.
- Use real Git commands through a narrow wrapper when Git integration is added; do not introduce `git2` until there is a clear reason.
- Keep dependencies minimal and justify new ones.
- Run only scoped, relevant checks after changes.

## Rust Conventions

- CLI parsing belongs near the command surface.
- Command execution should live behind modules that are easy to test.
- Prefer clear error messages over clever abstractions.
- Add integration tests for user-visible CLI behavior.

## Documentation Expectations

- Update `README.md` when user-facing commands or philosophy change.
- Update `docs/product.md` for product model or MVP boundary changes.
- Update `docs/architecture.md` for crate structure and design decisions.

## Safety Notes

Koba must never mutate Git history or commit repository changes on behalf of the user. Any future command that writes files should explain what it will change and require explicit approval or an explicit apply flag.
