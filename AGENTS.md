# AGENTS.md

## Project

Koba is a local-first Git workflow configurator.

It helps repositories make their development workflow explicit, inspectable, and reproducible: commit conventions, hooks, smoke checks, PR templates, `.github/` infrastructure, branch rules, and repo hygiene.

Koba is not a Git replacement. It is a workflow layer around Git and existing tools such as Husky, GitHub Actions, GitHub CLI, and native Git hooks.

## Working Principles

* Prefer small, coherent changes over broad rewrites.
* Preserve user intent, but challenge weak assumptions when there is a better technical path.
* Be creative in design, but conservative in repository mutation.
* Do not invent product scope silently. If a change expands scope, say so.
* Do not hide tradeoffs. Explain them briefly when they matter.
* Optimize for a serious devtool: predictable behavior, inspectable config, clear errors, and safe defaults.

## Agent Autonomy

Agents may:

* Propose architecture improvements.
* Push back on bad abstractions or premature complexity.
* Suggest simpler implementations.
* Create small supporting docs when they clarify the implementation.
* Refactor touched code when it directly supports the requested change.

Agents must not:

* Commit changes unless explicitly asked.
* Rewrite history, rebase, squash, or force-push.
* Install unrelated dependencies.
* Make large unrelated formatting changes.
* Replace working implementation with speculative architecture.
* Add AI-dependent behavior as a core requirement.
* Store credentials, GitHub tokens, or secrets.

## Tooling Expectations

Use fast local inspection first.

Preferred inspection commands:

* `rg` for searching text.
* `fd` or `find` for locating files.
* `git status --short` before and after edits.
* `git diff --stat` and targeted `git diff` before summarizing changes.

Use Context7 or equivalent documentation lookup when:

* Working with unfamiliar library APIs.
* Updating code that depends on current framework behavior.
* Unsure about current best practices for a dependency.

Use brainstorming/planning skills when:

* Designing new commands or config shape.
* Comparing architecture options.
* Deciding whether a feature belongs in Koba core, an adapter, or later roadmap.

Use koba skill when:

* It's useful to suggest a surgical commit flow using focused commands

Use create-readme skill when:

* You are updating the README.md

## Token Efficiency

* Inspect only files relevant to the requested task.
* Prefer `rg` over opening whole directories.
* Read narrow file ranges when possible.
* Do not paste large unchanged files into responses.
* Summarize command output instead of dumping it unless the exact output matters.
* Avoid re-reading files already inspected unless they changed.

## Testing Policy

Run scoped checks first.

Examples:

* For CLI argument changes, run the CLI tests or the smallest relevant test target.
* For Rust formatting changes, run `cargo fmt --check`.
* For Rust compile-level changes, run `cargo check`.
* For behavior changes, run the relevant tests before broad test suites.
* Run full test suites only when the change is cross-cutting or before suggesting a release/merge.

Do not claim tests passed unless they were actually run.

When reporting, include:

* Commands run.
* Whether each command passed or failed.
* Any failures and whether they are related to the change.

## Git and Repository Safety

Before editing:

```bash
git status --short
```

After editing:

```bash
git status --short
git diff --stat
```

Never commit by default.

If a commit is appropriate, suggest the exact command using Conventional Commits:

```bash
git add <files>
git commit -m "feat(scope): description"
```

Prefer surgical commits:

* one concept per commit
* scoped files
* clear Conventional Commit message
* no bundled unrelated cleanup

## Product Boundaries

Koba should default to recommend-only behavior.

Dangerous or mutating actions should require explicit flags or confirmation, especially:

* installing hooks
* overwriting `.github/` files
* modifying Husky files
* changing branch rules
* opening PRs
* running commands that change repository state

Koba should never store GitHub tokens. It should use existing Git, SSH, Git Credential Manager, or GitHub CLI authentication.

## Design Direction

Core concepts:

* `koba.yml` is the repo workflow contract.
* `scan` inspects the repository and discovers workflow infrastructure.
* `doctor` diagnoses missing or inconsistent workflow pieces.
* `run` executes configured checks.
* `hooks` installs or manages adapters such as native Git hooks or Husky.
* `suggest-commit` proposes Conventional Commit messages and file groupings.
* `pr` prepares PR titles/bodies and may later integrate with GitHub CLI.

Prefer adapters over replacement:

* Husky adapter for JS/TS repositories.
* Native Git hooks adapter for general repositories.
* GitHub CLI adapter for authenticated GitHub operations.
* `.github/` discovery for workflows, PR templates, issue templates, CODEOWNERS, and Dependabot config.

## Response Style

When finished, report:

1. What changed.
2. Files changed.
3. Commands run and results.
4. Important design decisions.
5. Suggested next step.

Optional:
6. When useful, suggest a surgical commit flow using focused commands and the convention type(scope): description. Using the skill koba

Keep summaries concise but specific.
