use std::{
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    output::{self, Status},
    repo::{self, WorkflowFiles},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitOptions {
    pub apply: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitOutcome {
    Preview { yaml: String },
    Applied { path: PathBuf, yaml: String },
    RefusedExisting { path: PathBuf },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    RustCli,
    Node,
    Python,
    Mixed,
    Custom,
}

impl Profile {
    fn name(&self) -> &'static str {
        match self {
            Profile::RustCli => "rust-cli",
            Profile::Node => "node",
            Profile::Python => "python",
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
    note: Option<&'static str>,
}

impl WorkflowContract {
    fn from_profile(profile: Profile) -> Self {
        match profile {
            Profile::RustCli => Self {
                profile,
                pre_commit: vec!["cargo fmt --check"],
                pre_push: vec!["cargo test"],
                note: None,
            },
            Profile::Node => Self {
                profile,
                pre_commit: vec!["npm test"],
                pre_push: vec!["npm run build"],
                note: None,
            },
            Profile::Python => Self {
                profile,
                pre_commit: vec!["python -m pytest"],
                pre_push: vec!["python -m pytest"],
                note: None,
            },
            Profile::Mixed => Self {
                profile,
                pre_commit: Vec::new(),
                pre_push: Vec::new(),
                note: Some(
                    "Mixed project detected. Add explicit checks after reviewing the stack.",
                ),
            },
            Profile::Custom => Self {
                profile,
                pre_commit: Vec::new(),
                pre_push: Vec::new(),
                note: Some(
                    "No known project marker detected. Add checks that match this repository.",
                ),
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

        if let Some(note) = self.note {
            writeln!(yaml).unwrap();
            writeln!(yaml, "# {note}").unwrap();
        }

        yaml
    }
}

pub fn run(cwd: PathBuf, options: InitOptions) -> Result<(), String> {
    match execute(&cwd, options)? {
        InitOutcome::Preview { yaml } => {
            println!("Koba init");
            println!();
            println!(
                "{}",
                output::line(Status::Step, "Preview only; no files were written")
            );
            println!();
            print!("{yaml}");
        }
        InitOutcome::Applied { path, .. } => {
            println!("Koba init");
            println!();
            println!(
                "{}",
                output::line(Status::Ok, format!("Wrote {}", path.display()))
            );
        }
        InitOutcome::RefusedExisting { path } => {
            println!("Koba init");
            println!();
            println!(
                "{}",
                output::line(
                    Status::Warning,
                    format!("{} already exists; refusing to overwrite", path.display())
                )
            );
            println!(
                "{}",
                output::line(
                    Status::Step,
                    "Review the existing koba.yml before changing it"
                )
            );
        }
    }

    Ok(())
}

pub fn execute(cwd: &Path, options: InitOptions) -> Result<InitOutcome, String> {
    let workflow = repo::discover(cwd, None).workflow;
    let yaml = WorkflowContract::from_profile(select_profile(&workflow)).render_yaml();
    let path = cwd.join("koba.yml");

    if !options.apply {
        return Ok(InitOutcome::Preview { yaml });
    }

    if path.exists() {
        return Ok(InitOutcome::RefusedExisting { path });
    }

    fs::write(&path, &yaml)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    Ok(InitOutcome::Applied { path, yaml })
}

pub fn select_profile(workflow: &WorkflowFiles) -> Profile {
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
            select_profile(&workflow(true, false, false)),
            Profile::RustCli
        );
        assert_eq!(select_profile(&workflow(false, true, false)), Profile::Node);
        assert_eq!(
            select_profile(&workflow(false, false, true)),
            Profile::Python
        );
        assert_eq!(select_profile(&workflow(true, true, false)), Profile::Mixed);
        assert_eq!(
            select_profile(&workflow(false, false, false)),
            Profile::Custom
        );
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
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
