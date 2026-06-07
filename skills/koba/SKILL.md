---
name: koba
description: Use Koba to inspect Git workflow health, diagnose repository workflow infrastructure, preview checks, prepare surgical Conventional Commits, draft PRs, preview hooks, and review or initialize koba.yml. Trigger for Git workflow review, repository diagnostics, checks, commit preparation, PR preparation, hooks, or koba.yml work. Require explicit approval before --apply, non-dry-run checks, staging, committing, pushing, opening PRs, or writing files.
---

# Koba

Use the globally installed `koba` CLI to inspect and prepare Git workflow changes safely inside a repository. Koba recommendations are advisory; preserve the current user request and repository policy first.

For worked command examples and edge cases, read [references/workflows.md](references/workflows.md) when preparing commits, PRs, hooks, or workflow initialization.

## First Checks

Before using Koba:

1. Inspect repository instructions that exist: `AGENTS.md`, `CLAUDE.md`, `CONTRIBUTING.md`, and `README.md`.
2. Verify Koba is available:

```sh
koba --version
```

If Koba is unavailable, report that clearly, provide installation guidance, and stop. Do not silently install Koba and do not invent Koba output.

Use this precedence:

1. Explicit current user request.
2. Repository-specific safety and contribution instructions.
3. This Koba skill workflow.
4. Koba-generated recommendations.

## Safe Preview Commands

These commands are read-oriented or preview-only and may be run without extra approval:

```sh
koba scan
koba doctor
koba init
koba run pre-commit --dry-run
koba run pre-push --dry-run
koba hooks install --adapter native --dry-run
koba hooks install --adapter husky --dry-run
koba github template pr --dry-run
koba suggest-commit
koba pr --dry-run
```

## Approval Boundaries

Ask for explicit approval before:

- passing `--apply`;
- running non-dry-run configured checks;
- writing `koba.yml`;
- writing hooks;
- writing `.github/` files;
- writing `.koba/pr-body.md`;
- staging files;
- committing;
- pushing;
- opening a pull request.

Approval for one action does not imply approval for later actions. Approval to run checks is not approval to commit. Approval to commit is not approval to push. Approval to write a PR body is not approval to open a PR.

Do not normally perform force pushes, destructive resets, history rewriting, rebases, bypassing failed checks, or overwriting user files. If the user explicitly asks for one of these outside the ordinary Koba workflow, require the request to be unambiguous and follow repository instructions.

## Repository Inspection

Run:

```sh
koba scan
koba doctor
```

Summarize repository status, detected project markers, workflow contract status, hook status, `.github/` infrastructure, and important recommendations. Distinguish essential problems from optional hygiene and irrelevant recommendations. Do not ritualistically apply everything Koba suggests.

## Check Preparation

Run:

```sh
koba run pre-commit --dry-run
koba run pre-push --dry-run
```

Explain which commands would run, which stage each command belongs to, and whether the commands appear scoped and appropriate. Only run actual checks when the user explicitly requested validation or approves after seeing the dry-run. Preserve subprocess output and report failures honestly. Never claim a check passed unless it was executed.

## Surgical Commit Preparation

Run:

```sh
git status --short
koba suggest-commit
```

Then inspect relevant diffs. Determine whether the proposed file grouping is coherent, challenge weak scopes or messages, and prefer one concept per commit.

Show the exact files, proposed Conventional Commit message, and relevant checks already run. Ask before staging or committing. Immediately before staging, rerun `git status --short`; stop if the working tree changed unexpectedly. Stage only approved files, commit only with the approved message, and never push without separate approval.

Koba's deterministic suggestion is evidence, not authority.

## Pull Request Preparation

Run:

```sh
koba pr --dry-run
```

Also inspect the current branch, base branch when known, relevant commit list, and relevant diff. Review and refine the suggested title/body.

Do not invent tests that were not run, impact that was not demonstrated, issue numbers, deployment status, or reviewer decisions. Show the final proposed title and body before writing or opening anything. Ask separately before writing `.koba/pr-body.md`, pushing the branch, or opening a PR.

## Workflow Initialization

Run:

```sh
koba init
```

Review the proposed YAML against the actual repository. Check whether the generated profile and commands make sense. Ask before running `koba init --apply`. Never overwrite an existing `koba.yml`.

## Hook Installation

Always preview first:

```sh
koba hooks install --adapter native --dry-run
koba hooks install --adapter husky --dry-run
```

Explain the selected adapter, target files, commands each hook will execute, and whether another hook system already exists. Ask before applying. Do not blindly mix native hooks and Husky.

## GitHub PR Template Generation

Preview first:

```sh
koba github template pr --dry-run
```

Explain the target file and generated sections. Ask before applying. Do not overwrite an existing template.

## Agent Judgment

Be technically critical rather than mechanically obedient. Identify mixed or unrelated changes, prefer scoped checks before broad test suites, avoid unnecessary repository hygiene files, preserve repository conventions, stop on unexpected state changes, separate observed facts from recommendations, keep approval boundaries explicit, and avoid generic corporate PR language.
