use std::{collections::BTreeMap, fmt::Write, fs, path::PathBuf};

use crate::{
    git,
    output::{self, Status, StatusRow},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkingTreeFile {
    pub path: String,
    pub status: String,
    pub staged: bool,
    pub unstaged: bool,
    pub untracked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitPlan {
    pub message: String,
    pub files: Vec<String>,
    pub confidence: Confidence,
    pub reasons: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckRecommendation {
    pub command: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangesReport {
    pub changed_count: usize,
    pub staged_count: usize,
    pub unstaged_count: usize,
    pub untracked_count: usize,
    pub plans: Vec<CommitPlan>,
    pub checks: Vec<CheckRecommendation>,
    pub risks: Vec<Risk>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Risk {
    pub status: Status,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ChangeConcept {
    AgentsDocs,
    Skill,
    CommitEngine,
    Output,
    Pr,
    Hooks,
    Github,
    RunChecks,
    Init,
    Changes,
    Scoop,
    GithubCi,
    GithubTemplate,
    Rust,
    Docs,
    Config,
    Other,
}

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

    let files = parse_status_porcelain(&git::status_porcelain(&cwd)?);
    let report = analyze(&cwd, &files);
    Ok(render(&report))
}

pub fn parse_status_porcelain(output: &str) -> Vec<WorkingTreeFile> {
    output.lines().filter_map(parse_status_line).collect()
}

pub fn analyze(cwd: &PathBuf, files: &[WorkingTreeFile]) -> ChangesReport {
    let staged_count = files.iter().filter(|file| file.staged).count();
    let unstaged_count = files.iter().filter(|file| file.unstaged).count();
    let untracked_count = files.iter().filter(|file| file.untracked).count();
    let plans = plan_commits(files);
    let checks = recommend_checks(cwd, files);
    let risks = assess_risks(files, &plans);

    ChangesReport {
        changed_count: files.len(),
        staged_count,
        unstaged_count,
        untracked_count,
        plans,
        checks,
        risks,
    }
}

fn parse_status_line(line: &str) -> Option<WorkingTreeFile> {
    if line.len() < 4 {
        return None;
    }

    let mut chars = line.chars();
    let x = chars.next()?;
    let y = chars.next()?;
    let raw_path = line.get(3..)?.trim();
    let path = raw_path
        .rsplit_once(" -> ")
        .map(|(_, new_path)| new_path)
        .unwrap_or(raw_path)
        .trim_matches('"')
        .to_owned();
    let untracked = x == '?' && y == '?';
    let staged = !untracked && x != ' ';
    let unstaged = !untracked && y != ' ';
    let status = if untracked {
        "??".to_owned()
    } else {
        format!("{x}{y}").trim().to_owned()
    };

    Some(WorkingTreeFile {
        path,
        status,
        staged,
        unstaged,
        untracked,
    })
}

fn plan_commits(files: &[WorkingTreeFile]) -> Vec<CommitPlan> {
    if files.is_empty() {
        return Vec::new();
    }

    let mut groups = BTreeMap::<ChangeConcept, Vec<&WorkingTreeFile>>::new();
    let dominant_support_concept = files
        .iter()
        .filter(|file| !is_weak_support_file(&file.path))
        .map(|file| concept_for_path(&file.path))
        .filter(|concept| *concept != ChangeConcept::Docs && *concept != ChangeConcept::Other)
        .collect::<Vec<_>>();
    let dominant_support_concept = if dominant_support_concept.len() == 1 {
        dominant_support_concept.first().cloned()
    } else {
        None
    };

    for file in files {
        let concept = if is_weak_support_file(&file.path) {
            dominant_support_concept
                .clone()
                .unwrap_or_else(|| concept_for_path(&file.path))
        } else {
            concept_for_path(&file.path)
        };
        groups.entry(concept).or_default().push(file);
    }

    groups
        .into_iter()
        .map(|(concept, files)| plan_for_group(concept, files))
        .collect()
}

fn plan_for_group(concept: ChangeConcept, files: Vec<&WorkingTreeFile>) -> CommitPlan {
    let paths = files
        .iter()
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    let docs_only = paths.iter().all(|path| is_docs_file(path));
    let mut warnings = Vec::new();

    if paths.iter().any(|path| is_readme(path)) && paths.iter().any(|path| is_rust_source(path)) {
        warnings.push(
            "README changed with Rust source; confirm the docs describe the same change."
                .to_owned(),
        );
    }

    let (message, reason) = match concept {
        ChangeConcept::AgentsDocs => (
            "docs(agents): update agent documentation",
            "matched docs/agents.md",
        ),
        ChangeConcept::Skill => {
            if docs_only {
                (
                    "docs(skill): document workspace binary fallback",
                    "matched skills/koba/** documentation",
                )
            } else {
                ("feat(skill): update agent skill", "matched skills/koba/**")
            }
        }
        ChangeConcept::CommitEngine => (
            "feat(commit): sharpen path-based scope inference",
            "matched crates/koba/src/suggest_commit.rs",
        ),
        ChangeConcept::Output => (
            "feat(output): improve terminal rendering",
            "matched crates/koba/src/output.rs",
        ),
        ChangeConcept::Pr => ("feat(pr): update PR draft helper", "matched PR module"),
        ChangeConcept::Hooks => (
            "feat(hooks): update hook installation",
            "matched hooks module",
        ),
        ChangeConcept::Github => (
            "feat(github): update GitHub workflow infrastructure",
            "matched GitHub module",
        ),
        ChangeConcept::RunChecks => (
            "feat(run): update configured check execution",
            "matched run checks module",
        ),
        ChangeConcept::Init => (
            "feat(init): update workflow initialization",
            "matched init module",
        ),
        ChangeConcept::Changes => (
            "feat(changes): review working tree changes",
            "matched changes module",
        ),
        ChangeConcept::Scoop => (
            "chore(scoop): update Scoop packaging",
            "matched Scoop packaging paths",
        ),
        ChangeConcept::GithubCi => (
            "ci(github): update GitHub Actions workflow",
            "matched .github/workflows/**",
        ),
        ChangeConcept::GithubTemplate => (
            "docs(github): update pull request template",
            "matched .github/pull_request_template.md",
        ),
        ChangeConcept::Rust => (
            "feat: update Rust workflow tooling",
            "matched Rust source files",
        ),
        ChangeConcept::Docs => ("docs: update documentation", "matched documentation files"),
        ChangeConcept::Config => (
            "chore(config): update configuration",
            "matched config files",
        ),
        ChangeConcept::Other => ("chore: update project files", "matched uncategorized files"),
    };

    let confidence = if warnings.is_empty() {
        match concept {
            ChangeConcept::Other => Confidence::Low,
            ChangeConcept::Docs | ChangeConcept::Rust | ChangeConcept::Config => Confidence::Medium,
            _ => Confidence::High,
        }
    } else {
        Confidence::Low
    };

    CommitPlan {
        message: message.to_owned(),
        files: paths,
        confidence,
        reasons: vec![reason.to_owned()],
        warnings,
    }
}

fn recommend_checks(cwd: &PathBuf, files: &[WorkingTreeFile]) -> Vec<CheckRecommendation> {
    if files.is_empty() {
        return Vec::new();
    }

    let mut checks = Vec::new();
    let has_rust = files
        .iter()
        .any(|file| is_rust_source(&file.path) || is_cargo_file(&file.path));
    let has_scoop = files.iter().any(|file| is_scoop_manifest(&file.path));
    let has_github_workflow = files.iter().any(|file| is_github_workflow(&file.path));
    let has_js_ts = files.iter().any(|file| is_js_ts_source(&file.path));
    let has_python = files.iter().any(|file| is_python_source(&file.path));

    checks.push(CheckRecommendation {
        command: "git diff --check".to_owned(),
        reason: "changed files present".to_owned(),
    });

    if has_rust {
        let koba_workspace = cwd.join("crates").join("koba").join("Cargo.toml").is_file();
        let suffix = if koba_workspace { " -p koba" } else { "" };
        checks.push(CheckRecommendation {
            command: "cargo fmt --check".to_owned(),
            reason: "Rust source or Cargo files changed".to_owned(),
        });
        checks.push(CheckRecommendation {
            command: format!("cargo check{suffix}"),
            reason: "Rust source or Cargo files changed".to_owned(),
        });
        checks.push(CheckRecommendation {
            command: format!("cargo test{suffix}"),
            reason: "Rust source or Cargo files changed".to_owned(),
        });
    }

    if has_scoop {
        for file in files.iter().filter(|file| is_scoop_manifest(&file.path)) {
            checks.push(CheckRecommendation {
                command: format!("python -m json.tool {}", quote_path(&file.path)),
                reason: "Scoop manifest changed; parse JSON and verify release URL/hash manually"
                    .to_owned(),
            });
        }
    }

    if has_github_workflow {
        checks.push(CheckRecommendation {
            command: "review GitHub Actions workflow changes".to_owned(),
            reason: "workflow changes may require CI for final validation".to_owned(),
        });
    }

    if has_js_ts {
        checks.extend(node_script_checks(cwd));
    }

    if has_python {
        checks.extend(python_checks(cwd));
    }

    checks
}

fn node_script_checks(cwd: &PathBuf) -> Vec<CheckRecommendation> {
    let package_json = cwd.join("package.json");
    let contents = fs::read_to_string(package_json).unwrap_or_default();
    let mut checks = Vec::new();

    for (script, command) in [
        ("\"lint\"", "npm run lint"),
        ("\"test\"", "npm test"),
        ("\"build\"", "npm run build"),
    ] {
        if contents.contains(script) {
            checks.push(CheckRecommendation {
                command: command.to_owned(),
                reason: "JavaScript/TypeScript files changed and matching package script exists"
                    .to_owned(),
            });
        }
    }

    checks
}

fn python_checks(cwd: &PathBuf) -> Vec<CheckRecommendation> {
    let pyproject = fs::read_to_string(cwd.join("pyproject.toml")).unwrap_or_default();
    let mut checks = Vec::new();

    if cwd.join("pytest.ini").is_file()
        || cwd.join("tests").is_dir()
        || pyproject.to_ascii_lowercase().contains("pytest")
    {
        checks.push(CheckRecommendation {
            command: "pytest".to_owned(),
            reason: "Python files changed and pytest appears configured".to_owned(),
        });
    }

    if pyproject.to_ascii_lowercase().contains("ruff") {
        checks.push(CheckRecommendation {
            command: "ruff check .".to_owned(),
            reason: "Python files changed and Ruff appears configured".to_owned(),
        });
    }

    if pyproject.to_ascii_lowercase().contains("mypy") {
        checks.push(CheckRecommendation {
            command: "mypy .".to_owned(),
            reason: "Python files changed and mypy appears configured".to_owned(),
        });
    }

    checks
}

fn assess_risks(files: &[WorkingTreeFile], plans: &[CommitPlan]) -> Vec<Risk> {
    if files.is_empty() {
        return vec![Risk {
            status: Status::Ok,
            message: "working tree is clean".to_owned(),
        }];
    }

    let mut risks = Vec::new();

    if plans.len() > 1 {
        risks.push(Risk {
            status: Status::Warn,
            message: "working tree appears to contain multiple commit concepts".to_owned(),
        });
        risks.push(Risk {
            status: Status::Warn,
            message: "split commits before staging unless the diff proves one concept".to_owned(),
        });
    } else {
        risks.push(Risk {
            status: Status::Ok,
            message: "no mixed-change risk detected".to_owned(),
        });
    }

    for plan in plans {
        for warning in &plan.warnings {
            risks.push(Risk {
                status: Status::Warn,
                message: warning.clone(),
            });
        }
    }

    risks
}

fn render(report: &ChangesReport) -> String {
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
        ],
    );

    if !report.plans.is_empty() {
        writeln!(output).unwrap();
        output::section(&mut output, "Change groups", &plan_rows(&report.plans));
    }

    if !report.checks.is_empty() {
        writeln!(output).unwrap();
        output::section(&mut output, "Checks", &check_rows(&report.checks));
    }

    writeln!(output).unwrap();
    output::section(&mut output, "Risk", &risk_rows(&report.risks));

    writeln!(output).unwrap();
    writeln!(output, "Next steps").unwrap();
    if report.changed_count == 0 {
        writeln!(
            output,
            "{}",
            output::next_step("No commit preparation needed")
        )
        .unwrap();
    } else {
        writeln!(output, "{}", output::next_step("Inspect each group diff")).unwrap();
        writeln!(output, "{}", output::next_step("Run relevant checks")).unwrap();
        writeln!(
            output,
            "{}",
            output::next_step("Stage only the approved files")
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
            row = row.detail(format!("confidence: {}", confidence_label(plan.confidence)));
            for reason in &plan.reasons {
                row = row.detail(format!("reason: {reason}"));
            }
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

fn status_for_changed_count(report: &ChangesReport) -> Status {
    if report.changed_count == 0 {
        Status::Ok
    } else if report.plans.len() > 1 {
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

fn confidence_label(confidence: Confidence) -> &'static str {
    match confidence {
        Confidence::High => "high",
        Confidence::Medium => "medium",
        Confidence::Low => "low",
    }
}

fn concept_for_path(path: &str) -> ChangeConcept {
    let path = normalize(path);
    let file_name = path.rsplit('/').next().unwrap_or(path.as_str());

    if path == "docs/agents.md" {
        return ChangeConcept::AgentsDocs;
    }
    if path.starts_with("skills/koba/") {
        return ChangeConcept::Skill;
    }
    if path == "crates/koba/src/suggest_commit.rs" {
        return ChangeConcept::CommitEngine;
    }
    if path == "crates/koba/src/output.rs" {
        return ChangeConcept::Output;
    }
    if path == "crates/koba/src/pr.rs" {
        return ChangeConcept::Pr;
    }
    if path == "crates/koba/src/hooks.rs" {
        return ChangeConcept::Hooks;
    }
    if path == "crates/koba/src/github.rs" {
        return ChangeConcept::Github;
    }
    if path == "crates/koba/src/run_checks.rs" {
        return ChangeConcept::RunChecks;
    }
    if path == "crates/koba/src/init.rs" {
        return ChangeConcept::Init;
    }
    if path == "crates/koba/src/changes.rs" {
        return ChangeConcept::Changes;
    }
    if path.starts_with("packaging/scoop/")
        || (path.starts_with("bucket/") && file_name.ends_with(".json"))
    {
        return ChangeConcept::Scoop;
    }
    if is_github_workflow(&path) {
        return ChangeConcept::GithubCi;
    }
    if path == ".github/pull_request_template.md" {
        return ChangeConcept::GithubTemplate;
    }
    if is_rust_source(&path) {
        return ChangeConcept::Rust;
    }
    if is_docs_file(&path) {
        return ChangeConcept::Docs;
    }
    if is_config_file(&path) {
        return ChangeConcept::Config;
    }

    ChangeConcept::Other
}

fn normalize(path: &str) -> String {
    path.replace('\\', "/").to_ascii_lowercase()
}

fn is_readme(path: &str) -> bool {
    normalize(path) == "readme.md"
}

fn is_weak_support_file(path: &str) -> bool {
    let path = normalize(path);
    is_readme(&path)
        || path == "crates/koba/src/cli.rs"
        || path == "crates/koba/src/commands.rs"
        || path == "crates/koba/src/lib.rs"
        || path == "crates/koba/tests/cli.rs"
}

fn is_docs_file(path: &str) -> bool {
    let path = normalize(path);
    path.starts_with("docs/")
        || path.ends_with(".md")
        || path.ends_with(".mdx")
        || path.ends_with(".rst")
        || path == ".github/pull_request_template.md"
}

fn is_rust_source(path: &str) -> bool {
    normalize(path).ends_with(".rs")
}

fn is_cargo_file(path: &str) -> bool {
    let path = normalize(path);
    path.ends_with("cargo.toml") || path.ends_with("cargo.lock")
}

fn is_scoop_manifest(path: &str) -> bool {
    let path = normalize(path);
    (path.starts_with("packaging/scoop/") || path.starts_with("bucket/")) && path.ends_with(".json")
}

fn is_github_workflow(path: &str) -> bool {
    normalize(path).starts_with(".github/workflows/")
}

fn is_js_ts_source(path: &str) -> bool {
    let path = normalize(path);
    path.ends_with(".js")
        || path.ends_with(".jsx")
        || path.ends_with(".ts")
        || path.ends_with(".tsx")
        || path.ends_with(".mjs")
        || path.ends_with(".cjs")
}

fn is_python_source(path: &str) -> bool {
    normalize(path).ends_with(".py")
}

fn is_config_file(path: &str) -> bool {
    let path = normalize(path);
    path.ends_with(".toml")
        || path.ends_with(".yml")
        || path.ends_with(".yaml")
        || path.ends_with(".json")
        || path == ".gitignore"
        || path == ".gitattributes"
}

fn quote_path(path: &str) -> String {
    format!("\"{}\"", path.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        path::{Path, PathBuf},
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn file(path: &str) -> WorkingTreeFile {
        WorkingTreeFile {
            path: path.to_owned(),
            status: "M".to_owned(),
            staged: false,
            unstaged: true,
            untracked: false,
        }
    }

    #[test]
    fn parses_status_counts_without_mutation() {
        let files = parse_status_porcelain("M  staged.rs\n M unstaged.rs\n?? new.md\n");

        let report = analyze(&PathBuf::from("."), &files);

        assert_eq!(report.changed_count, 3);
        assert_eq!(report.staged_count, 1);
        assert_eq!(report.unstaged_count, 1);
        assert_eq!(report.untracked_count, 1);
    }

    #[test]
    fn clean_tree_has_no_commit_plan() {
        let fixture = TempTree::new();
        fixture.git_init();

        let output = execute(fixture.path().to_path_buf()).unwrap();

        assert!(output.contains("changed files"));
        assert!(output.contains("working tree is clean"));
        assert!(!output.contains("Change groups"));
    }

    #[test]
    fn non_git_directory_errors() {
        let fixture = TempTree::new();

        let error = execute(fixture.path().to_path_buf()).unwrap_err();

        assert_eq!(error, "not inside a Git repository");
    }

    #[test]
    fn docs_only_readme_and_agent_docs_stay_docs_agents() {
        let report = analyze(
            &PathBuf::from("."),
            &[file("README.md"), file("docs/agents.md")],
        );

        assert_eq!(report.plans.len(), 1);
        assert_eq!(
            report.plans[0].message,
            "docs(agents): update agent documentation"
        );
        assert!(report
            .checks
            .iter()
            .any(|check| check.command == "git diff --check"));
        assert!(!report
            .checks
            .iter()
            .any(|check| check.command.starts_with("cargo ")));
    }

    #[test]
    fn skill_docs_and_commit_engine_split_into_multiple_groups() {
        let report = analyze(
            &PathBuf::from("."),
            &[
                file("skills/koba/SKILL.md"),
                file("skills/koba/references/workflows.md"),
                file("crates/koba/src/suggest_commit.rs"),
            ],
        );

        let messages = report
            .plans
            .iter()
            .map(|plan| plan.message.as_str())
            .collect::<Vec<_>>();

        assert!(messages.contains(&"docs(skill): document workspace binary fallback"));
        assert!(messages.contains(&"feat(commit): sharpen path-based scope inference"));
        assert!(report.risks.iter().any(|risk| {
            risk.status == Status::Warn && risk.message.contains("multiple commit concepts")
        }));
        assert!(!messages
            .iter()
            .all(|message| message.contains("feat(skill)")));
    }

    #[test]
    fn scoop_manifest_recommends_packaging_and_json_hash_review() {
        let report = analyze(
            &PathBuf::from("."),
            &[file("packaging/scoop/bucket/koba.json")],
        );

        assert_eq!(
            report.plans[0].message,
            "chore(scoop): update Scoop packaging"
        );
        assert!(report
            .checks
            .iter()
            .any(|check| check.command.contains("json.tool")
                && check.reason.contains("release URL/hash")));
    }

    #[test]
    fn github_workflow_recommends_ci_review() {
        let report = analyze(&PathBuf::from("."), &[file(".github/workflows/ci.yml")]);

        assert_eq!(
            report.plans[0].message,
            "ci(github): update GitHub Actions workflow"
        );
        assert!(report
            .checks
            .iter()
            .any(|check| check.reason.contains("require CI")));
    }

    #[test]
    fn rust_source_recommends_koba_workspace_checks_when_detected() {
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap()
            .to_path_buf();
        let report = analyze(&cwd, &[file("crates/koba/src/output.rs")]);

        assert!(report
            .checks
            .iter()
            .any(|check| check.command == "cargo fmt --check"));
        assert!(report
            .checks
            .iter()
            .any(|check| check.command == "cargo check -p koba"));
        assert!(report
            .checks
            .iter()
            .any(|check| check.command == "cargo test -p koba"));
    }

    #[test]
    fn mixed_rust_and_readme_warns_when_grouped_together() {
        let report = analyze(
            &PathBuf::from("."),
            &[file("crates/koba/src/output.rs"), file("README.md")],
        );

        assert_eq!(report.plans.len(), 1);
        assert!(report
            .risks
            .iter()
            .any(|risk| risk.message.contains("README changed with Rust source")));
    }

    #[test]
    fn cli_support_files_follow_new_changes_command_group() {
        let report = analyze(
            &PathBuf::from("."),
            &[
                file("crates/koba/src/changes.rs"),
                file("crates/koba/src/cli.rs"),
                file("crates/koba/src/commands.rs"),
                file("crates/koba/src/lib.rs"),
                file("crates/koba/tests/cli.rs"),
            ],
        );

        assert_eq!(report.plans.len(), 1);
        assert_eq!(
            report.plans[0].message,
            "feat(changes): review working tree changes"
        );
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
