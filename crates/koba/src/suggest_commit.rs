use std::{collections::BTreeMap, fmt::Write, path::PathBuf};

use crate::{
    git,
    output::{self, Status},
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
            println!("{}", output::line(Status::Missing, &error));
            Err(error)
        }
    }
}

pub fn execute(cwd: PathBuf) -> Result<String, String> {
    let info = git::inspect(&cwd);
    if !info.inside_repo {
        return Err("not inside a Git repository".to_owned());
    }

    let changed_files = parse_porcelain(&git::status_porcelain(&cwd)?);
    Ok(render(&changed_files))
}

pub fn parse_porcelain(output: &str) -> Vec<ChangedFile> {
    output.lines().filter_map(parse_porcelain_line).collect()
}

pub fn suggest(files: &[ChangedFile]) -> Option<CommitSuggestion> {
    if files.is_empty() {
        return None;
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
    for file in files {
        writeln!(
            output,
            "{}",
            output::line(Status::Step, format!("{} {}", file.status, file.path))
        )
        .unwrap();
    }

    writeln!(output).unwrap();
    writeln!(output, "Suggested commit").unwrap();
    writeln!(
        output,
        "{}",
        output::line(Status::Ok, suggestion.message.as_str())
    )
    .unwrap();
    if let Some(note) = &suggestion.note {
        writeln!(output, "{}", output::line(Status::Warning, note)).unwrap();
    }

    writeln!(output).unwrap();
    writeln!(output, "Recommended commands").unwrap();
    writeln!(
        output,
        "{}",
        output::line(Status::Step, format!("git add -- {}", quote_paths(&paths)))
    )
    .unwrap();
    writeln!(
        output,
        "{}",
        output::line(
            Status::Step,
            format!("git commit -m {}", quote_shell_arg(&suggestion.message))
        )
    )
    .unwrap();

    output
}

fn parse_porcelain_line(line: &str) -> Option<ChangedFile> {
    if line.len() < 4 {
        return None;
    }

    let status = line.get(0..2)?.trim().to_owned();
    let raw_path = line.get(3..)?.trim();
    let path = raw_path
        .rsplit_once(" -> ")
        .map(|(_, new_path)| new_path)
        .unwrap_or(raw_path)
        .trim_matches('"')
        .to_owned();

    Some(ChangedFile { status, path })
}

fn suggest_type(files: &[ChangedFile]) -> &'static str {
    if files.iter().all(|file| is_docs_file(&file.path)) {
        return "docs";
    }

    if files.iter().all(|file| is_test_file(&file.path)) {
        return "test";
    }

    if files.iter().all(|file| is_chore_file(&file.path)) {
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
        if let Some(scope) = scope_for_path(&file.path) {
            *counts.entry(scope).or_default() += 1;
        }
    }

    if counts.is_empty() {
        return (None, false);
    }

    let max_count = counts.values().copied().max().unwrap_or_default();
    let scope = scope_priority()
        .into_iter()
        .find(|scope| counts.get(scope).copied() == Some(max_count))
        .or_else(|| counts.keys().next().copied())
        .expect("non-empty scope map should have a scope");

    (Some(scope.to_owned()), counts.len() > 1)
}

fn scope_priority() -> [&'static str; 9] {
    [
        "github", "hooks", "run", "init", "doctor", "scan", "repo", "config", "cli",
    ]
}

fn suggest_description(commit_type: &str, scope: Option<&str>) -> &'static str {
    match (commit_type, scope) {
        ("docs", Some("product")) => "update product documentation",
        ("docs", _) => "update documentation",
        ("test", Some("scan")) => "cover workflow file discovery",
        ("test", _) => "add coverage",
        ("feat", Some("github")) => "add PR template generation",
        ("feat", Some("hooks")) => "install native and husky hooks",
        ("feat", Some("run")) => "execute configured checks",
        ("feat", Some("init")) => "create workflow contract preview",
        ("feat", Some("doctor")) => "diagnose workflow setup",
        ("feat", Some("scan")) => "scan workflow files",
        ("feat", Some("cli")) => "update command surface",
        ("feat", _) => "update workflow tooling",
        ("chore", Some("config")) => "update configuration",
        ("chore", _) => "update project setup",
        _ => "update workflow tooling",
    }
}

fn scope_for_path(path: &str) -> Option<&'static str> {
    let normalized = path.replace('\\', "/").to_ascii_lowercase();
    let file_name = normalized.rsplit('/').next().unwrap_or(normalized.as_str());

    if normalized.contains("docs/product") {
        return Some("product");
    }

    for scope in [
        "scan", "doctor", "init", "run", "hooks", "github", "repo", "config",
    ] {
        if file_name.contains(scope) || normalized.contains(&format!("/{scope}/")) {
            return Some(scope);
        }
    }

    if file_name == "cli.rs" || normalized.contains("/src/cli.") {
        return Some("cli");
    }

    None
}

fn is_docs_file(path: &str) -> bool {
    let path = path.replace('\\', "/").to_ascii_lowercase();
    path.starts_with("docs/")
        || path.ends_with(".md")
        || path.ends_with(".mdx")
        || path.ends_with(".rst")
}

fn is_test_file(path: &str) -> bool {
    let path = path.replace('\\', "/").to_ascii_lowercase();
    let file_name = path.rsplit('/').next().unwrap_or(path.as_str());
    path.contains("/tests/")
        || path.starts_with("tests/")
        || path.contains("__tests__")
        || file_name.ends_with("_test.rs")
        || file_name.ends_with("_tests.rs")
        || file_name.starts_with("test_")
        || file_name.contains(".test.")
        || file_name.contains(".spec.")
}

fn is_chore_file(path: &str) -> bool {
    let path = path.replace('\\', "/").to_ascii_lowercase();
    let file_name = path.rsplit('/').next().unwrap_or(path.as_str());
    matches!(
        file_name,
        "cargo.toml"
            | "cargo.lock"
            | "package.json"
            | "package-lock.json"
            | "pnpm-lock.yaml"
            | "yarn.lock"
            | ".gitignore"
            | ".gitattributes"
            | "rust-toolchain.toml"
            | "rustfmt.toml"
    ) || path.starts_with(".github/")
        || path.ends_with(".yml")
        || path.ends_with(".yaml")
        || path.ends_with(".toml")
        || path.ends_with(".json")
}

fn is_feature_signal(file: &ChangedFile) -> bool {
    let path = file.path.replace('\\', "/").to_ascii_lowercase();
    let file_name = path.rsplit('/').next().unwrap_or(path.as_str());
    let added = file.status.contains('A') || file.status == "??";

    file_name == "cli.rs"
        || path.contains("/src/")
        || (added && (path.ends_with(".rs") || path.ends_with(".ts") || path.ends_with(".tsx")))
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
        let files = parse_porcelain(" M docs/product.md\n?? crates/koba/src/github.rs\nR  old.rs -> crates/koba/src/run_checks.rs\n");

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
}
