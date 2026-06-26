use std::{
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    analysis::{self, WorkingTreeAnalysis},
    git, git_status,
    output::{self, Status},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrOptions {
    pub dry_run: bool,
    pub apply: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrDraft {
    pub title: String,
    pub body: String,
    pub source_notes: Vec<String>,
    pub recommended_commands: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrApplyPlan {
    pub path: PathBuf,
    pub contents: String,
    pub exists: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrOutcome {
    Preview { draft: PrDraft, plan: PrApplyPlan },
    Applied { draft: PrDraft, plan: PrApplyPlan },
}

pub fn run(cwd: PathBuf, options: PrOptions) -> Result<(), String> {
    match execute(&cwd, options) {
        Ok(outcome) => {
            print!("{}", render_outcome(&outcome));
            Ok(())
        }
        Err(error) => {
            println!("Koba PR draft");
            println!();
            println!("{}", output::line(Status::Error, &error));
            Err(error)
        }
    }
}

pub fn execute(cwd: &Path, options: PrOptions) -> Result<PrOutcome, String> {
    if options.dry_run && options.apply {
        return Err("choose either --dry-run or --apply, not both".to_owned());
    }

    let git_info = git::inspect(cwd);
    if !git_info.inside_repo {
        return Err("not inside a Git repository".to_owned());
    }

    let analysis = analysis::analyze(cwd, git_status::status_entries(cwd)?);
    let (base_branch, commits) = git::commits_since_base(cwd)
        .map(|(base, commits)| (Some(base), commits))
        .unwrap_or_else(|| (None, Vec::new()));
    let template = fs::read_to_string(cwd.join(".github").join("pull_request_template.md")).ok();
    let draft = build_draft(
        git_info.branch.as_deref(),
        base_branch.as_deref(),
        &analysis,
        &commits,
        template.as_deref(),
    );
    let plan = PrApplyPlan {
        path: cwd.join(".koba").join("pr-body.md"),
        exists: cwd.join(".koba").join("pr-body.md").exists(),
        contents: draft.body.clone(),
    };

    if !options.apply {
        return Ok(PrOutcome::Preview { draft, plan });
    }

    if !plan.exists {
        if let Some(parent) = plan.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }
        fs::write(&plan.path, &plan.contents)
            .map_err(|error| format!("failed to write {}: {error}", plan.path.display()))?;
    }

    Ok(PrOutcome::Applied { draft, plan })
}

pub fn build_draft(
    branch: Option<&str>,
    base_branch: Option<&str>,
    analysis: &WorkingTreeAnalysis,
    commits: &[String],
    template: Option<&str>,
) -> PrDraft {
    let title = analysis
        .primary_plan
        .as_ref()
        .map(|plan| plan.message.clone())
        .or_else(|| commits.first().cloned())
        .unwrap_or_else(|| "chore: prepare pull request".to_owned());
    let source_notes = source_notes(branch, base_branch, analysis, commits);
    let sections = template
        .map(template_sections)
        .filter(|sections| !sections.is_empty())
        .unwrap_or_else(default_sections);
    let body = render_body(&sections, analysis, commits);

    PrDraft {
        title,
        body,
        source_notes,
        recommended_commands: vec![
            "Review the title and body before opening a PR.".to_owned(),
            "Use your preferred Git host or CLI when ready.".to_owned(),
        ],
    }
}

fn source_notes(
    branch: Option<&str>,
    base_branch: Option<&str>,
    analysis: &WorkingTreeAnalysis,
    commits: &[String],
) -> Vec<String> {
    let mut notes = Vec::new();

    if let Some(branch) = branch.filter(|branch| !branch.is_empty()) {
        notes.push(format!("Current branch: {branch}"));
    }

    match base_branch {
        Some(base) => notes.push(format!("Compared commits against {base}")),
        None => notes.push(
            "Base branch detection was uncertain; using working tree and recent local context."
                .to_owned(),
        ),
    }

    if analysis.is_clean {
        notes.push("No uncommitted changes detected.".to_owned());
    } else {
        notes.push(format!(
            "{} changed file(s) detected.",
            analysis.files.len()
        ));
    }

    if analysis.commit_plans.len() > 1 {
        notes.push(format!(
            "{} commit group(s) detected; consider splitting before opening a PR.",
            analysis.commit_plans.len()
        ));
    }

    if !commits.is_empty() {
        notes.push(format!("{} branch commit(s) detected.", commits.len()));
    }

    notes
}

fn render_body(sections: &[String], analysis: &WorkingTreeAnalysis, commits: &[String]) -> String {
    let mut body = String::new();
    let has_changes = sections
        .iter()
        .any(|section| normalize_section(section) == "changes");

    for section in sections {
        writeln!(body, "## {section}").unwrap();
        writeln!(body).unwrap();
        write_section_content(&mut body, section, analysis, commits);
        writeln!(body).unwrap();
    }

    if !has_changes && (!analysis.files.is_empty() || !commits.is_empty()) {
        writeln!(body, "## Changes").unwrap();
        writeln!(body).unwrap();
        write_section_content(&mut body, "Changes", analysis, commits);
        writeln!(body).unwrap();
    }

    body
}

fn write_section_content(
    body: &mut String,
    section: &str,
    analysis: &WorkingTreeAnalysis,
    commits: &[String],
) {
    match normalize_section(section).as_str() {
        "summary" => {
            writeln!(body, "Describe what changed and why.").unwrap();
        }
        "changes" => {
            if commits.is_empty() && analysis.files.is_empty() {
                writeln!(body, "- No local changes detected yet.").unwrap();
                return;
            }
            for commit in commits {
                writeln!(body, "- {commit}").unwrap();
            }
            if !analysis.commit_plans.is_empty() {
                for plan in &analysis.commit_plans {
                    writeln!(body, "- {}", plan.message).unwrap();
                    for file in &plan.files {
                        writeln!(body, "  - {file}").unwrap();
                    }
                }
            } else {
                for file in &analysis.files {
                    writeln!(body, "- {} {}", file.short_status(), file.path).unwrap();
                }
            }
        }
        "checks run" => {
            writeln!(body, "- [ ] Tests").unwrap();
            writeln!(body, "- [ ] Formatting/linting").unwrap();
        }
        "risk / rollback" | "risk / rollback plan" => {
            writeln!(
                body,
                "Note risks, migration concerns, and how to roll back."
            )
            .unwrap();
        }
        "screenshots or demo" => {
            writeln!(
                body,
                "Add screenshots, recordings, or demo notes if behavior or UI changed."
            )
            .unwrap();
        }
        "notes for reviewer" => {
            writeln!(body, "Call out review focus areas or follow-up questions.").unwrap();
        }
        _ => {
            writeln!(body, "Add details for this section.").unwrap();
        }
    }
}

fn default_sections() -> Vec<String> {
    [
        "Summary",
        "Changes",
        "Checks run",
        "Risk / rollback",
        "Notes for reviewer",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn template_sections(template: &str) -> Vec<String> {
    template
        .lines()
        .filter_map(|line| line.strip_prefix("## "))
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

fn normalize_section(section: &str) -> String {
    section.trim().to_ascii_lowercase()
}

fn render_outcome(outcome: &PrOutcome) -> String {
    let mut output = String::new();
    let (draft, plan, applied) = match outcome {
        PrOutcome::Preview { draft, plan } => (draft, plan, false),
        PrOutcome::Applied { draft, plan } => (draft, plan, true),
    };

    writeln!(output, "Koba PR draft").unwrap();
    writeln!(output).unwrap();

    writeln!(output, "Git context").unwrap();
    for note in &draft.source_notes {
        let status = if note.contains("uncertain") {
            Status::Warn
        } else {
            Status::Ok
        };
        writeln!(output, "{}", output::line(status, note)).unwrap();
    }
    writeln!(output).unwrap();

    writeln!(output, "Suggested title").unwrap();
    writeln!(output, "  {}", draft.title).unwrap();
    writeln!(output).unwrap();

    output::content_block(&mut output, "Body preview", &draft.body);
    writeln!(output).unwrap();

    writeln!(output, "Apply target").unwrap();
    if plan.exists {
        writeln!(
            output,
            "{}",
            output::line(
                Status::Refuse,
                format!("{} already exists", plan.path.display())
            )
        )
        .unwrap();
        writeln!(
            output,
            "{}",
            output::next_step("Existing files are never overwritten")
        )
        .unwrap();
    } else if applied {
        writeln!(
            output,
            "{}",
            output::line(Status::Write, plan.path.display().to_string())
        )
        .unwrap();
    } else {
        writeln!(
            output,
            "{}",
            output::line(Status::Plan, plan.path.display().to_string())
        )
        .unwrap();
    }
    writeln!(output).unwrap();
    writeln!(output, "Recommended next steps").unwrap();
    for command in &draft.recommended_commands {
        writeln!(output, "{}", output::next_step(command)).unwrap();
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::Path as StdPath,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn analysis_for(files: &[(&str, &str)]) -> WorkingTreeAnalysis {
        analysis::analyze(
            StdPath::new("."),
            files
                .iter()
                .map(|(status, path)| analysis::WorkingTreeFile::from_status(status, *path))
                .collect(),
        )
    }

    #[test]
    fn builds_title_and_body_from_changed_files() {
        let draft = build_draft(
            Some("feature/pr"),
            None,
            &analysis_for(&[("A ", "crates/koba/src/pr.rs")]),
            &[],
            None,
        );

        assert_eq!(draft.title, "feat(pr): update PR draft helper");
        assert!(draft.body.contains("## Summary"));
        assert!(draft.body.contains("- feat(pr): update PR draft helper"));
        assert!(draft.body.contains("  - crates/koba/src/pr.rs"));
        assert!(draft
            .source_notes
            .iter()
            .any(|note| note.contains("Base branch detection was uncertain")));
    }

    #[test]
    fn uses_template_section_shape_when_available() {
        let draft = build_draft(
            Some("feature/pr"),
            Some("origin/main"),
            &analysis_for(&[(" M", "docs/product.md")]),
            &["docs(product): update product docs".to_owned()],
            Some("## Summary\n\n## Screenshots or demo\n\n## Notes for reviewer\n"),
        );

        assert!(draft.body.contains("## Screenshots or demo"));
        assert!(!draft.body.contains("## Risk / rollback"));
        assert!(draft.body.contains("docs(product): update product docs"));
    }

    #[test]
    fn body_lists_every_untracked_file_from_git_status() {
        let entries = git_status::parse_porcelain_z(
            b"?? crates/koba/src/git_status.rs\0?? crates/koba/src/path_classification.rs\0",
        )
        .unwrap();
        let analysis = analysis::analyze(StdPath::new("."), entries);
        let draft = build_draft(Some("analysis/refactor"), None, &analysis, &[], None);

        assert!(draft.body.contains("crates/koba/src/git_status.rs"));
        assert!(draft
            .body
            .contains("crates/koba/src/path_classification.rs"));
    }

    #[test]
    fn title_uses_shared_primary_plan() {
        let analysis = analysis_for(&[
            ("A ", "crates/koba/src/git_status.rs"),
            ("A ", "crates/koba/src/path_classification.rs"),
            (" M", "crates/koba/src/changes.rs"),
            (" M", "crates/koba/src/suggest_commit.rs"),
            (" M", "crates/koba/src/pr.rs"),
        ]);
        let draft = build_draft(Some("analysis/refactor"), None, &analysis, &[], None);

        assert_eq!(
            draft.title,
            "refactor(analysis): centralize status and path classification"
        );
    }

    #[test]
    fn preview_does_not_write_pr_body() {
        let fixture = TempTree::new();
        fixture.git_init();
        fixture.file("docs/change.md", "");

        let outcome = execute(
            fixture.path(),
            PrOptions {
                dry_run: false,
                apply: false,
            },
        )
        .unwrap();

        assert!(matches!(outcome, PrOutcome::Preview { .. }));
        assert!(!fixture.path().join(".koba/pr-body.md").exists());
    }

    #[test]
    fn apply_writes_pr_body() {
        let fixture = TempTree::new();
        fixture.git_init();
        fixture.file("docs/change.md", "");

        let outcome = execute(
            fixture.path(),
            PrOptions {
                dry_run: false,
                apply: true,
            },
        )
        .unwrap();

        assert!(matches!(outcome, PrOutcome::Applied { .. }));
        let body = fs::read_to_string(fixture.path().join(".koba/pr-body.md")).unwrap();
        assert!(body.contains("## Summary"));
        assert!(body.contains("docs/change.md"));
    }

    #[test]
    fn apply_does_not_overwrite_existing_pr_body() {
        let fixture = TempTree::new();
        fixture.git_init();
        fixture.file(".koba/pr-body.md", "existing\n");

        let outcome = execute(
            fixture.path(),
            PrOptions {
                dry_run: false,
                apply: true,
            },
        )
        .unwrap();

        assert!(matches!(outcome, PrOutcome::Applied { plan, .. } if plan.exists));
        assert_eq!(
            fs::read_to_string(fixture.path().join(".koba/pr-body.md")).unwrap(),
            "existing\n"
        );
    }

    #[test]
    fn dry_run_and_apply_are_rejected_together() {
        let fixture = TempTree::new();
        fixture.git_init();

        let error = execute(
            fixture.path(),
            PrOptions {
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
            let path = std::env::temp_dir().join(format!("koba-pr-test-{id}"));
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

        fn git_init(&self) {
            let output = std::process::Command::new("git")
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
