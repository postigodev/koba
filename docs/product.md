# Product Model

Koba is a local-first workflow configurator for Git repositories. It is like `tweakcn` for Git workflows: instead of configuring UI theme tokens, Koba configures repository workflow infrastructure.

Koba's core object is a repo workflow contract. That contract describes how a repository expects work to move from local changes to reviewed code: commit conventions, hooks, checks, `.github/` files, pull request templates, branch rules, and hygiene rules.

The product flow is discovery -> recommendation -> preview -> explicit apply. Read-only inspection should be easy. File mutation should be deliberate, reviewable, and never surprising.

## Core Use Cases

- Understand the current workflow setup in a repository.
- Detect whether commit conventions, hooks, checks, `.github/` files, PR templates, branch rules, and hygiene rules exist.
- Recommend a small set of next steps to make the workflow safer and more reproducible.
- Surface safe commands for humans and AI coding agents.
- Provide a local `koba.yml` that documents expected checks and conventions.
- Treat `.github/` discovery as a first-class surface for workflows, pull requests, issue templates, and repository automation.

## MVP Boundaries

The MVP should be read-heavy and recommend-only.

In scope:

- CLI command surface.
- Read-only scan output.
- Basic diagnostics for Git workflow files, especially `.github/` assets.
- A lightweight `koba.yml` schema.
- Preview-first apply flows for generated workflow files.
- Integration with existing shell commands.
- Husky detection and recommendations as an adapter path.

Out of scope:

- Rewriting Git history.
- Auto-committing changes.
- Replacing Git, Husky, GitHub Actions, GitHub CLI, or language-specific tooling.
- Storing GitHub tokens. Future GitHub CLI integration should inherit existing `gh` authentication.
- Remote service requirements.
- AI-dependent core behavior.
- Studio-style UI workflows. A future Studio could make contracts easier to inspect and edit, but it is not part of the MVP.

## Product Principles

- Local first: the repository is the source of truth.
- Inspectable: explain findings and recommended changes.
- Reproducible: workflow configuration should be stored in files.
- Conservative: prefer suggestions and previews over mutation.
- Agent-friendly: expose safe, scoped checks and diagnostics.
- Tool-friendly: integrate with the workflow tools users already have.
- Adapter-minded: treat Husky, native Git hooks, GitHub CLI, and `.github/` generation as integration surfaces.
