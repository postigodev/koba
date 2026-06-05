use std::{
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    git,
    output::{self, Status},
    suggest_commit::{self, ChangedFile},
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
            println!("Koba pr");
            println!();
            println!("{}", output::line(Status::Missing, &error));
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

    let changed_files = suggest_commit::parse_porcelain(&git::status_porcelain(cwd)?);
    let (base_branch, commits) = git::commits_since_base(cwd)
        .map(|(base, commits)| (Some(base), commits))
        .unwrap_or_else(|| (None, Vec::new()));
    let template = fs::read_to_string(cwd.join(".github").join("pull_request_template.md")).ok();
    let draft = build_draft(
        git_info.branch.as_deref(),
        base_branch.as_deref(),
        &changed_files,
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
    changed_files: &[ChangedFile],
    commits: &[String],
    template: Option<&str>,
) -> PrDraft {
    let title = suggest_commit::suggest(changed_files)
        .map(|suggestion| suggestion.message)
        .or_else(|| commits.first().cloned())
        .unwrap_or_else(|| "chore: prepare pull request".to_owned());
    let source_notes = source_notes(branch, base_branch, changed_files, commits);
    let sections = template
        .map(template_sections)
        .filter(|sections| !sections.is_empty())
        .unwrap_or_else(default_sections);
    let body = render_body(&sections, changed_files, commits);

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
    changed_files: &[ChangedFile],
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

    if changed_files.is_empty() {
        notes.push("No uncommitted changes detected.".to_owned());
    } else {
        notes.push(format!("{} changed file(s) detected.", changed_files.len()));
    }

    if !commits.is_empty() {
        notes.push(format!("{} branch commit(s) detected.", commits.len()));
    }

    notes
}

fn render_body(sections: &[String], changed_files: &[ChangedFile], commits: &[String]) -> String {
    let mut body = String::new();
    let has_changes = sections
        .iter()
        .any(|section| normalize_section(section) == "changes");

    for section in sections {
        writeln!(body, "## {section}").unwrap();
        writeln!(body).unwrap();
        write_section_content(&mut body, section, changed_files, commits);
        writeln!(body).unwrap();
    }

    if !has_changes && (!changed_files.is_empty() || !commits.is_empty()) {
        writeln!(body, "## Changes").unwrap();
        writeln!(body).unwrap();
        write_section_content(&mut body, "Changes", changed_files, commits);
        writeln!(body).unwrap();
    }

    body
}

fn write_section_content(
    body: &mut String,
    section: &str,
    changed_files: &[ChangedFile],
    commits: &[String],
) {
    match normalize_section(section).as_str() {
        "summary" => {
            writeln!(body, "Describe what changed and why.").unwrap();
        }
        "changes" => {
            if commits.is_empty() && changed_files.is_empty() {
                writeln!(body, "- No local changes detected yet.").unwrap();
                return;
            }
            for commit in commits {
                writeln!(body, "- {commit}").unwrap();
            }
            for file in changed_files {
                writeln!(body, "- {} {}", file.status, file.path).unwrap();
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

    writeln!(output, "Koba pr").unwrap();
    writeln!(output).unwrap();
    writeln!(output, "Title").unwrap();
    writeln!(output, "{}", output::line(Status::Ok, &draft.title)).unwrap();
    writeln!(output).unwrap();
    writeln!(output, "Source notes").unwrap();
    for note in &draft.source_notes {
        writeln!(output, "{}", output::line(Status::Step, note)).unwrap();
    }
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
    } else if applied {
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
    writeln!(output, "Body").unwrap();
    writeln!(output, "{}", indent_contents(&draft.body)).unwrap();
    writeln!(output).unwrap();
    writeln!(output, "Recommended next steps").unwrap();
    for command in &draft.recommended_commands {
        writeln!(output, "{}", output::line(Status::Step, command)).unwrap();
    }

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
        time::{SystemTime, UNIX_EPOCH},
    };

    fn file(status: &str, path: &str) -> ChangedFile {
        ChangedFile {
            status: status.to_owned(),
            path: path.to_owned(),
        }
    }

    #[test]
    fn builds_title_and_body_from_changed_files() {
        let draft = build_draft(
            Some("feature/pr"),
            None,
            &[file("A", "crates/koba/src/pr.rs")],
            &[],
            None,
        );

        assert_eq!(draft.title, "feat: update workflow tooling");
        assert!(draft.body.contains("## Summary"));
        assert!(draft.body.contains("- A crates/koba/src/pr.rs"));
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
            &[file("M", "docs/product.md")],
            &["docs(product): update product docs".to_owned()],
            Some("## Summary\n\n## Screenshots or demo\n\n## Notes for reviewer\n"),
        );

        assert!(draft.body.contains("## Screenshots or demo"));
        assert!(!draft.body.contains("## Risk / rollback"));
        assert!(draft.body.contains("docs(product): update product docs"));
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
