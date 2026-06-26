use std::{fmt::Write, path::PathBuf};

use crate::{
    analysis::{self, CheckRecommendation, CommitPlan, Risk, WorkingTreeAnalysis},
    git,
    output::{self, Status, StatusRow},
};

pub fn run(cwd: PathBuf) -> Result<(), String> {
    match execute(cwd) {
        Ok(output) => {
            print!("{output}");
            Ok(())
        }
        Err(error) => {
            println!("Koba changes");
            println!();
            println!("{}", output::line(Status::Error, &error));
            Err(error)
        }
    }
}

pub fn execute(cwd: PathBuf) -> Result<String, String> {
    let info = git::inspect(&cwd);
    if !info.inside_repo {
        return Err("not inside a Git repository".to_owned());
    }

    let report = analysis::analyze_cwd(&cwd)?;
    Ok(render(&report))
}

fn render(report: &WorkingTreeAnalysis) -> String {
    let mut output = String::new();

    writeln!(output, "Koba changes").unwrap();
    writeln!(output).unwrap();

    output::section(
        &mut output,
        "Working tree",
        &[
            output::row(status_for_changed_count(report), "changed files")
                .value(report.changed_count.to_string()),
            output::row(status_for_count(report.staged_count), "staged files")
                .value(report.staged_count.to_string()),
            output::row(status_for_count(report.unstaged_count), "unstaged files")
                .value(report.unstaged_count.to_string()),
            output::row(status_for_count(report.untracked_count), "untracked files")
                .value(report.untracked_count.to_string()),
            output::row(tree_state_status(report), "state").value(tree_state_label(report)),
        ],
    );

    if !report.commit_plans.is_empty() {
        writeln!(output).unwrap();
        if report.commit_plans.len() == 1 {
            output::section(
                &mut output,
                "Recommended commit",
                &plan_rows(&report.commit_plans),
            );
        } else {
            output::section(
                &mut output,
                "Commit groups",
                &plan_rows(&report.commit_plans),
            );
        }
    }

    if !report.check_recommendations.is_empty() {
        writeln!(output).unwrap();
        output::section(
            &mut output,
            "Checks",
            &check_rows(&report.check_recommendations),
        );
    }

    writeln!(output).unwrap();
    output::section(&mut output, "Risk", &risk_rows(&report.risks));

    writeln!(output).unwrap();
    writeln!(output, "Next steps").unwrap();
    if report.is_clean {
        writeln!(
            output,
            "{}",
            output::next_step("No commit preparation needed")
        )
        .unwrap();
    } else if report.commit_plans.len() == 1 {
        writeln!(output, "{}", output::next_step("Review the diff")).unwrap();
        writeln!(output, "{}", output::next_step("Run relevant checks")).unwrap();
        writeln!(
            output,
            "{}",
            output::next_step("Stage only the approved files")
        )
        .unwrap();
    } else {
        writeln!(output, "{}", output::next_step("Inspect each group diff")).unwrap();
        writeln!(output, "{}", output::next_step("Run relevant checks")).unwrap();
        writeln!(
            output,
            "{}",
            output::next_step("Stage one approved group at a time")
        )
        .unwrap();
    }

    output
}

fn plan_rows(plans: &[CommitPlan]) -> Vec<StatusRow> {
    plans
        .iter()
        .map(|plan| {
            let mut row = output::row(Status::Plan, &plan.message);
            for file in &plan.files {
                row = row.detail(file);
            }
            row = row.detail(format!(
                "confidence: {}",
                analysis::confidence_label(plan.confidence)
            ));
            for reason in &plan.reasons {
                row = row.detail(format!("reason: {reason}"));
            }
            row = row.detail(plan.git_add_command());
            row = row.detail(plan.git_commit_command());
            for warning in &plan.warnings {
                row = row.detail(format!("warning: {warning}"));
            }
            row
        })
        .collect()
}

fn check_rows(checks: &[CheckRecommendation]) -> Vec<StatusRow> {
    checks
        .iter()
        .map(|check| {
            output::row(Status::Plan, &check.command).detail(format!("reason: {}", check.reason))
        })
        .collect()
}

fn risk_rows(risks: &[Risk]) -> Vec<StatusRow> {
    risks
        .iter()
        .map(|risk| output::row(risk.status, &risk.message))
        .collect()
}

fn status_for_changed_count(report: &WorkingTreeAnalysis) -> Status {
    if report.is_clean {
        Status::Ok
    } else if report.commit_plans.len() > 1 {
        Status::Warn
    } else {
        Status::Ok
    }
}

fn status_for_count(count: usize) -> Status {
    if count == 0 {
        Status::Ok
    } else {
        Status::Warn
    }
}

fn tree_state_status(report: &WorkingTreeAnalysis) -> Status {
    if report.is_clean || report.commit_plans.len() <= 1 {
        Status::Ok
    } else {
        Status::Warn
    }
}

fn tree_state_label(report: &WorkingTreeAnalysis) -> &'static str {
    if report.is_clean {
        "clean"
    } else if report.commit_plans.len() <= 1 {
        "coherent"
    } else {
        "mixed"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::{Path, PathBuf},
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn clean_tree_has_no_commit_plan() {
        let fixture = TempTree::new();
        fixture.git_init();

        let output = execute(fixture.path().to_path_buf()).unwrap();

        assert!(output.contains("changed files"));
        assert!(output.contains("working tree is clean"));
        assert!(!output.contains("Recommended commit"));
        assert!(!output.contains("Commit groups"));
    }

    #[test]
    fn non_git_directory_errors() {
        let fixture = TempTree::new();

        let error = execute(fixture.path().to_path_buf()).unwrap_err();

        assert_eq!(error, "not inside a Git repository");
    }

    #[test]
    fn coherent_tree_renders_recommended_commit() {
        let report = analysis::analyze(
            Path::new("."),
            vec![analysis::WorkingTreeFile::from_status(
                " M",
                "crates/koba/src/output.rs",
            )],
        );
        let output = render(&report);

        assert!(output.contains("Recommended commit"));
        assert!(!output.contains("Commit groups"));
        assert!(output.contains("feat(output): improve terminal rendering"));
        assert!(output.contains("git add -- \"crates/koba/src/output.rs\""));
    }

    #[test]
    fn mixed_tree_renders_commit_groups() {
        let report = analysis::analyze(
            Path::new("."),
            vec![
                analysis::WorkingTreeFile::from_status(" M", "skills/koba/SKILL.md"),
                analysis::WorkingTreeFile::from_status(" M", "crates/koba/src/output.rs"),
            ],
        );
        let output = render(&report);

        assert!(output.contains("Commit groups"));
        assert!(output.contains("working tree appears to contain multiple commit concepts"));
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
            let path = std::env::temp_dir().join(format!("koba-changes-test-{id}"));
            fs::create_dir(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }

        fn git_init(&self) {
            let output = Command::new("git")
                .arg("init")
                .current_dir(&self.path)
                .output()
                .expect("failed to run git init");
            assert!(
                output.status.success(),
                "git init failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
