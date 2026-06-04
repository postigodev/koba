# Koba

**Local-first Git workflow configuration for real repositories.**

Koba makes repository workflows explicit: commit conventions, hooks, smoke checks, PR templates, `.github/` infrastructure, branch rules, and repo hygiene.

Think of it like a workflow configurator for Git repos. Instead of manually spreading rules across README files, package scripts, Husky hooks, CI YAML, PR templates, and team habits, Koba gives the repo a visible workflow contract.

## Why

Git is powerful, but real Git workflows usually live across scattered places:

* `package.json` scripts
* `.husky/`
* `.git/hooks/`
* `.github/workflows/`
* `.github/pull_request_template.md`
* branch naming conventions
* commit message conventions
* contributor habits
* maintainer expectations
* AI agent instructions

Koba turns those informal rules into something inspectable and reproducible.

## Philosophy

Koba does not replace Git.

Koba does not replace Husky.

Koba does not replace GitHub Actions.

Koba does not commit for you by default.

Koba is a local workflow layer that helps developers and coding agents understand what should happen before commits, pushes, and pull requests.

Default behavior should be safe, explicit, and recommend-first.

## Planned Commands

```bash
koba init
koba scan
koba doctor
koba run pre-commit
koba run pre-push
koba hooks install
koba suggest-commit
koba pr
```

## Example Config

```yaml
project:
  name: koba
  profile: rust-cli

commit:
  convention: conventional
  requireScope: true
  allowedTypes:
    - feat
    - fix
    - docs
    - chore
    - refactor
    - test

hooks:
  adapter: native
  mode: recommend

checks:
  preCommit:
    - name: format
      run: cargo fmt --check
    - name: test
      run: cargo test

  prePush:
    - name: clippy
      run: cargo clippy -- -D warnings

github:
  discover: true
  prTemplate:
    sections:
      - summary
      - changes
      - checks
      - risk
      - reviewer-notes
```

## MVP

The first version of Koba focuses on:

* Rust CLI foundation
* repository scanning
* `.github/` discovery
* `koba.yml` workflow config
* native Git hook adapter
* Husky adapter
* scoped check execution
* Conventional Commit suggestions
* PR template generation

## Non-goals

* Replacing Git
* Replacing CI
* Storing GitHub credentials
* Auto-committing changes
* Hiding dangerous mutations behind magic automation
* Depending on AI for core behavior

## Status

Early development.
