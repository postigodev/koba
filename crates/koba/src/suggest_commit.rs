use std::{fmt::Write, path::PathBuf};

use crate::{
    analysis::{self, WorkingTreeAnalysis},
    git,
    output::{self, Status},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitSuggestion {
    pub message: String,
    pub note: Option<String>,
}

pub fn run(cwd: PathBuf) -> Result<(), String> {
    match execute(cwd) {
        Ok(output) => {
            print!("{output}");
            Ok(())
        }
        Err(error) => {
            println!("Koba suggest-commit");
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

    let analysis = analysis::analyze_cwd(&cwd)?;
    Ok(render(&analysis))
}

pub fn suggest(analysis: &WorkingTreeAnalysis) -> Option<CommitSuggestion> {
    analysis.primary_plan.as_ref().map(|plan| CommitSuggestion {
        message: plan.message.clone(),
        note: (analysis.commit_plans.len() > 1).then(|| {
            "Multiple areas changed; consider splitting this into more than one commit.".to_owned()
        }),
    })
}

fn render(analysis: &WorkingTreeAnalysis) -> String {
    let mut output = String::new();

    writeln!(output, "Koba suggest-commit").unwrap();
    writeln!(output).unwrap();

    if analysis.is_clean {
        writeln!(
            output,
            "{}",
            output::line(Status::Ok, "Working tree is clean")
        )
        .unwrap();
        return output;
    }

    let suggestion = suggest(analysis).expect("non-empty analysis should produce a suggestion");
    let plan = analysis
        .primary_plan
        .as_ref()
        .expect("suggestion should have a primary plan");

    writeln!(output, "Changed files").unwrap();
    writeln!(
        output,
        "{}",
        output::line(Status::Ok, format!("{} files", analysis.files.len()))
    )
    .unwrap();
    for file in &analysis.files {
        writeln!(output, "          {} {}", file.short_status(), file.path).unwrap();
    }

    writeln!(output).unwrap();
    writeln!(output, "Suggested commit").unwrap();
    writeln!(output, "  {}", suggestion.message).unwrap();

    writeln!(output).unwrap();
    writeln!(output, "Recommended commands").unwrap();
    writeln!(output, "  {}", plan.git_add_command()).unwrap();
    writeln!(output, "  {}", plan.git_commit_command()).unwrap();

    if let Some(note) = &suggestion.note {
        writeln!(output).unwrap();
        writeln!(output, "Notes").unwrap();
        writeln!(output, "{}", output::line(Status::Warn, note)).unwrap();
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git_status;
    use std::path::Path;

    fn analysis_for(paths: &[&str]) -> WorkingTreeAnalysis {
        analysis::analyze(
            Path::new("."),
            paths
                .iter()
                .map(|path| analysis::WorkingTreeFile::from_status(" M", *path))
                .collect(),
        )
    }

    #[test]
    fn suggests_docs_commit_for_docs_only() {
        let analysis = analysis_for(&["docs/product.md"]);
        let suggestion = suggest(&analysis).unwrap();

        assert_eq!(
            suggestion.message,
            "docs(product): update product documentation"
        );
    }

    #[test]
    fn suggests_test_commit_for_tests_only() {
        let analysis = analysis_for(&["crates/koba/tests/cli.rs"]);
        let suggestion = suggest(&analysis).unwrap();

        assert_eq!(suggestion.message, "test(cli): add coverage");
    }

    #[test]
    fn clean_analysis_has_no_suggestion() {
        let analysis = analysis::analyze(Path::new("."), Vec::new());

        assert!(suggest(&analysis).is_none());
    }

    #[test]
    fn suggests_agent_docs_scope_for_agent_documentation() {
        let analysis = analysis_for(&["README.md", "docs/agents.md"]);
        let suggestion = suggest(&analysis).unwrap();

        assert_eq!(
            suggestion.message,
            "docs(agents): update agent documentation"
        );
    }

    #[test]
    fn suggests_skill_docs_scope_for_skill_markdown() {
        let analysis = analysis_for(&[
            "skills/koba/SKILL.md",
            "skills/koba/references/workflows.md",
        ]);
        let suggestion = suggest(&analysis).unwrap();

        assert_eq!(
            suggestion.message,
            "docs(skill): update skill documentation"
        );
    }

    #[test]
    fn suggests_skill_feature_for_agent_skill_repo_changes() {
        let analysis = analysis_for(&[
            "README.md",
            "skills/hoi4-modding/SKILL.md",
            "skills/hoi4-modding/examples/minimal-event.txt",
            "evals/expected-behavior.md",
            "evals/trigger-evals.json",
            "tests/smoke-prompts.md",
        ]);
        let suggestion = suggest(&analysis).unwrap();

        assert_eq!(
            suggestion.message,
            "feat(skill): expand skill examples and evals"
        );
    }

    #[test]
    fn suggests_scoop_scope_for_scoop_packaging() {
        let analysis = analysis_for(&["packaging/scoop/bucket/koba.json"]);
        let suggestion = suggest(&analysis).unwrap();

        assert_eq!(suggestion.message, "chore(scoop): update Scoop packaging");
    }

    #[test]
    fn suggests_github_ci_scope_for_workflows() {
        let analysis = analysis_for(&[".github/workflows/ci.yml"]);
        let suggestion = suggest(&analysis).unwrap();

        assert_eq!(
            suggestion.message,
            "ci(github): update GitHub Actions workflow"
        );
    }

    #[test]
    fn suggests_koba_source_module_scopes() {
        assert_eq!(
            suggest(&analysis_for(&["crates/koba/src/changes.rs"]))
                .unwrap()
                .message,
            "feat(changes): review working tree changes"
        );
        assert_eq!(
            suggest(&analysis_for(&["crates/koba/src/suggest_commit.rs"]))
                .unwrap()
                .message,
            "feat(commit): sharpen path-based scope inference"
        );
        assert_eq!(
            suggest(&analysis_for(&["crates/koba/src/output.rs"]))
                .unwrap()
                .message,
            "feat(output): improve terminal rendering"
        );
        assert_eq!(
            suggest(&analysis_for(&["crates/koba/src/pr.rs"]))
                .unwrap()
                .message,
            "feat(pr): update PR draft helper"
        );
    }

    #[test]
    fn suggests_analysis_refactor_for_shared_modules_and_consumers() {
        let analysis = analysis_for(&[
            "crates/koba/src/git_status.rs",
            "crates/koba/src/path_classification.rs",
            "crates/koba/src/changes.rs",
            "crates/koba/src/suggest_commit.rs",
            "crates/koba/src/pr.rs",
        ]);
        let suggestion = suggest(&analysis).unwrap();

        assert_eq!(
            suggestion.message,
            "refactor(analysis): centralize status and path classification"
        );
        assert!(suggestion.note.is_none());
    }

    #[test]
    fn render_lists_every_untracked_file_used_in_recommended_add_command() {
        let entries = git_status::parse_porcelain_z(
            b"?? crates/koba/src/git_status.rs\0?? crates/koba/src/path_classification.rs\0",
        )
        .unwrap();
        let analysis = analysis::analyze(Path::new("."), entries);
        let output = render(&analysis);

        assert!(output.contains("[ok]     2 files"));
        assert!(output.contains("?? crates/koba/src/git_status.rs"));
        assert!(output.contains("?? crates/koba/src/path_classification.rs"));
        assert!(output.contains("\"crates/koba/src/git_status.rs\""));
        assert!(output.contains("\"crates/koba/src/path_classification.rs\""));
    }

    #[test]
    fn multiple_plans_warn_but_choose_primary_plan() {
        let analysis = analysis_for(&["skills/koba/SKILL.md", "crates/koba/src/output.rs"]);
        let suggestion = suggest(&analysis).unwrap();

        assert_eq!(
            suggestion.message,
            "feat(output): improve terminal rendering"
        );
        assert!(suggestion.note.is_some());
    }
}
