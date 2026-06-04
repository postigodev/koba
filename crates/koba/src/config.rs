use std::{fs, path::Path};

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowConfig {
    pub profile: Option<String>,
    pub commits: Option<CommitConfig>,
    pub checks: Checks,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitConfig {
    pub convention: Option<String>,
    pub require_scope: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Checks {
    pub pre_commit: Vec<Check>,
    pub pre_push: Vec<Check>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    pub name: String,
    pub command: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawWorkflowConfig {
    profile: Option<String>,
    commits: Option<CommitConfig>,
    checks: Option<RawChecks>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawChecks {
    #[serde(default)]
    pre_commit: Vec<RawCheck>,
    #[serde(default)]
    pre_push: Vec<RawCheck>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawCheck {
    Command(String),
    Named {
        name: Option<String>,
        command: String,
    },
}

pub fn load_from_repo(cwd: &Path) -> Result<WorkflowConfig, String> {
    let path = cwd.join("koba.yml");

    if !path.exists() {
        return Err("koba.yml not found".to_owned());
    }

    let contents = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    parse(&contents).map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

pub fn parse(contents: &str) -> Result<WorkflowConfig, serde_yml::Error> {
    let raw: RawWorkflowConfig = serde_yml::from_str(contents)?;
    Ok(WorkflowConfig {
        profile: raw.profile,
        commits: raw.commits,
        checks: raw.checks.map(Checks::from).unwrap_or_default(),
    })
}

impl From<RawChecks> for Checks {
    fn from(raw: RawChecks) -> Self {
        Self {
            pre_commit: raw.pre_commit.into_iter().map(Check::from).collect(),
            pre_push: raw.pre_push.into_iter().map(Check::from).collect(),
        }
    }
}

impl From<RawCheck> for Check {
    fn from(raw: RawCheck) -> Self {
        match raw {
            RawCheck::Command(command) => Self {
                name: command.clone(),
                command,
            },
            RawCheck::Named { name, command } => Self {
                name: name.unwrap_or_else(|| command.clone()),
                command,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_generated_koba_yml() {
        let config = parse(
            r#"
version: 1
profile: rust-cli

commits:
  convention: conventional
  requireScope: true

checks:
  preCommit:
    - cargo fmt --check
  prePush:
    - cargo test
"#,
        )
        .unwrap();

        assert_eq!(config.profile.as_deref(), Some("rust-cli"));
        assert_eq!(config.commits.unwrap().require_scope, Some(true));
        assert_eq!(config.checks.pre_commit[0].command, "cargo fmt --check");
        assert_eq!(config.checks.pre_push[0].command, "cargo test");
    }

    #[test]
    fn parses_named_checks_and_ignores_unknown_fields() {
        let config = parse(
            r#"
version: 1
profile: custom
unknown: still-ok
checks:
  preCommit:
    - name: format
      command: echo format
  prePush: []
"#,
        )
        .unwrap();

        assert_eq!(config.checks.pre_commit[0].name, "format");
        assert_eq!(config.checks.pre_commit[0].command, "echo format");
        assert!(config.checks.pre_push.is_empty());
    }
}
