use std::{fmt::Write, path::PathBuf};

use crate::{
    git,
    output::{self, Status, StatusRow},
    repo::{self, AgentSkillFiles, GithubFiles, WorkflowFiles},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanReport {
    pub git: git::GitInfo,
    pub workflow: WorkflowFiles,
    pub github: GithubFiles,
    pub agent_skill: AgentSkillFiles,
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
            agent_skill: files.agent_skill,
        }
    }

    pub fn render(&self) -> String {
        let mut output = String::new();

        writeln!(output, "Koba scan").unwrap();
        writeln!(output).unwrap();
        self.render_repository(&mut output);
        writeln!(output).unwrap();
        self.render_project(&mut output);
        writeln!(output).unwrap();
        self.render_workflow(&mut output);
        writeln!(output).unwrap();
        self.render_github(&mut output);
        writeln!(output).unwrap();
        self.render_next_steps(&mut output);

        output
    }

    fn render_project(&self, output: &mut String) {
        if !self.agent_skill.detected() {
            return;
        }

        let mut rows = vec![
            output::row(Status::Ok, "Profile").value("agent-skill"),
            output::row(Status::Ok, "Skills").value(self.agent_skill.skills.len().to_string()),
        ];

        for skill in &self.agent_skill.skills {
            rows.push(output::row(Status::Ok, "Skill").value(&skill.name));
            rows.push(if skill.references_dir {
                output::row(Status::Ok, "References").value("detected")
            } else {
                output::row(Status::Miss, "References")
            });
            rows.push(if skill.examples_dir {
                output::row(Status::Ok, "Examples").value("detected")
            } else {
                output::row(Status::Miss, "Examples")
            });
        }

        rows.push(if self.agent_skill.evals_dir {
            output::row(Status::Ok, "Evals").value("detected")
        } else {
            output::row(Status::Miss, "Evals")
        });
        rows.push(if self.agent_skill.smoke_prompts {
            output::row(Status::Ok, "Smoke prompts").value("tests/smoke-prompts.md")
        } else {
            output::row(Status::Miss, "Smoke prompts")
        });
        rows.push(if self.agent_skill.readme {
            output::row(Status::Ok, "README")
        } else {
            output::row(Status::Miss, "README")
        });

        output::section(output, "Project", &rows);
    }

    fn render_repository(&self, output: &mut String) {
        let mut rows = Vec::new();

        if self.git.inside_repo {
            rows.push(output::row(Status::Ok, "Git repository"));
        } else {
            rows.push(output::row(Status::Miss, "Git repository").value("not detected"));
        }

        match &self.git.branch {
            Some(branch) => {
                rows.push(output::row(Status::Ok, "Branch").value(branch));
            }
            None if self.git.inside_repo => {
                rows.push(output::row(Status::Warn, "Branch").value("not available"));
            }
            None => {}
        }

        if self.git.has_origin {
            rows.push(output::row(Status::Ok, "Remote").value("origin"));
        } else if self.git.inside_repo {
            rows.push(output::row(Status::Warn, "Remote").value("origin not configured"));
        }

        if self.git.inside_repo {
            rows.push(config_row(self.git.has_user_name, "Git user.name"));
            rows.push(config_row(self.git.has_user_email, "Git user.email"));
        }

        output::section(output, "Repository", &rows);
    }

    fn render_workflow(&self, output: &mut String) {
        let mut rows = vec![file_row(self.workflow.koba_yml, "koba.yml")];

        if !self.agent_skill.detected() {
            rows.extend([
                file_row(self.workflow.package_json, "package.json"),
                file_row(self.workflow.cargo_toml, "Cargo.toml"),
                file_row(self.workflow.pyproject_toml, "pyproject.toml"),
            ]);
        }

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
            rows.push(output::row(Status::Miss, "Hooks"));
        } else {
            rows.push(output::row(Status::Ok, "Hooks").value(hook_sources.join(", ")));
        }

        output::section(output, "Workflow", &rows);
    }

    fn render_github(&self, output: &mut String) {
        if !self.github.github_dir {
            output::section(
                output,
                "GitHub",
                &[output::row(Status::Miss, ".github/").value("not found")],
            );
            return;
        }

        let rows = [
            output::row(Status::Ok, ".github/"),
            file_row(self.github.workflows_dir, "workflows/"),
            file_row(
                self.github.pull_request_template,
                "pull_request_template.md",
            ),
            file_row(self.github.issue_template_dir, "ISSUE_TEMPLATE/"),
            file_row(self.github.codeowners, "CODEOWNERS"),
            file_row(self.github.dependabot_yml, "dependabot.yml"),
        ];
        output::section(output, "GitHub", &rows);
    }

    fn render_next_steps(&self, output: &mut String) {
        writeln!(output, "Next steps").unwrap();

        if !self.workflow.koba_yml {
            writeln!(
                output,
                "{}",
                output::next_step("Run `koba init` to create a workflow contract")
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
                output::next_step("Add a pre-commit check for formatting/tests")
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
                output::next_step("Review scan findings before applying workflow changes")
            )
            .unwrap();
        }
    }
}

