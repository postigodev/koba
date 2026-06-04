use std::{fmt::Write, path::PathBuf};

use clap::ValueEnum;

use crate::{
    config::{self, Check, WorkflowConfig},
    executor,
    output::{self, Status},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum Stage {
    PreCommit,
    PrePush,
}

impl Stage {
    fn label(&self) -> &'static str {
        match self {
            Stage::PreCommit => "pre-commit",
            Stage::PrePush => "pre-push",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunOptions {
    pub stage: Stage,
    pub dry_run: bool,
}

pub fn run(cwd: PathBuf, options: RunOptions) -> Result<(), String> {
    match execute(cwd, options) {
        Ok(output) => {
            print!("{output}");
            Ok(())
        }
        Err(error) => {
            println!("Koba run");
            println!();
            println!("{}", output::line(Status::Missing, &error));
            if error.starts_with("koba.yml not found") {
                println!(
                    "{}",
                    output::line(
                        Status::Step,
                        "Run `koba init` to create a workflow contract"
                    )
                );
            }
            Err(error)
        }
    }
}

pub fn execute(cwd: PathBuf, options: RunOptions) -> Result<String, String> {
    let config = config::load_from_repo(&cwd)?;
    let checks = checks_for_stage(&config, options.stage);
    let mut output = render_header(options.stage, options.dry_run);

    if checks.is_empty() {
        writeln!(
            output,
            "{}",
            output::line(
                Status::Ok,
                format!("No checks configured for {}", options.stage.label())
            )
        )
        .unwrap();
        return Ok(output);
    }

    for check in checks {
        writeln!(output, "{}", render_check_line(check)).unwrap();

        if !options.dry_run {
            executor::run_shell(&cwd, &check.command)?;
        }
    }

    if options.dry_run {
        writeln!(
            output,
            "{}",
            output::line(Status::Step, "Dry run only; no checks were executed")
        )
        .unwrap();
    } else {
        writeln!(output, "{}", output::line(Status::Ok, "All checks passed")).unwrap();
    }

    Ok(output)
}

pub fn checks_for_stage(config: &WorkflowConfig, stage: Stage) -> &[Check] {
    match stage {
        Stage::PreCommit => &config.checks.pre_commit,
        Stage::PrePush => &config.checks.pre_push,
    }
}

fn render_header(stage: Stage, dry_run: bool) -> String {
    let mut output = String::new();
    writeln!(output, "Koba run {}", stage.label()).unwrap();
    if dry_run {
        writeln!(output, "{}", output::line(Status::Step, "Dry run")).unwrap();
    }
    writeln!(output).unwrap();
    output
}

fn render_check_line(check: &Check) -> String {
    output::line(Status::Step, format!("{}: {}", check.name, check.command))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn selects_checks_for_stage() {
        let config = config::parse(
            r#"
checks:
  preCommit:
    - echo commit
  prePush:
    - echo push
"#,
        )
        .unwrap();

        assert_eq!(
            checks_for_stage(&config, Stage::PreCommit)[0].command,
            "echo commit"
        );
        assert_eq!(
            checks_for_stage(&config, Stage::PrePush)[0].command,
            "echo push"
        );
    }

    #[test]
    fn dry_run_lists_checks_without_executing() {
        let fixture = TempTree::new();
        let marker = fixture.path().join("marker.txt");
        fs::write(
            fixture.path().join("koba.yml"),
            r#"
checks:
  preCommit:
    - echo changed > marker.txt
  prePush: []
"#,
        )
        .unwrap();

        let output = execute(
            fixture.path().to_path_buf(),
            RunOptions {
                stage: Stage::PreCommit,
                dry_run: true,
            },
        )
        .unwrap();

        assert!(output.contains("Dry run"));
        assert!(!marker.exists());
    }

    #[test]
    fn no_checks_for_stage_exits_successfully() {
        let fixture = TempTree::new();
        fs::write(
            fixture.path().join("koba.yml"),
            r#"
checks:
  preCommit: []
  prePush: []
"#,
        )
        .unwrap();

        let output = execute(
            fixture.path().to_path_buf(),
            RunOptions {
                stage: Stage::PrePush,
                dry_run: false,
            },
        )
        .unwrap();

        assert!(output.contains("No checks configured for pre-push"));
    }

    #[test]
    fn failing_check_returns_error() {
        let fixture = TempTree::new();
        fs::write(
            fixture.path().join("koba.yml"),
            r#"
checks:
  preCommit:
    - exit 1
  prePush: []
"#,
        )
        .unwrap();

        let error = execute(
            fixture.path().to_path_buf(),
            RunOptions {
                stage: Stage::PreCommit,
                dry_run: false,
            },
        )
        .unwrap_err();

        assert!(error.contains("check command failed"));
    }

    struct TempTree {
        path: PathBuf,
    }

    impl TempTree {
        fn new() -> Self {
            let id = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!("koba-run-test-{id}"));
            fs::create_dir(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
