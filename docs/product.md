# Product Model

> **Koba turns a messy working tree into clean commits.**

Koba is a local-first Git tool. Its primary daily job is to turn an unclear set of working-tree changes into coherent, accurately described commits through a reviewable and conservative flow.

Repository workflow configuration remains useful, but it supports that job rather than defining the entire product. `koba.yml`, hooks, checks, templates, diagnostics, and PR drafting are supporting capabilities.

## Primary Daily Use Case

The intended product flow is:

```text
working tree
    ↓
deterministic analysis
    ↓
coherent commit groups
    ↓
accurate commit descriptions
    ↓
concise preview
    ↓
relevant checks
    ↓
explicit user approval
    ↓
scoped staging
    ↓
commit
```

The user should be able to understand which changes belong together, what each commit says, which checks matter, and exactly what Koba proposes to mutate before approving anything.

## Current Implementation

The current release implements analysis and recommendation, not commit execution:

- `changes` reviews the working tree, plans commit groups, identifies risks, and recommends checks.
- `suggest-commit` returns a deterministic/path-driven Conventional Commit suggestion and scoped manual Git commands.
- Commit messages are still coupled to the current planning heuristics.
- Creating a commit requires manual `git add -- <paths>` and `git commit` commands.
- `koba commit` does not exist yet.
- The current broad CLI also includes `scan`, `doctor`, `init`, `run`, hooks, GitHub PR-template generation, and local PR drafting.

Current file-writing commands preview their target and require `--apply`. Existing files are protected from overwrite.

## Product Direction

Commit execution is a planned product capability, not a permanent boundary violation. A future Koba-native flow may stage approved paths and create an approved commit after exact preview and explicit user approval.

The responsibilities remain separate:

- deterministic analysis owns Git-state understanding, grouping, relevant checks, risk, and mutation safety;
- commit-description generation describes a plan but does not choose its files or authorize mutation; and
- preview, approval, and execution may act only on the approved plan.

Description generation may support multiple implementations later. Deterministic generation remains available; an optional local AI generator may improve prose, but AI must not own grouping, safety, or approval decisions. Hosted AI and remote services are not required for the core workflow.

## Product Safety Invariants

These rules apply both now and after commit execution exists:

- Koba remains local-first.
- Inspection and planning remain non-mutating.
- Every mutation is shown exactly before it runs and requires explicit user approval.
- Approval applies only to the displayed operation; it does not carry forward.
- Staging and commit creation are limited to the approved paths and message.
- The default commit workflow never uses broad staging such as `git add .`.
- The default commit workflow never pushes automatically.
- Koba never silently rebases, resets destructively, force-pushes, or rewrites history.
- Existing user files are not overwritten by generated-file flows.
- Koba does not store GitHub tokens.

## Supporting Capabilities

- Inspect and diagnose repository workflow infrastructure with `scan` and `doctor`.
- Configure and run local checks through `koba.yml` and `run`.
- Preview and install native Git or Husky hook adapters.
- Preview and generate GitHub pull-request templates.
- Draft PR titles and bodies locally.
- Surface repository hygiene and workflow recommendations.

Koba complements Git, Husky, GitHub Actions, GitHub CLI, and language-specific tooling rather than replacing them.

## Roadmap

The commit-first roadmap is tracked as focused issues:

1. separate deterministic commit planning from message generation (#2);
2. add optional diff-aware local description generation without transferring safety decisions to AI (#3);
3. add an interactive, approval-gated commit flow with scoped staging and no automatic push (#4);
4. make normal terminal output concise and decision-first (#5);
5. align the README quickstart with the shipped primary workflow (#6);
6. add end-to-end temporary-repository coverage for planning and mutation safety (#7); and
7. simplify the public CLI surface without silently breaking automation (#8).

Until those issues land, documentation must label their behavior as planned and keep current commands truthful.
