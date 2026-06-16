# Koba Workflow Examples

Use these examples as patterns, not scripts to follow blindly. Preserve repository instructions and the user's current request.

## 1. Inspect Only

```sh
koba --version
koba scan
koba doctor
```

Report what exists, what is missing, and which recommendations matter. Do not apply generated files just because Koba recommends them.

Decision points:

- Is the repository actually a Git repo?
- Is `koba.yml` missing or present?
- Are missing `.github/` files relevant for this repository?

## 2. Prepare A Commit Without Executing It

```sh
koba changes
git status --short
koba suggest-commit
git diff -- <approved-file>
```

Use `koba changes` for the broad working-tree plan: changed-file counts, likely commit groups, mixed-change warnings, and relevant checks. Use `koba suggest-commit` for the focused Conventional Commit suggestion. Show the changed files, proposed Conventional Commit message, and why the grouping is or is not coherent. Stop before staging.

Refuse to stage if the user only asked for a suggestion.

## 3. Validate, Obtain Approval, And Commit

```sh
koba changes
```

Review the recommended checks before executing anything.

```sh
koba run pre-commit --dry-run
koba run pre-push --dry-run
```

After approval to validate:

```sh
koba run pre-commit
```

If checks pass and the user approves the commit:

```sh
git status --short
git add -- <approved-files>
git commit -m "<approved-message>"
```

Approval to run checks is not approval to commit. Approval to commit is not approval to push.

If the working tree changes between approval and staging, stop and ask.

## 4. Prepare A PR Without Opening It

```sh
koba pr --dry-run
git branch --show-current
git log --oneline --decorate -n 20
git diff --stat
```

Review the suggested title and body. Remove claims about tests, deployment, screenshots, or issue numbers unless verified. Ask separately before writing `.koba/pr-body.md`, pushing, or opening a PR.

## 5. Initialize `koba.yml`

```sh
koba init
```

Review the detected profile and checks. If the profile or commands are wrong, explain the mismatch and do not apply. Ask before:

```sh
koba init --apply
```

Refuse to overwrite an existing `koba.yml`.

## 6. Preview And Install Native Hooks

```sh
koba scan
koba hooks install --adapter native --dry-run
```

Explain target files such as `.git/hooks/pre-commit` and `.git/hooks/pre-push`, and that they call `koba run pre-commit` and `koba run pre-push`.

Ask before:

```sh
koba hooks install --adapter native --apply
```

Do not install if existing hooks need manual review.

## 7. Preview And Install Husky Hooks

```sh
koba scan
koba hooks install --adapter husky --dry-run
```

Use Husky when the repository already uses or clearly prefers Husky. Do not modify `package.json` or install npm packages unless the user separately approves that non-Koba work.

Ask before:

```sh
koba hooks install --adapter husky --apply
```

Do not blindly mix Husky and native Git hooks.

## 8. Missing Koba Executable

```sh
koba --version
```

If this fails, report that Koba is not available on `PATH`. Provide installation options from the Koba README, such as Scoop on Windows or `cargo install --path crates/koba` for local development. Do not install Koba silently.

## 9. Koba Source Workspace Fallback

When `koba --version` fails but the current repository is the Koba source workspace:

```sh
test -f crates/koba/Cargo.toml
cargo run -q -p koba -- scan
cargo run -q -p koba -- doctor
```

State that you are using the workspace binary fallback because global `koba` is unavailable. Do not use this fallback in unrelated repositories.

## 10. Failed Configured Check

```sh
koba run pre-commit
```

If a check fails, preserve the subprocess output, report the failing command and exit status, and stop. Do not bypass the failure, commit anyway, or claim validation passed.

## 11. Suggestion Spans Unrelated Files

```sh
koba changes
git status --short
koba suggest-commit
git diff --stat
```

If `koba changes` reports multiple groups, treat that as the default commit boundary. If `koba suggest-commit` suggests one commit for unrelated docs, source, packaging, or generated files, challenge it. Propose separate commits and ask which grouping the user wants.

Refuse to stage unrelated files under a single message unless the user explicitly approves that grouping.

## 12. Clean Docs-Only Working Tree

```sh
koba changes
git status --short
koba suggest-commit
```

Expected interpretation:

- One docs-oriented group, such as `docs(agents): update agent documentation`.
- `git diff --check` is relevant.
- Rust, Node, or Python test suites are not automatically required for docs-only changes.

Still inspect the diff before recommending a commit. Do not stage until the user approves the exact files and message.

## 13. Mixed Skill Docs And Commit Engine Source

```sh
koba changes
git diff -- skills/koba/SKILL.md skills/koba/references/workflows.md
git diff -- crates/koba/src/suggest_commit.rs
koba suggest-commit
```

Expected interpretation:

- Skill docs form one group, such as `docs(skill): document workspace binary fallback`.
- Commit-suggestion source forms another group, such as `feat(commit): sharpen path-based scope inference`.
- The working tree should be treated as mixed unless the diffs clearly prove one coherent change.

Do not collapse the groups into one commit just because a single message is convenient.

## 14. Rust Source Change With Recommended Checks

```sh
koba changes
koba run pre-commit --dry-run
koba run pre-push --dry-run
```

Expected interpretation:

- Rust source changes should usually recommend `cargo fmt --check`, `cargo check`, and `cargo test`, with package scoping when Koba can detect the workspace package.
- The dry-run output shows configured Koba checks, which may differ from the heuristic recommendations.

Ask before running non-dry-run checks. Preserve output and stop on failure.

## 15. `koba changes` Unavailable In Old Koba

```sh
koba --version
koba changes
```

If `koba changes` is not recognized, report that the installed Koba version appears older than the skill workflow expects. Do not pretend the command ran.

In the Koba source workspace only, and only when `crates/koba/Cargo.toml` exists, you may state that you are using the workspace binary fallback:

```sh
cargo run -q -p koba -- changes
```

Do not use this fallback in unrelated repositories. In other repositories, ask the user to update Koba before using the `changes` workflow, or fall back to manual `git status --short` and diff inspection without attributing that work to Koba.
