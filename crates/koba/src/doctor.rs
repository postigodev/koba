use std::{fmt::Write, path::PathBuf};

use crate::{
    git::GitInfo,
    output::{self, Status},
    repo::{GithubFiles, WorkflowFiles},
    scan::ScanReport,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Repository,
    Workflow,
    Project,
    Hooks,
    Github,
}

impl Section {
    fn title(&self) -> &'static str {
        match self {
            Section::Repository => "Repository",
            Section::Workflow => "Workflow contract",
            Section::Project => "Project type hints",
            Section::Hooks => "Hooks",
            Section::Github => "GitHub workflow surface",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Ok,
    Warning,
    Error,
}

impl Severity {
    fn status(&self) -> Status {
        match self {
            Severity::Ok => Status::Ok,
            Severity::Warning => Status::Warn,
            Severity::Error => Status::Error,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub section: Section,
    pub severity: Severity,
    pub message: String,
    pub recommendation: Option<String>,
}

impl Diagnostic {
    fn ok(section: Section, message: impl Into<String>) -> Self {
        Self {
            section,
            severity: Severity::Ok,
            message: message.into(),
            recommendation: None,
        }
    }

    fn warn(
        section: Section,
        message: impl Into<String>,
        recommendation: impl Into<String>,
    ) -> Self {
        Self {
            section,
            severity: Severity::Warning,
            message: message.into(),
            recommendation: Some(recommendation.into()),
        }
    }

    fn error(
        section: Section,
        message: impl Into<String>,
        recommendation: impl Into<String>,
    ) -> Self {
        Self {
            section,
            severity: Severity::Error,
            message: message.into(),
            recommendation: Some(recommendation.into()),
        }
    }
}

pub fn run(cwd: PathBuf) -> Result<(), String> {
    let report = ScanReport::from_cwd(cwd);
    let diagnostics = diagnose(&report);
    print!("{}", render(&diagnostics));
    Ok(())
}

pub fn diagnose(report: &ScanReport) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    diagnose_repository(&report.git, &mut diagnostics);
    diagnose_workflow(&report.workflow, &mut diagnostics);
    diagnose_project(&report.workflow, &mut diagnostics);
    diagnose_hooks(&report.workflow, &mut diagnostics);
    diagnose_github(&report.github, &mut diagnostics);

    diagnostics
}

pub fn render(diagnostics: &[Diagnostic]) -> String {
    let mut output = String::new();

    writeln!(output, "Koba doctor").unwrap();

    for section in [
        Section::Repository,
        Section::Workflow,
        Section::Project,
        Section::Hooks,
        Section::Github,
    ] {
        let section_diagnostics: Vec<_> = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.section == section)
            .collect();

        if section_diagnostics.is_empty() {
            continue;
        }

        writeln!(output).unwrap();
        writeln!(output, "{}", section.title()).unwrap();

        let rows = section_diagnostics
            .iter()
            .map(|diagnostic| output::row(diagnostic.severity.status(), &diagnostic.message))
            .collect::<Vec<_>>();
        output.push_str(&output::render_rows(&rows));
    }

    let recommendations: Vec<_> = diagnostics
        .iter()
        .filter_map(|diagnostic| diagnostic.recommendation.as_deref())
        .collect();

    if !recommendations.is_empty() {
        writeln!(output).unwrap();
        writeln!(output, "Next steps").unwrap();

        for recommendation in recommendations {
            writeln!(output, "{}", output::next_step(recommendation)).unwrap();
        }
    }

