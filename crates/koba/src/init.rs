use std::{
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    output::{self, Status},
    repo::{self, AgentSkillFiles, RepoFiles},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitOptions {
    pub apply: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitOutcome {
    Preview {
        profile: Profile,
        yaml: String,
    },
    Applied {
        profile: Profile,
        path: PathBuf,
        yaml: String,
    },
    RefusedExisting {
        path: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    RustCli,
    Node,
    Python,
    AgentSkill,
    Mixed,
    Custom,
}

impl Profile {
    fn name(&self) -> &'static str {
        match self {
            Profile::RustCli => "rust-cli",
            Profile::Node => "node",
            Profile::Python => "python",
            Profile::AgentSkill => "agent-skill",
            Profile::Mixed => "mixed",
            Profile::Custom => "custom",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowContract {
    profile: Profile,
    pre_commit: Vec<&'static str>,
    pre_push: Vec<&'static str>,
    notes: Vec<&'static str>,
}

impl WorkflowContract {
    fn from_profile(profile: Profile, agent_skill: &AgentSkillFiles) -> Self {
        match profile {
            Profile::RustCli => Self {
                profile,
                pre_commit: vec!["cargo fmt --check"],
                pre_push: vec!["cargo test"],
                notes: Vec::new(),
            },
            Profile::Node => Self {
                profile,
                pre_commit: vec!["npm test"],
                pre_push: vec!["npm run build"],
                notes: Vec::new(),
            },
            Profile::Python => Self {
                profile,
                pre_commit: vec!["python -m pytest"],
                pre_push: vec!["python -m pytest"],
                notes: Vec::new(),
            },
            Profile::AgentSkill => {
                let mut notes = Vec::new();
                if agent_skill.evals_dir {
                    notes.push(
                        "evals/ detected. Add project-specific eval validation when a runner is documented.",
                    );
                }
                if agent_skill.smoke_prompts {
                    notes.push(
                        "tests/smoke-prompts.md detected. Review smoke prompts before publishing skill changes.",
                    );
                }

                Self {
                    profile,
                    pre_commit: vec!["\"git diff --check\"", "\"npx skills add . --list\""],
                    pre_push: Vec::new(),
                    notes,
                }
            }
            Profile::Mixed => Self {
                profile,
                pre_commit: Vec::new(),
                pre_push: Vec::new(),
                notes: vec![
                    "Mixed project detected. Add explicit checks after reviewing the stack.",
                ],
            },
            Profile::Custom => Self {
                profile,
                pre_commit: Vec::new(),
                pre_push: Vec::new(),
                notes: vec![
                    "No known project marker detected. Add checks that match this repository.",
                ],
            },
        }
    }

    fn render_yaml(&self) -> String {
        let mut yaml = String::new();

        writeln!(yaml, "version: 1").unwrap();
        writeln!(yaml, "profile: {}", self.profile.name()).unwrap();
        writeln!(yaml).unwrap();
        writeln!(yaml, "commits:").unwrap();
        writeln!(yaml, "  convention: conventional").unwrap();
        writeln!(yaml, "  requireScope: true").unwrap();
        writeln!(yaml).unwrap();
        writeln!(yaml, "checks:").unwrap();
        render_check_list(&mut yaml, "preCommit", &self.pre_commit);
        render_check_list(&mut yaml, "prePush", &self.pre_push);

        for note in &self.notes {
            writeln!(yaml).unwrap();
            writeln!(yaml, "# {note}").unwrap();
        }

        yaml
    }
}

pub fn run(cwd: PathBuf, options: InitOptions) -> Result<(), String> {
    match execute(&cwd, options)? {
        InitOutcome::Preview { profile, yaml } => {
            let mut output_text = String::new();
            writeln!(output_text, "Koba init").unwrap();
            writeln!(output_text).unwrap();
            output_text.push_str(&output::render_rows(&[
                output::row(Status::Ok, "Profile").value(profile.name()),
                output::row(Status::Plan, "Target").value("koba.yml"),
                output::row(Status::Plan, "Mode").value("preview"),
            ]));
            writeln!(output_text).unwrap();
            output::content_block(&mut output_text, "Proposed workflow contract", &yaml);
            writeln!(output_text).unwrap();
            writeln!(output_text, "Next step").unwrap();
            writeln!(
                output_text,
                "{}",
                output::next_step("Run `koba init --apply` to write the file")
            )
            .unwrap();
            print!("{output_text}");
        }
        InitOutcome::Applied { path, .. } => {
            println!("Koba init");
            println!();
            println!(
                "{}",
                output::line(Status::Write, path.display().to_string())
            );
            println!("{}", output::line(Status::Ok, "Workflow contract created"));
        }
        InitOutcome::RefusedExisting { path } => {
            println!("Koba init");
            println!();
            println!(
                "{}",
                output::line(Status::Refuse, format!("{} already exists", path.display()))
            );
            println!(
                "{}",
                output::next_step("Existing files are never overwritten")
            );
        }
    }

    Ok(())
}

pub fn execute(cwd: &Path, options: InitOptions) -> Result<InitOutcome, String> {
    let files = repo::discover(cwd, None);
    let profile = select_profile(&files);
    let yaml = WorkflowContract::from_profile(profile, &files.agent_skill).render_yaml();
    let path = cwd.join("koba.yml");

    if !options.apply {
        return Ok(InitOutcome::Preview { profile, yaml });
    }

    if path.exists() {
        return Ok(InitOutcome::RefusedExisting { path });
    }

    fs::write(&path, &yaml)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    Ok(InitOutcome::Applied {
        profile,
        path,
        yaml,
    })
}

pub fn select_profile(files: &RepoFiles) -> Profile {
    let workflow = &files.workflow;

    if files.agent_skill.detected() {
        return Profile::AgentSkill;
    }

    let markers = [
        workflow.cargo_toml,
        workflow.package_json,
        workflow.pyproject_toml,
    ]
    .into_iter()
    .filter(|present| *present)
    .count();

    match markers {
        0 => Profile::Custom,
        1 if workflow.cargo_toml => Profile::RustCli,
        1 if workflow.package_json => Profile::Node,
        1 if workflow.pyproject_toml => Profile::Python,
        _ => Profile::Mixed,
    }
}

fn render_check_list(yaml: &mut String, key: &str, checks: &[&str]) {
    writeln!(yaml, "  {key}:").unwrap();

    if checks.is_empty() {
        writeln!(yaml, "    []").unwrap();
        return;
    }

    for check in checks {
        writeln!(yaml, "    - {check}").unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::WorkflowFiles;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn preview_does_not_write_koba_yml() {
        let fixture = TempTree::new();
        fixture.file("Cargo.toml");

        let outcome = execute(fixture.path(), InitOptions { apply: false }).unwrap();

        assert!(matches!(outcome, InitOutcome::Preview { .. }));
        assert!(!fixture.path().join("koba.yml").exists());
    }

    #[test]
    fn apply_writes_koba_yml() {
        let fixture = TempTree::new();
        fixture.file("Cargo.toml");

        let outcome = execute(fixture.path(), InitOptions { apply: true }).unwrap();
        let contents = fs::read_to_string(fixture.path().join("koba.yml")).unwrap();

        assert!(matches!(outcome, InitOutcome::Applied { .. }));
        assert!(contents.contains("profile: rust-cli"));
        assert!(contents.contains("cargo fmt --check"));
        assert!(contents.contains("cargo test"));
    }

    #[test]
    fn apply_refuses_to_overwrite_existing_koba_yml() {
        let fixture = TempTree::new();
        fixture.file("Cargo.toml");
        fs::write(fixture.path().join("koba.yml"), "existing: true\n").unwrap();

        let outcome = execute(fixture.path(), InitOptions { apply: true }).unwrap();
        let contents = fs::read_to_string(fixture.path().join("koba.yml")).unwrap();

        assert!(matches!(outcome, InitOutcome::RefusedExisting { .. }));
        assert_eq!(contents, "existing: true\n");
    }

    #[test]
    fn selects_profiles_from_project_markers() {
        assert_eq!(
            select_profile(&repo_files(
                workflow(true, false, false),
                agent_skill(false)
            )),
            Profile::RustCli
        );
        assert_eq!(
            select_profile(&repo_files(
                workflow(false, true, false),
                agent_skill(false)
            )),
            Profile::Node
        );
        assert_eq!(
            select_profile(&repo_files(
                workflow(false, false, true),
                agent_skill(false)
            )),
            Profile::Python
        );
        assert_eq!(
            select_profile(&repo_files(workflow(true, true, false), agent_skill(false))),
            Profile::Mixed
        );
        assert_eq!(
            select_profile(&repo_files(
                workflow(false, false, false),
                agent_skill(false)
            )),
            Profile::Custom
        );
        assert_eq!(
            select_profile(&repo_files(
                workflow(false, false, false),
                agent_skill(true)
            )),
            Profile::AgentSkill
        );
    }

    #[test]
    fn previews_agent_skill_profile() {
        let fixture = TempTree::new();
        fixture.file("skills/hoi4-modding/SKILL.md");
        fixture.dir("evals");
        fixture.file("tests/smoke-prompts.md");

        let outcome = execute(fixture.path(), InitOptions { apply: false }).unwrap();

        let InitOutcome::Preview { profile, yaml } = outcome else {
            panic!("expected preview outcome");
        };

        assert_eq!(profile, Profile::AgentSkill);
        assert!(yaml.contains("profile: agent-skill"));
        assert!(yaml.contains("- \"git diff --check\""));
        assert!(yaml.contains("- \"npx skills add . --list\""));
        assert!(yaml.contains("evals/ detected"));
        assert!(yaml.contains("tests/smoke-prompts.md detected"));
    }

    fn workflow(cargo_toml: bool, package_json: bool, pyproject_toml: bool) -> WorkflowFiles {
        WorkflowFiles {
            koba_yml: false,
            package_json,
            cargo_toml,
            pyproject_toml,
            husky_dir: false,
            native_pre_commit: false,
            native_pre_push: false,
        }
    }

    fn agent_skill(present: bool) -> AgentSkillFiles {
        AgentSkillFiles {
            skills: present
                .then(|| crate::repo::AgentSkill {
                    name: "hoi4-modding".to_owned(),
                    references_dir: false,
                    examples_dir: false,
                })
                .into_iter()
                .collect(),
            evals_dir: false,
            smoke_prompts: false,
            readme: false,
        }
    }

    fn repo_files(workflow: WorkflowFiles, agent_skill: AgentSkillFiles) -> RepoFiles {
        RepoFiles {
            workflow,
            github: crate::repo::GithubFiles {
                github_dir: false,
                workflows_dir: false,
                pull_request_template: false,
                issue_template_dir: false,
                codeowners: false,
                dependabot_yml: false,
            },
            agent_skill,
        }
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
            let path = std::env::temp_dir().join(format!("koba-init-test-{id}"));
            fs::create_dir(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }

        fn file(&self, relative: &str) {
            let path = self.path.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, "").unwrap();
        }

        fn dir(&self, relative: &str) {
            fs::create_dir_all(self.path.join(relative)).unwrap();
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
