# Koba

Koba is a local-first Git workflow configurator.

Think `tweakcn` for Git workflows: instead of configuring UI themes, Koba scans and configures repository workflow infrastructure such as commit conventions, hooks, pre-commit and pre-push checks, pull request templates, `.github/` files, and repo hygiene.

Koba is starting small. The first milestone is a serious project foundation for a CLI-first Rust tool, not a finished workflow engine.

## Philosophy

- Make Git workflows explicit, inspectable, and reproducible.
- Integrate with Git, Husky, GitHub Actions, GitHub CLI, and existing project tools instead of replacing them.
- Default to recommend-only behavior.
- Never mutate history or commit changes without explicit user approval.
- Prefer safe, scoped commands that are useful to humans and AI coding agents.
- Keep AI optional. Koba should be valuable as a normal local CLI.

## Planned Commands

```text
koba init
koba scan
koba doctor
koba run
koba hooks
koba suggest-commit
koba pr
```

- `init`: create or update a repository-local `koba.yml`.
- `scan`: inspect workflow infrastructure and report what exists.
- `doctor`: diagnose missing, conflicting, or risky workflow setup.
- `run`: execute named checks from `koba.yml`.
- `hooks`: inspect and eventually install or sync Git hook configuration.
- `suggest-commit`: suggest a safe commit command from staged changes.
- `pr`: inspect or prepare pull request workflow assets.

## Non-Goals

- Koba is not a replacement for Git.
- Koba is not a replacement for Husky, pre-commit, GitHub Actions, or GitHub CLI.
- Koba will not mutate Git history.
- Koba will not auto-commit or push changes.
- Koba will not require AI services for core workflows.

## Early Roadmap

1. Establish the CLI surface and repository documentation.
2. Add read-only repository scanning for Git metadata and workflow files.
3. Define the first `koba.yml` schema for checks, hooks, and conventions.
4. Implement `doctor` diagnostics with recommend-only output.
5. Add explicit apply flows for safe file generation.
6. Explore optional AI-assisted layers after the local workflow model is useful on its own.

## Development

Run the scoped checks for the current foundation:

```sh
cargo test -p koba
```
