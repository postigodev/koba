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
git status --short
koba suggest-commit
git diff -- <approved-file>
```

Show the changed files, proposed Conventional Commit message, and why the grouping is or is not coherent. Stop before staging.

Refuse to stage if the user only asked for a suggestion.

## 3. Validate, Obtain Approval, And Commit

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
git status --short
koba suggest-commit
git diff --stat
```

If Koba suggests one commit for unrelated docs, source, packaging, or generated files, challenge it. Propose separate commits and ask which grouping the user wants.

Refuse to stage unrelated files under a single message unless the user explicitly approves that grouping.
