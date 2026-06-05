use std::{
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

use clap::Subcommand;

use crate::output::{self, Status};

#[derive(Debug, Clone, Subcommand)]
pub enum GithubCommand {
    /// Preview or generate GitHub templates.
    Template {
        #[command(subcommand)]
        command: TemplateCommand,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum TemplateCommand {
    /// Preview or generate .github/pull_request_template.md.
    Pr {
        /// Preview without writing files.
        #[arg(long)]
        dry_run: bool,
        /// Write the template if it does not already exist.
        #[arg(long)]
        apply: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemplateOptions {
    pub dry_run: bool,
    pub apply: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplatePlan {
    pub path: PathBuf,
    pub contents: String,
    pub exists: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateOutcome {
    Preview(TemplatePlan),
    Applied(TemplatePlan),
}

pub fn run(cwd: PathBuf, command: GithubCommand) -> Result<(), String> {
    let options = match command {
        GithubCommand::Template { command } => match command {
            TemplateCommand::Pr { dry_run, apply } => TemplateOptions { dry_run, apply },
        },
    };

    match execute_pr_template(&cwd, options) {
        Ok(outcome) => {
            print!("{}", render_outcome(&outcome));
            Ok(())
        }
        Err(error) => {
            println!("Koba github template pr");
            println!();
            println!("{}", output::line(Status::Missing, &error));
            Err(error)
        }
    }
}

pub fn execute_pr_template(
    cwd: &Path,
    options: TemplateOptions,
) -> Result<TemplateOutcome, String> {
    if options.dry_run && options.apply {
        return Err("choose either --dry-run or --apply, not both".to_owned());
    }

    let plan = build_pr_template_plan(cwd);

    if !options.apply {
        return Ok(TemplateOutcome::Preview(plan));
    }

    if !plan.exists {
        if let Some(parent) = plan.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }

        fs::write(&plan.path, &plan.contents)
            .map_err(|error| format!("failed to write {}: {error}", plan.path.display()))?;
    }

    Ok(TemplateOutcome::Applied(plan))
}

pub fn build_pr_template_plan(cwd: &Path) -> TemplatePlan {
    let path = cwd.join(".github").join("pull_request_template.md");

    TemplatePlan {
        exists: path.exists(),
        path,
        contents: pr_template_contents(),
    }
}

fn pr_template_contents() -> String {
    [
        "## Summary",
        "",
        "What changed and why?",
        "",
        "## Changes",
        "",
        "- ",
        "",
        "## Checks run",
        "",
        "- [ ] Tests",
        "- [ ] Formatting/linting",
        "",
        "## Risk / rollback",
        "",
        "What could go wrong, and how should this be rolled back?",
        "",
        "## Screenshots or demo",
        "",
        "Add screenshots, recordings, or notes when the change affects UI or behavior.",
        "",
        "## Notes for reviewer",
        "",
        "Anything specific to focus on?",
        "",
    ]
    .join("\n")
}

fn render_outcome(outcome: &TemplateOutcome) -> String {
    let mut output = String::new();
    let (plan, applied) = match outcome {
        TemplateOutcome::Preview(plan) => (plan, false),
        TemplateOutcome::Applied(plan) => (plan, true),
    };

    writeln!(output, "Koba github template pr").unwrap();
    writeln!(output).unwrap();

    if plan.exists {
        writeln!(
            output,
            "{}",
            output::line(
                Status::Warning,
                format!(
                    "{} already exists; refusing to overwrite",
                    plan.path.display()
                )
            )
        )
        .unwrap();
        return output;
    }

    if applied {
        writeln!(
            output,
            "{}",
            output::line(Status::Ok, format!("Wrote {}", plan.path.display()))
        )
        .unwrap();
    } else {
        writeln!(
            output,
            "{}",
            output::line(Status::Step, "Preview only; no files were written")
        )
        .unwrap();
        writeln!(
            output,
            "{}",
            output::line(Status::Step, format!("Would write {}", plan.path.display()))
        )
        .unwrap();
    }

    writeln!(output).unwrap();
    writeln!(output, "Contents").unwrap();
    writeln!(output, "{}", indent_contents(&plan.contents)).unwrap();

    output
}

fn indent_contents(contents: &str) -> String {
    contents
        .lines()
        .map(|line| format!("    {line}"))
        .collect::<Vec<_>>()
        .join("\n")
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
    fn preview_does_not_write_pr_template() {
        let fixture = TempTree::new();

        let outcome = execute_pr_template(
            fixture.path(),
            TemplateOptions {
                dry_run: false,
                apply: false,
            },
        )
        .unwrap();

        assert!(matches!(outcome, TemplateOutcome::Preview(_)));
        assert!(!fixture
            .path()
            .join(".github/pull_request_template.md")
            .exists());
    }

    #[test]
    fn apply_creates_github_dir_and_pr_template() {
        let fixture = TempTree::new();

        execute_pr_template(
            fixture.path(),
            TemplateOptions {
                dry_run: false,
                apply: true,
            },
        )
        .unwrap();

        let contents =
            fs::read_to_string(fixture.path().join(".github/pull_request_template.md")).unwrap();
        assert!(contents.contains("## Summary"));
        assert!(contents.contains("## Notes for reviewer"));
    }

    #[test]
    fn apply_does_not_overwrite_existing_pr_template() {
        let fixture = TempTree::new();
        fixture.file(".github/pull_request_template.md", "existing\n");

        let outcome = execute_pr_template(
            fixture.path(),
            TemplateOptions {
                dry_run: false,
                apply: true,
            },
        )
        .unwrap();
        let contents =
            fs::read_to_string(fixture.path().join(".github/pull_request_template.md")).unwrap();

        assert!(matches!(outcome, TemplateOutcome::Applied(plan) if plan.exists));
        assert_eq!(contents, "existing\n");
    }

    #[test]
    fn dry_run_and_apply_are_rejected_together() {
        let fixture = TempTree::new();

        let error = execute_pr_template(
            fixture.path(),
            TemplateOptions {
                dry_run: true,
                apply: true,
            },
        )
        .unwrap_err();

        assert_eq!(error, "choose either --dry-run or --apply, not both");
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
            let path = std::env::temp_dir().join(format!("koba-github-test-{id}"));
            fs::create_dir(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }

        fn file(&self, relative: &str, contents: &str) {
            let path = self.path.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, contents).unwrap();
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
