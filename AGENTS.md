# AGENTS.md

## Project

Koba is a local-first Git tool whose primary product thesis is:

> **Koba turns a messy working tree into clean commits.**

The current release analyzes working trees, plans commit groups, suggests deterministic/path-driven commit messages, and supports repository workflow infrastructure. Commit execution is still manual, and `koba commit` does not exist yet.

The intended daily flow is deterministic analysis -> coherent commit groups -> accurate descriptions -> concise preview -> relevant checks -> explicit user approval -> scoped staging and commit creation. Koba remains a workflow layer around Git rather than a Git replacement.

Supporting capabilities include configured checks, `koba.yml`, hooks, PR templates, `.github/` discovery, branch rules, and repository hygiene.

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

* Stage, commit, push, rebase, squash, rewrite history, or force-push
  unless the user's request explicitly authorizes that exact action.
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

## Contributor-Agent Git and Repository Safety

This section governs an agent's own actions while contributing to Koba. It is distinct from the Koba product's approval-gated mutation model.

Before editing:

```bash
git status --short
```

After editing:

```bash
git status --short
git diff --stat
```

File edits are allowed only when the user's request authorizes them. Otherwise ask before applying writes. Never stage, commit, push, rebase, squash, rewrite history, or otherwise mutate repository history unless the user's request explicitly authorizes that exact action. Approval for one action does not authorize another.

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

Koba is local-first and conservative about mutation. The current release remains recommend-only for commit execution. The product direction permits narrowly scoped Git mutation only after showing the exact proposed action and receiving explicit user approval.

Product safety invariants:

* never stage or commit silently
* preview the exact paths and final commit message before mutation
* limit staging and commit creation to the approved plan
* never use broad staging such as `git add .` in the default commit workflow
* never push automatically as part of the default commit workflow
* never silently rebase, reset destructively, force-push, or rewrite history
* treat approval for each operation independently
* keep deterministic code responsible for Git-state analysis, grouping, checks, risk, and mutation safety

Optional AI-backed description generation may be added later, but AI must not own grouping, mutation safety, or approval decisions.

Other dangerous or mutating actions should also require an exact preview and explicit approval, especially:

* installing hooks
* overwriting `.github/` files
* modifying Husky files
* changing branch rules
* opening PRs
* running commands that change repository state

Koba should never store GitHub tokens. It should use existing Git, SSH, Git Credential Manager, or GitHub CLI authentication.

## Design Direction

The intended commit-first responsibility boundaries are:

1. deterministic analysis and planning own Git-state understanding, grouping, relevant checks, risk, and safety;
2. commit-description generation describes an existing plan and may support multiple generators later; and
3. preview, approval, and execution may stage and commit only the approved plan.

Current and supporting capabilities:

* `changes` analyzes the current working tree and recommends commit groups and checks.
* `suggest-commit` currently proposes deterministic/path-driven Conventional Commit messages and scoped Git commands.
* `scan` and `doctor` inspect and diagnose repository workflow infrastructure.
* `koba.yml` supports configured checks and workflow behavior; it is not the entire product model.
* `run` executes configured checks.
* `hooks` manages adapters such as native Git hooks or Husky.
* `github template pr` and `pr` support pull-request infrastructure and drafting.

Do not document `koba commit`, AI-backed generation, or a simplified CLI as implemented until the corresponding source exists.

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
