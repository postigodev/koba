use std::{collections::BTreeMap, fmt::Write, path::PathBuf};

use crate::{
    git, git_status,
    output::{self, Status},
    path_classification,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangedFile {
    pub status: String,
    pub path: String,
}

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

    let changed_files = changed_files_from_status(&git_status::status_entries(&cwd)?);
    Ok(render(&changed_files))
}

pub fn changed_files_from_status(entries: &[git_status::GitStatusEntry]) -> Vec<ChangedFile> {
    entries
        .iter()
        .map(|entry| ChangedFile {
            status: entry.short_status(),
            path: entry.path.clone(),
        })
        .collect()
}

pub fn suggest(files: &[ChangedFile]) -> Option<CommitSuggestion> {
    if files.is_empty() {
        return None;
    }

    if path_classification::is_analysis_refactor_path_set(
        files.iter().map(|file| file.path.as_str()),
    ) {
        return Some(CommitSuggestion {
            message: "refactor(analysis): centralize status and path classification".to_owned(),
            note: None,
        });
    }

    let commit_type = suggest_type(files);
    let (scope, multiple_scopes) = suggest_scope(files);
    let description = suggest_description(commit_type, scope.as_deref());
    let message = match scope {
        Some(scope) => format!("{commit_type}({scope}): {description}"),
        None => format!("{commit_type}: {description}"),
    };

    Some(CommitSuggestion {
        message,
        note: multiple_scopes.then(|| {
            "Multiple areas changed; consider splitting this into more than one commit.".to_owned()
        }),
    })
}

fn render(files: &[ChangedFile]) -> String {
    let mut output = String::new();

    writeln!(output, "Koba suggest-commit").unwrap();
    writeln!(output).unwrap();

    if files.is_empty() {
        writeln!(
            output,
            "{}",
            output::line(Status::Ok, "Working tree is clean")
        )
        .unwrap();
        return output;
    }

    let suggestion = suggest(files).expect("non-empty files should produce a suggestion");
    let paths: Vec<_> = files.iter().map(|file| file.path.as_str()).collect();

    writeln!(output, "Changed files").unwrap();
    writeln!(
        output,
        "{}",
        output::line(Status::Ok, format!("{} files", files.len()))
    )
    .unwrap();
    for file in files {
        writeln!(output, "          {} {}", file.status, file.path).unwrap();
    }

    writeln!(output).unwrap();
    writeln!(output, "Suggested commit").unwrap();
    writeln!(output, "  {}", suggestion.message).unwrap();

    writeln!(output).unwrap();
    writeln!(output, "Recommended commands").unwrap();
    writeln!(output, "  git add -- {}", quote_paths(&paths)).unwrap();
    writeln!(
        output,
        "  git commit -m {}",
        quote_shell_arg(&suggestion.message)
    )
    .unwrap();

    if let Some(note) = &suggestion.note {
        writeln!(output).unwrap();
        writeln!(output, "Notes").unwrap();
        writeln!(output, "{}", output::line(Status::Warn, note)).unwrap();
    }

    output
}

fn suggest_type(files: &[ChangedFile]) -> &'static str {
    if files
        .iter()
        .all(|file| path_classification::is_skill_repo_file(&file.path))
    {
        if files
            .iter()
            .all(|file| path_classification::is_docs_file(&file.path))
        {
            return "docs";
        }

        return "feat";
    }

    if files
        .iter()
        .all(|file| path_classification::is_github_workflow(&file.path))
    {
        return "ci";
    }

    if files
        .iter()
        .all(|file| path_classification::is_docs_file(&file.path))
    {
        return "docs";
    }

    if files
        .iter()
        .all(|file| path_classification::is_test_file(&file.path))
    {
        return "test";
    }

    if files
        .iter()
        .all(|file| path_classification::is_chore_file(&file.path))
    {
        return "chore";
    }

    if files.iter().any(is_feature_signal) {
        return "feat";
    }

    "chore"
}

fn suggest_scope(files: &[ChangedFile]) -> (Option<String>, bool) {
    let mut counts = BTreeMap::<&'static str, usize>::new();
    for file in files {
        if let Some(scope) = path_classification::commit_scope_for_path(&file.path) {
            *counts.entry(scope).or_default() += 1;
        }
    }

    if counts.is_empty() {
        return (None, false);
    }

    let max_count = counts.values().copied().max().unwrap_or_default();
    let scope = path_classification::scope_priority()
        .into_iter()
        .find(|scope| counts.get(scope).copied() == Some(max_count))
        .or_else(|| counts.keys().next().copied())
        .expect("non-empty scope map should have a scope");

    (Some(scope.to_owned()), counts.len() > 1)
}

