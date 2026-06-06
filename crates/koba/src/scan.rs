use std::{fmt::Write, path::PathBuf};

use crate::{
    git,
    output::{self, Status},
    repo::{self, GithubFiles, WorkflowFiles},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanReport {
    pub git: git::GitInfo,
    pub workflow: WorkflowFiles,
    pub github: GithubFiles,
}

pub fn run(cwd: PathBuf) -> Result<(), String> {
    let report = ScanReport::from_cwd(cwd);
    print!("{}", report.render());
    Ok(())
}

impl ScanReport {
    pub fn from_cwd(cwd: PathBuf) -> Self {
        let git = git::inspect(&cwd);
        let root = git.root.as_deref().unwrap_or(&cwd);
        let files = repo::discover(root, git.git_dir.as_deref());

        Self {
            git,
            workflow: files.workflow,
            github: files.github,
        }
    }

    pub fn render(&self) -> String {
        let mut output = String::new();

        writeln!(output, "Koba scan").unwrap();
        writeln!(output).unwrap();
        self.render_repository(&mut output);
        writeln!(output).unwrap();
        self.render_workflow(&mut output);
        writeln!(output).unwrap();
        self.render_github(&mut output);
        writeln!(output).unwrap();
        self.render_next_steps(&mut output);

        output
    }

    fn render_repository(&self, output: &mut String) {
        writeln!(output, "Repository").unwrap();

        if self.git.inside_repo {
            writeln!(
                output,
                "{}",
                output::line(Status::Ok, "Git repository detected")
            )
            .unwrap();
        } else {
            writeln!(
                output,
                "{}",
                output::line(Status::Missing, "Git repository not detected")
            )
            .unwrap();
        }

        match &self.git.branch {
            Some(branch) => {
                writeln!(
                    output,
                    "{}",
                    output::line(Status::Ok, format!("Branch: {branch}"))
                )
                .unwrap();
            }
            None if self.git.inside_repo => {
                writeln!(
                    output,
                    "{}",
                    output::line(Status::Warning, "Branch not available")
                )
                .unwrap();
            }
            None => {}
        }

        if self.git.has_origin {
            writeln!(output, "{}", output::line(Status::Ok, "Remote: origin")).unwrap();
        } else if self.git.inside_repo {
            writeln!(
                output,
                "{}",
                output::line(Status::Warning, "Remote origin not configured")
            )
            .unwrap();
        }

        if self.git.inside_repo {
            render_config_status(output, self.git.has_user_name, "Git user.name configured");
            render_config_status(output, self.git.has_user_email, "Git user.email configured");
        }
    }

    fn render_workflow(&self, output: &mut String) {
        writeln!(output, "Workflow").unwrap();
        render_file_status(
            output,
            self.workflow.koba_yml,
            "koba.yml found",
            "koba.yml not found",
        );
        render_file_status(
            output,
            self.workflow.package_json,
            "package.json found",
            "package.json not found",
        );
        render_file_status(
            output,
            self.workflow.cargo_toml,
            "Cargo.toml found",
            "Cargo.toml not found",
        );
        render_file_status(
            output,
            self.workflow.pyproject_toml,
            "pyproject.toml found",
            "pyproject.toml not found",
        );

        let mut hook_sources = Vec::new();
        if self.workflow.husky_dir {
            hook_sources.push("Husky");
        }
        if self.workflow.native_pre_commit {
            hook_sources.push("native pre-commit");
        }
        if self.workflow.native_pre_push {
            hook_sources.push("native pre-push");
        }

        if hook_sources.is_empty() {
            writeln!(
                output,
                "{}",
                output::line(Status::Missing, "no hooks detected")
            )
            .unwrap();
        } else {
            writeln!(
                output,
                "{}",
                output::line(Status::Ok, format!("Hooks: {}", hook_sources.join(", ")))
            )
            .unwrap();
        }
    }

    fn render_github(&self, output: &mut String) {
        writeln!(output, "GitHub").unwrap();

        if !self.github.github_dir {
            writeln!(
                output,
                "{}",
                output::line(Status::Missing, ".github directory not found")
            )
            .unwrap();
            return;
        }

        writeln!(
            output,
            "{}",
            output::line(Status::Ok, ".github directory found")
        )
        .unwrap();
        render_file_status(
            output,
            self.github.workflows_dir,
            ".github/workflows found",
            ".github/workflows not found",
        );
        render_file_status(
            output,
            self.github.pull_request_template,
            ".github/pull_request_template.md found",
            ".github/pull_request_template.md not found",
        );
        render_file_status(
            output,
            self.github.issue_template_dir,
            ".github/ISSUE_TEMPLATE found",
            ".github/ISSUE_TEMPLATE not found",
        );
        render_file_status(
            output,
            self.github.codeowners,
            ".github/CODEOWNERS found",
            ".github/CODEOWNERS not found",
        );
        render_file_status(
            output,
            self.github.dependabot_yml,
            ".github/dependabot.yml found",
            ".github/dependabot.yml not found",
        );
    }

    fn render_next_steps(&self, output: &mut String) {
        writeln!(output, "Next steps").unwrap();

        if !self.workflow.koba_yml {
            writeln!(
                output,
                "{}",
                output::line(
                    Status::Step,
                    "Run `koba init` to create a workflow contract"
                )
            )
            .unwrap();
        }

        if !self.workflow.native_pre_commit
            && !self.workflow.native_pre_push
            && !self.workflow.husky_dir
        {
            writeln!(
                output,
                "{}",
                output::line(Status::Step, "Add a pre-commit check for formatting/tests")
            )
            .unwrap();
        }

        if self.workflow.koba_yml
            && (self.workflow.native_pre_commit
                || self.workflow.native_pre_push
                || self.workflow.husky_dir)
        {
            writeln!(
                output,
                "{}",
                output::line(
                    Status::Step,
                    "Review scan findings before applying workflow changes"
                )
            )
            .unwrap();
        }
    }
}

fn render_file_status(output: &mut String, present: bool, found: &str, missing: &str) {
    let status = if present { Status::Ok } else { Status::Missing };
    let text = if present { found } else { missing };
    writeln!(output, "{}", output::line(status, text)).unwrap();
}

fn render_config_status(output: &mut String, present: bool, configured: &str) {
    if present {
        writeln!(output, "{}", output::line(Status::Ok, configured)).unwrap();
    } else {
        writeln!(
            output,
            "{}",
            output::line(
                Status::Warning,
                configured.replace("configured", "not configured")
            )
        )
        .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_includes_github_pull_request_template_detection() {
        let report = ScanReport {
            git: git::GitInfo {
                inside_repo: true,
                root: None,
                git_dir: None,
                branch: Some("main".to_owned()),
                has_origin: true,
                has_user_name: true,
                has_user_email: false,
            },
            workflow: WorkflowFiles {
                koba_yml: false,
                package_json: false,
                cargo_toml: true,
                pyproject_toml: false,
                husky_dir: false,
                native_pre_commit: false,
                native_pre_push: false,
            },
            github: GithubFiles {
                github_dir: true,
                workflows_dir: false,
                pull_request_template: true,
                issue_template_dir: false,
                codeowners: false,
                dependabot_yml: false,
            },
        };

        let rendered = report.render();

        assert!(rendered.contains("Cargo.toml found"));
        assert!(rendered.contains(".github/pull_request_template.md found"));
        assert!(rendered.contains("Git user.email not configured"));
    }

    #[test]
    fn render_outside_git_repo_omits_git_identity_warnings() {
        let report = ScanReport {
            git: git::GitInfo {
                inside_repo: false,
                root: None,
                git_dir: None,
                branch: None,
                has_origin: false,
                has_user_name: false,
                has_user_email: false,
            },
            workflow: WorkflowFiles {
                koba_yml: false,
                package_json: false,
                cargo_toml: false,
                pyproject_toml: false,
                husky_dir: false,
                native_pre_commit: false,
                native_pre_push: false,
            },
            github: GithubFiles {
                github_dir: false,
                workflows_dir: false,
                pull_request_template: false,
                issue_template_dir: false,
                codeowners: false,
                dependabot_yml: false,
            },
        };

        let rendered = report.render();

        assert!(rendered.contains("Git repository not detected"));
        assert!(!rendered.contains("Git user.name"));
        assert!(!rendered.contains("Git user.email"));
        assert!(!rendered.contains("Remote origin not configured"));
    }
}
