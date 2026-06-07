# Koba Agent Skill

The Koba Agent Skill teaches coding agents how to use the globally installed `koba` CLI safely inside any Git repository. It is an instruction package, not a new Koba command and not an MCP server.

## Relationship To Other Agent Files

- Koba CLI: the local executable that scans repositories, previews workflow files, runs configured checks, and drafts commit or PR text.
- Koba Agent Skill: portable agent instructions in `skills/koba/` for using the CLI safely.
- `AGENTS.md` / `CLAUDE.md`: repository-specific instructions. These take precedence over Koba recommendations.
- Future MCP server: possible later integration surface. This skill does not add MCP behavior.

Use this precedence:

1. Explicit current user request.
2. Repository-specific safety and contribution instructions.
3. Koba Agent Skill workflow.
4. Koba-generated recommendations.

## Install With The Skills CLI

Koba is discoverable on skills.sh at:

```text
https://skills.sh/postigodev/koba
```

List available skills without installing:

```sh
npx skills add postigodev/koba --list
```

Install globally for Codex and Claude Code:

```sh
npx skills add postigodev/koba \
  --skill koba \
  --global \
  --agent codex \
  --agent claude-code \
  --yes
```

PowerShell-friendly one-line equivalent:

```powershell
npx skills add postigodev/koba --skill koba --global --agent codex --agent claude-code --yes
```

List installed global skills:

```sh
npx skills list --global
```

Update:

```sh
npx skills update koba
```

Remove:

```sh
npx skills remove --global koba
```

The `skills` CLI handles installation into agent-specific locations. Do not manually duplicate the canonical skill into `.agents/skills/` or `.claude/skills/` in this repository.

## Invocation

Agents may use the skill implicitly when a request matches the skill description, or explicitly when the user names it.

Example prompt:

```text
Prepare a surgical commit using Koba, but do not stage or commit until I approve.
```

## Approval Boundaries

The skill allows read-oriented and preview-only Koba commands without extra approval:

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

Agents must ask before `--apply`, non-dry-run configured checks, staging, committing, pushing, opening pull requests, or writing `koba.yml`, hooks, `.github/` files, or `.koba/pr-body.md`.

Approval for one action does not imply approval for later actions.

## Troubleshooting

The skill must first run:

```sh
koba --version
```

If that fails, Koba is not available on `PATH`. Install Koba first, for example with Scoop on Windows:

```powershell
scoop bucket add postigodev https://github.com/postigodev/scoop-bucket
scoop install koba
```

Or install from a checked-out Koba workspace for local development:

```sh
cargo install --path crates/koba
```

Do not let an agent silently install Koba or pretend Koba output exists.

## Trigger Matrix

Should trigger:

- "Review this repository's Git workflow."
- "Prepare a surgical commit using Koba."
- "Check what will run before I push."
- "Draft a PR from my current branch using Koba."
- "Set up Koba hooks, but preview everything first."
- "Generate a workflow contract for this repository."

Should not automatically trigger:

- "Explain what a Git commit is."
- "Rewrite this README paragraph."
- "Resolve this merge conflict."
- "Force-push this branch."
- "Install an unrelated dependency."
- "What is the weather?"

## skills.sh Discoverability

Koba is published and discoverable through skills.sh and the `skills` CLI.

Repository page:

```text
https://skills.sh/postigodev/koba
```

Official badge:

```markdown
[![skills.sh](https://skills.sh/b/postigodev/koba)](https://skills.sh/postigodev/koba)
```

Remote discovery:

```sh
npx skills add postigodev/koba --list
```

Install:

```sh
npx skills add postigodev/koba --skill koba --global --agent codex --agent claude-code --yes
```

The repository README now includes the official skills.sh badge. The skills.sh docs describe ranking through anonymous install telemetry from the `skills` CLI.

When adding new skills later, repeat remote discovery before documenting them as available.

## Verified CLI Notes

Verified with `skills` CLI `1.5.10`:

- `npx skills add <source> --list`
- `npx skills add <source> --skill <name> --agent <agent> --yes`
- `--global`
- `--agent codex`
- `--agent claude-code`
- `npx skills list --global`
- `npx skills update <skill>`
- `npx skills remove --global <skill>`

The official docs and CLI help support installing from a GitHub repository source such as `postigodev/koba`. Direct GitHub tree-path installation is not documented here; prefer repository source plus `--skill koba`.