fn suggest_description(commit_type: &str, scope: Option<&str>) -> &'static str {
    match (commit_type, scope) {
        ("ci", Some("github")) => "update GitHub Actions workflow",
        ("ci", _) => "update CI workflow",
        ("docs", Some("agents")) => "update agent documentation",
        ("docs", Some("skill")) => "update skill documentation",
        ("docs", Some("github")) => "update GitHub documentation",
        ("docs", Some("product")) => "update product documentation",
        ("docs", _) => "update documentation",
        ("test", Some("scan")) => "cover workflow file discovery",
        ("test", Some("skill")) => "validate skill behavior",
        ("test", _) => "add coverage",
        ("feat", Some("changes")) => "review working tree changes",
        ("feat", Some("commit")) => "sharpen path-based scope inference",
        ("feat", Some("output")) => "improve terminal rendering",
        ("feat", Some("pr")) => "update PR draft helper",
        ("feat", Some("skill")) => "expand skill examples and evals",
        ("feat", Some("github")) => "add PR template generation",
        ("feat", Some("hooks")) => "install native and husky hooks",
        ("feat", Some("run")) => "execute configured checks",
        ("feat", Some("init")) => "create workflow contract preview",
        ("feat", Some("doctor")) => "diagnose workflow setup",
        ("feat", Some("scan")) => "scan workflow files",
        ("feat", Some("cli")) => "update command surface",
        ("feat", _) => "update workflow tooling",
        ("chore", Some("scoop")) => "update Scoop packaging",
        ("chore", Some("config")) => "update configuration",
        ("chore", _) => "update project setup",
        ("refactor", Some("analysis")) => "centralize status and path classification",
        _ => "update workflow tooling",
    }
}

fn is_feature_signal(file: &ChangedFile) -> bool {
    path_classification::is_feature_signal(&file.status, &file.path)
}

fn quote_paths(paths: &[&str]) -> String {
    paths
        .iter()
        .map(|path| quote_shell_arg(path))
        .collect::<Vec<_>>()
        .join(" ")
}

