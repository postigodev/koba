# Product Model

Koba is a local-first workflow configurator for Git repositories. It helps teams and solo developers understand, document, and improve the workflow infrastructure around a repository.

The product should feel like a careful repo mechanic: it looks at what exists, explains the current state, recommends improvements, and only applies changes when the user asks.

## Core Use Cases

- Understand the current workflow setup in a repository.
- Detect whether commit conventions, hooks, checks, PR templates, and GitHub workflow files exist.
- Recommend a small set of next steps to make the workflow safer and more reproducible.
- Surface safe commands for humans and AI coding agents.
- Provide a local `koba.yml` that documents expected checks and conventions.

## MVP Boundaries

The MVP should be read-heavy and recommend-only.

In scope:

- CLI command surface.
- Read-only scan output.
- Basic diagnostics for common workflow files.
- A lightweight `koba.yml` schema.
- Explicit apply flows for generated workflow files.
- Integration with existing shell commands.

Out of scope:

- Rewriting Git history.
- Auto-committing changes.
- Replacing Git, Husky, GitHub Actions, GitHub CLI, or language-specific tooling.
- Remote service requirements.
- AI-dependent core behavior.

## Product Principles

- Local first: the repository is the source of truth.
- Inspectable: explain findings and recommended changes.
- Reproducible: workflow configuration should be stored in files.
- Conservative: prefer suggestions over mutation.
- Agent-friendly: expose safe, scoped checks and diagnostics.
- Tool-friendly: integrate with the workflow tools users already have.