fn file_row(present: bool, label: &str) -> StatusRow {
    output::row(if present { Status::Ok } else { Status::Miss }, label)
}

fn config_row(present: bool, label: &str) -> StatusRow {
    if present {
        output::row(Status::Ok, label)
    } else {
        output::row(Status::Warn, label).value("not configured")
    }
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
            agent_skill: AgentSkillFiles {
                skills: Vec::new(),
                evals_dir: false,
                smoke_prompts: false,
                readme: false,
            },
        };

        let rendered = report.render();

        assert!(rendered.contains("Cargo.toml"));
        assert!(rendered.contains("pull_request_template.md"));
        assert!(rendered.contains("Git user.email  not configured"));
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
            agent_skill: AgentSkillFiles {
                skills: Vec::new(),
                evals_dir: false,
                smoke_prompts: false,
                readme: false,
            },
        };

        let rendered = report.render();

        assert!(rendered.contains("Git repository  not detected"));
        assert!(!rendered.contains("Git user.name"));
        assert!(!rendered.contains("Git user.email"));
        assert!(!rendered.contains("Remote origin not configured"));
    }

    #[test]
    fn render_includes_agent_skill_project_section() {
        let report = ScanReport {
            git: git::GitInfo {
                inside_repo: true,
                root: None,
                git_dir: None,
                branch: Some("main".to_owned()),
                has_origin: true,
                has_user_name: true,
                has_user_email: true,
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
            agent_skill: AgentSkillFiles {
                skills: vec![repo::AgentSkill {
                    name: "hoi4-modding".to_owned(),
                    references_dir: true,
                    examples_dir: true,
                }],
                evals_dir: true,
                smoke_prompts: true,
                readme: true,
            },
        };

        let rendered = report.render();

        assert!(rendered.contains("Profile        agent-skill"));
        assert!(rendered.contains("Skill          hoi4-modding"));
        assert!(rendered.contains("Smoke prompts  tests/smoke-prompts.md"));
        assert!(!rendered.contains("package.json"));
        assert!(!rendered.contains("Cargo.toml"));
        assert!(!rendered.contains("pyproject.toml"));
    }

    #[test]
    fn detects_agent_skill_profile_from_fixture_tree() {
        let fixture = TempTree::new();
        fixture.file("skills/hoi4-modding/SKILL.md");
        fixture.dir("skills/hoi4-modding/examples");
        fixture.file("skills/rust-cli-review/SKILL.md");
        fixture.dir("evals");
        fixture.file("tests/smoke-prompts.md");
        fixture.file("README.md");

        let report = ScanReport::from_cwd(fixture.path().to_path_buf());
        let rendered = report.render();

        assert_eq!(report.agent_skill.skills.len(), 2);
        assert!(rendered.contains("Profile        agent-skill"));
        assert!(rendered.contains("Skill          hoi4-modding"));
        assert!(rendered.contains("Skill          rust-cli-review"));
        assert!(!rendered.contains("no supported project manifest"));
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
            let path = std::env::temp_dir().join(format!("koba-scan-test-{id}"));
            fs::create_dir(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }

        fn dir(&self, relative: &str) {
            fs::create_dir_all(self.path.join(relative)).unwrap();
        }

        fn file(&self, relative: &str) {
            let path = self.path.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, "").unwrap();
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