fn quote_shell_arg(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(status: &str, path: &str) -> ChangedFile {
        ChangedFile {
            status: status.to_owned(),
            path: path.to_owned(),
        }
    }

    #[test]
    fn parses_porcelain_changed_files() {
        let entries = git_status::parse_porcelain_z(
            b" M docs/product.md\0?? crates/koba/src/github.rs\0R  crates/koba/src/run_checks.rs\0old.rs\0",
        )
        .unwrap();
        let files = changed_files_from_status(&entries);

        assert_eq!(files[0], file("M", "docs/product.md"));
        assert_eq!(files[1], file("??", "crates/koba/src/github.rs"));
        assert_eq!(files[2], file("R", "crates/koba/src/run_checks.rs"));
    }

    #[test]
    fn suggests_docs_commit_for_docs_only() {
        let suggestion = suggest(&[file("M", "docs/product.md")]).unwrap();

        assert_eq!(
            suggestion.message,
            "docs(product): update product documentation"
        );
    }

    #[test]
    fn suggests_test_commit_for_tests_only() {
        let suggestion = suggest(&[file("M", "crates/koba/tests/cli.rs")]).unwrap();

        assert_eq!(suggestion.message, "test(cli): add coverage");
    }

    #[test]
    fn suggests_chore_for_cargo_and_config_only() {
        let suggestion = suggest(&[
            file("M", "Cargo.toml"),
            file("M", "Cargo.lock"),
            file("M", ".gitignore"),
        ])
        .unwrap();

        assert_eq!(suggestion.message, "chore: update project setup");
    }

    #[test]
    fn suggests_feature_scope_for_new_github_module() {
        let suggestion = suggest(&[
            file("A", "crates/koba/src/github.rs"),
            file("M", "crates/koba/src/cli.rs"),
        ])
        .unwrap();

        assert_eq!(
            suggestion.message,
            "feat(github): add PR template generation"
        );
        assert!(suggestion.note.is_some());
    }

    #[test]
    fn clean_file_list_has_no_suggestion() {
        assert!(suggest(&[]).is_none());
    }

    #[test]
    fn suggests_agent_docs_scope_for_agent_documentation() {
        let suggestion = suggest(&[file("M", "README.md"), file("M", "docs/agents.md")]).unwrap();

        assert_eq!(
            suggestion.message,
            "docs(agents): update agent documentation"
        );
    }

    #[test]
    fn suggests_skill_docs_scope_for_skill_markdown() {
        let suggestion = suggest(&[
            file("M", "skills/koba/SKILL.md"),
            file("M", "skills/koba/references/workflows.md"),
        ])
        .unwrap();

        assert_eq!(
            suggestion.message,
            "docs(skill): update skill documentation"
        );
    }

    #[test]
    fn suggests_skill_feature_scope_for_non_docs_skill_files() {
        let suggestion = suggest(&[file("A", "skills/koba/scripts/check.ps1")]).unwrap();

        assert_eq!(
            suggestion.message,
            "feat(skill): expand skill examples and evals"
        );
    }

    #[test]
    fn suggests_skill_feature_for_agent_skill_repo_changes() {
        let suggestion = suggest(&[
            file("M", "README.md"),
            file("M", "skills/hoi4-modding/SKILL.md"),
            file("A", "skills/hoi4-modding/examples/minimal-event.txt"),
            file("A", "evals/expected-behavior.md"),
            file("A", "evals/trigger-evals.json"),
            file("M", "tests/smoke-prompts.md"),
        ])
        .unwrap();

        assert_eq!(
            suggestion.message,
            "feat(skill): expand skill examples and evals"
        );
    }

    #[test]
    fn suggests_scoop_scope_for_scoop_packaging() {
        let suggestion = suggest(&[file("M", "packaging/scoop/bucket/koba.json")]).unwrap();

        assert_eq!(suggestion.message, "chore(scoop): update Scoop packaging");
    }

    #[test]
    fn suggests_github_ci_scope_for_workflows() {
        let suggestion = suggest(&[file("M", ".github/workflows/ci.yml")]).unwrap();

        assert_eq!(
            suggestion.message,
            "ci(github): update GitHub Actions workflow"
        );
    }

    #[test]
    fn suggests_github_docs_scope_for_pr_template() {
        let suggestion = suggest(&[file("M", ".github/pull_request_template.md")]).unwrap();

        assert_eq!(
            suggestion.message,
            "docs(github): update GitHub documentation"
        );
    }

    #[test]
    fn suggests_koba_source_module_scopes() {
        assert_eq!(
            suggest(&[file("M", "crates/koba/src/changes.rs")])
                .unwrap()
                .message,
            "feat(changes): review working tree changes"
        );
        assert_eq!(
            suggest(&[file("M", "crates/koba/src/suggest_commit.rs")])
                .unwrap()
                .message,
            "feat(commit): sharpen path-based scope inference"
        );
        assert_eq!(
            suggest(&[file("M", "crates/koba/src/output.rs")])
                .unwrap()
                .message,
            "feat(output): improve terminal rendering"
        );
        assert_eq!(
            suggest(&[file("M", "crates/koba/src/pr.rs")])
                .unwrap()
                .message,
            "feat(pr): update PR draft helper"
        );
    }

    #[test]
    fn suggests_analysis_refactor_for_shared_modules_and_consumers() {
        let suggestion = suggest(&[
            file("A", "crates/koba/src/git_status.rs"),
            file("A", "crates/koba/src/path_classification.rs"),
            file("M", "crates/koba/src/changes.rs"),
            file("M", "crates/koba/src/suggest_commit.rs"),
            file("M", "crates/koba/src/pr.rs"),
        ])
        .unwrap();

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
        let files = changed_files_from_status(&entries);
        let output = render(&files);

        assert!(output.contains("[ok]     2 files"));
        assert!(output.contains("?? crates/koba/src/git_status.rs"));
        assert!(output.contains("?? crates/koba/src/path_classification.rs"));
        assert!(output.contains("\"crates/koba/src/git_status.rs\""));
        assert!(output.contains("\"crates/koba/src/path_classification.rs\""));
    }

    #[test]
    fn isolated_changes_module_still_suggests_changes_scope() {
        let suggestion = suggest(&[file("M", "crates/koba/src/changes.rs")]).unwrap();

        assert_eq!(
            suggestion.message,
            "feat(changes): review working tree changes"
        );
    }
}