    output
}

fn diagnose_repository(git: &GitInfo, diagnostics: &mut Vec<Diagnostic>) {
    if git.inside_repo {
        diagnostics.push(Diagnostic::ok(
            Section::Repository,
            "Git repository detected",
        ));
    } else {
        diagnostics.push(Diagnostic::error(
            Section::Repository,
            "Not inside a Git repository",
            "Run Koba from a Git repository or initialize one with `git init`",
        ));
        return;
    }

    if git.has_user_name {
        diagnostics.push(Diagnostic::ok(
            Section::Repository,
            "Git user.name configured",
        ));
    } else {
        diagnostics.push(Diagnostic::warn(
            Section::Repository,
            "Git user.name is not configured",
            "Configure `git config user.name` for this repository or globally",
        ));
    }

    if git.has_user_email {
        diagnostics.push(Diagnostic::ok(
            Section::Repository,
            "Git user.email configured",
        ));
    } else {
        diagnostics.push(Diagnostic::warn(
            Section::Repository,
            "Git user.email is not configured",
            "Configure `git config user.email` for this repository or globally",
        ));
    }

    if git.has_origin {
        diagnostics.push(Diagnostic::ok(
            Section::Repository,
            "origin remote detected",
        ));
    } else {
        diagnostics.push(Diagnostic::warn(
            Section::Repository,
            "origin remote is not configured",
            "Add an `origin` remote when this repository should sync with a host",
        ));
    }
}

fn diagnose_workflow(workflow: &WorkflowFiles, diagnostics: &mut Vec<Diagnostic>) {
    if workflow.koba_yml {
        diagnostics.push(Diagnostic::ok(
            Section::Workflow,
            "koba.yml workflow contract present",
        ));
    } else {
        diagnostics.push(Diagnostic::warn(
            Section::Workflow,
            "koba.yml workflow contract is missing",
            "Run `koba init` to create a workflow contract",
        ));
    }
}

fn diagnose_project(workflow: &WorkflowFiles, diagnostics: &mut Vec<Diagnostic>) {
    if workflow.cargo_toml {
        diagnostics.push(Diagnostic::ok(
            Section::Project,
            "Rust project detected from Cargo.toml",
        ));
        diagnostics.push(Diagnostic::warn(
            Section::Project,
            "Rust workflow checks are not modeled yet",
            "Consider checks such as `cargo fmt --check` and `cargo test`",
        ));
    }

    if workflow.package_json {
        diagnostics.push(Diagnostic::ok(
            Section::Project,
            "JavaScript or TypeScript project detected from package.json",
        ));
        diagnostics.push(Diagnostic::warn(
            Section::Project,
            "package.json scripts are not parsed yet",
            "Consider lint, test, and build scripts for the workflow contract",
        ));
    }

    if workflow.pyproject_toml {
        diagnostics.push(Diagnostic::ok(
            Section::Project,
            "Python project detected from pyproject.toml",
        ));
        diagnostics.push(Diagnostic::warn(
            Section::Project,
            "Python project checks are not modeled yet",
            "Consider formatting, linting, type-checking, and test commands",
        ));
    }

    if !workflow.cargo_toml && !workflow.package_json && !workflow.pyproject_toml {
        diagnostics.push(Diagnostic::warn(
            Section::Project,
            "No supported project manifest detected",
            "Add workflow checks manually once the project type is known",
        ));
    }
}

fn diagnose_hooks(workflow: &WorkflowFiles, diagnostics: &mut Vec<Diagnostic>) {
    if !workflow.husky_dir && !workflow.native_pre_commit && !workflow.native_pre_push {
        diagnostics.push(Diagnostic::warn(
            Section::Hooks,
            "No hook adapter surface detected",
            "Add a pre-commit or pre-push check through native Git hooks or Husky",
        ));
        return;
    }

    if workflow.husky_dir {
        diagnostics.push(Diagnostic::ok(
            Section::Hooks,
            "Husky adapter surface present",
        ));
    }

    if workflow.native_pre_commit {
        diagnostics.push(Diagnostic::ok(
            Section::Hooks,
            "Native pre-commit hook present",
        ));
    }

    if workflow.native_pre_push {
        diagnostics.push(Diagnostic::ok(
            Section::Hooks,
            "Native pre-push hook present",
        ));
    }
}

fn diagnose_github(github: &GithubFiles, diagnostics: &mut Vec<Diagnostic>) {
    if github.github_dir {
        diagnostics.push(Diagnostic::ok(Section::Github, ".github directory present"));
    } else {
        diagnostics.push(Diagnostic::warn(
            Section::Github,
            ".github directory is missing",
            "Add `.github/` assets when this repository needs hosted workflow conventions",
        ));
        return;
    }

    if github.workflows_dir {
        diagnostics.push(Diagnostic::ok(Section::Github, ".github/workflows present"));
    } else {
        diagnostics.push(Diagnostic::warn(
            Section::Github,
            ".github/workflows is missing",
            "Add GitHub Actions workflows for hosted CI when appropriate",
        ));
    }

    if github.pull_request_template {
        diagnostics.push(Diagnostic::ok(
            Section::Github,
            ".github/pull_request_template.md present",
        ));
    } else {
        diagnostics.push(Diagnostic::warn(
            Section::Github,
            ".github/pull_request_template.md is missing",
            "Add a pull request template to make review expectations explicit",
        ));
    }

    if !github.issue_template_dir {
        diagnostics.push(Diagnostic::warn(
            Section::Github,
            ".github/ISSUE_TEMPLATE is not present",
            "Consider issue templates if this repository accepts issues",
        ));
    }

    if !github.codeowners {
        diagnostics.push(Diagnostic::warn(
            Section::Github,
            ".github/CODEOWNERS is not present",
            "Consider CODEOWNERS when reviews should route to specific owners",
        ));
    }

    if !github.dependabot_yml {
        diagnostics.push(Diagnostic::warn(
            Section::Github,
            ".github/dependabot.yml is not present",
            "Consider Dependabot config for dependency update hygiene",
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(workflow: WorkflowFiles, github: GithubFiles, git: GitInfo) -> ScanReport {
        ScanReport {
            git,
            workflow,
            github,
        }
    }

    fn git_info() -> GitInfo {
        GitInfo {
            inside_repo: true,
            root: None,
            git_dir: None,
            branch: Some("main".to_owned()),
            has_origin: true,
            has_user_name: true,
            has_user_email: true,
        }
    }

    fn workflow_files() -> WorkflowFiles {
        WorkflowFiles {
            koba_yml: false,
            package_json: false,
            cargo_toml: false,
            pyproject_toml: false,
            husky_dir: false,
            native_pre_commit: false,
            native_pre_push: false,
        }
    }

    fn github_files() -> GithubFiles {
        GithubFiles {
            github_dir: false,
            workflows_dir: false,
            pull_request_template: false,
            issue_template_dir: false,
            codeowners: false,
            dependabot_yml: false,
        }
    }

    #[test]
    fn diagnoses_missing_repository_as_error() {
        let mut git = git_info();
        git.inside_repo = false;
        git.has_origin = false;
        git.has_user_name = false;
        git.has_user_email = false;

        let diagnostics = diagnose(&report(workflow_files(), github_files(), git));

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.section == Section::Repository
                && diagnostic.severity == Severity::Error
                && diagnostic.message == "Not inside a Git repository"
        }));
    }

    #[test]
    fn diagnoses_missing_contract_hooks_and_github_surface() {
        let diagnostics = diagnose(&report(workflow_files(), github_files(), git_info()));

        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "koba.yml workflow contract is missing"));
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "No hook adapter surface detected"));
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == ".github directory is missing"));
    }

    #[test]
    fn diagnoses_project_hints_and_present_hook_surfaces() {
        let workflow = WorkflowFiles {
            koba_yml: true,
            package_json: true,
            cargo_toml: true,
            pyproject_toml: true,
            husky_dir: true,
            native_pre_commit: true,
            native_pre_push: true,
        };
        let github = GithubFiles {
            github_dir: true,
            workflows_dir: true,
            pull_request_template: true,
            issue_template_dir: true,
            codeowners: true,
            dependabot_yml: true,
        };

        let diagnostics = diagnose(&report(workflow, github, git_info()));

        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "Rust project detected from Cargo.toml"));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.message
            == "JavaScript or TypeScript project detected from package.json"));
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "Python project detected from pyproject.toml"));
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "Husky adapter surface present"));
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "Native pre-commit hook present"));
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "Native pre-push hook present"));
    }
}
