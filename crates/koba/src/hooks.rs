use std::{
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

use clap::{Subcommand, ValueEnum};

use crate::{
    git,
    output::{self, Status},
};

#[derive(Debug, Clone, Subcommand)]
pub enum HooksCommand {
    /// Preview or install hook files that call koba run.
    Install {
        /// Hook adapter to install.
        #[arg(long)]
        adapter: HookAdapter,
        /// Preview hook files without writing them.
        #[arg(long)]
        dry_run: bool,
        /// Write missing hook files.
        #[arg(long)]
        apply: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum HookAdapter {
    Native,
    Husky,
}

impl HookAdapter {
    fn label(&self) -> &'static str {
        match self {
            HookAdapter::Native => "native",
            HookAdapter::Husky => "husky",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstallOptions {
    pub adapter: HookAdapter,
    pub dry_run: bool,
    pub apply: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookFilePlan {
    pub path: PathBuf,
    pub contents: String,
    pub exists: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookInstallPlan {
    pub adapter: HookAdapter,
    pub files: Vec<HookFilePlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallOutcome {
    Preview(HookInstallPlan),
    Applied(HookInstallPlan),
}

pub fn run_install(cwd: PathBuf, options: InstallOptions) -> Result<(), String> {
    match execute_install(&cwd, options) {
        Ok(outcome) => {
            print!("{}", render_outcome(&outcome));
            Ok(())
        }
        Err(error) => {
            println!("Koba hooks install");
            println!();
            println!("{}", output::line(Status::Error, &error));
            Err(error)
        }
    }
}

pub fn execute_install(cwd: &Path, options: InstallOptions) -> Result<InstallOutcome, String> {
    if options.dry_run && options.apply {
        return Err("choose either --dry-run or --apply, not both".to_owned());
    }

    let plan = build_plan(cwd, options.adapter)?;

    if !options.apply {
        return Ok(InstallOutcome::Preview(plan));
    }

    apply_plan(&plan)?;
    Ok(InstallOutcome::Applied(plan))
}

pub fn build_plan(cwd: &Path, adapter: HookAdapter) -> Result<HookInstallPlan, String> {
    match adapter {
        HookAdapter::Native => native_plan(cwd),
        HookAdapter::Husky => Ok(husky_plan(cwd)),
    }
}

fn native_plan(cwd: &Path) -> Result<HookInstallPlan, String> {
    let git = git::inspect(cwd);

    if !git.inside_repo {
        return Err("native hook installation requires a Git repository".to_owned());
    }

    let git_dir = git
        .git_dir
        .ok_or_else(|| "native hook installation requires a .git directory".to_owned())?;
    let hooks_dir = git_dir.join("hooks");

    if !hooks_dir.is_dir() {
        return Err(format!(
            "native hook installation requires {}",
            hooks_dir.display()
        ));
    }

    Ok(HookInstallPlan {
        adapter: HookAdapter::Native,
        files: hook_files(&hooks_dir),
    })
}

fn husky_plan(cwd: &Path) -> HookInstallPlan {
    HookInstallPlan {
        adapter: HookAdapter::Husky,
        files: hook_files(&cwd.join(".husky")),
    }
}

fn hook_files(dir: &Path) -> Vec<HookFilePlan> {
    [("pre-commit", "pre-commit"), ("pre-push", "pre-push")]
        .into_iter()
        .map(|(file_name, stage)| {
            let path = dir.join(file_name);
            HookFilePlan {
                exists: path.exists(),
                path,
                contents: hook_contents(stage),
            }
        })
        .collect()
}

fn hook_contents(stage: &str) -> String {
    format!("#!/bin/sh\nkoba run {stage}\n")
}

fn apply_plan(plan: &HookInstallPlan) -> Result<(), String> {
    for file in &plan.files {
        if file.exists || file.path.exists() {
            continue;
        }

        if let Some(parent) = file.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }

        fs::write(&file.path, &file.contents)
            .map_err(|error| format!("failed to write {}: {error}", file.path.display()))?;
        make_executable(&file.path)?;
    }

    Ok(())
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .map_err(|error| format!("failed to read permissions for {}: {error}", path.display()))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
        .map_err(|error| format!("failed to set permissions for {}: {error}", path.display()))
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn render_outcome(outcome: &InstallOutcome) -> String {
    let mut output = String::new();
    let (plan, applied) = match outcome {
        InstallOutcome::Preview(plan) => (plan, false),
        InstallOutcome::Applied(plan) => (plan, true),
    };

    writeln!(output, "Koba hooks install").unwrap();
    writeln!(output).unwrap();
    writeln!(output, "Adapter: {}", plan.adapter.label()).unwrap();
    writeln!(
        output,
        "Mode: {}",
        if applied { "apply" } else { "preview" }
    )
    .unwrap();

    writeln!(output).unwrap();
    writeln!(output, "Files").unwrap();

    for file in &plan.files {
        let row = if file.exists {
            let status = if applied {
                Status::Keep
            } else {
                Status::Refuse
            };
            output::row(status, format!("{} already exists", file.path.display()))
        } else if applied {
            output::row(Status::Write, file.path.display().to_string())
        } else {
            output::row(Status::Plan, file.path.display().to_string())
        }
        .detail(file.contents.trim_end());

        output.push_str(&output::render_rows(&[row]));

        if file.exists {
            writeln!(
                output,
                "{}",
                output::next_step("Existing files are never overwritten")
            )
            .unwrap();
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::{Path, PathBuf},
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn native_dry_run_does_not_write_files() {
        let fixture = GitFixture::new();

        let outcome = execute_install(
            fixture.path(),
            InstallOptions {
                adapter: HookAdapter::Native,
                dry_run: true,
                apply: false,
            },
        )
        .unwrap();

        assert!(matches!(outcome, InstallOutcome::Preview(_)));
        assert!(!fixture.path().join(".git/hooks/pre-commit").exists());
        assert!(!fixture.path().join(".git/hooks/pre-push").exists());
    }

    #[test]
    fn native_apply_writes_both_hooks() {
        let fixture = GitFixture::new();

        execute_install(
            fixture.path(),
            InstallOptions {
                adapter: HookAdapter::Native,
                dry_run: false,
                apply: true,
            },
        )
        .unwrap();

        assert_eq!(
            fs::read_to_string(fixture.path().join(".git/hooks/pre-commit")).unwrap(),
            "#!/bin/sh\nkoba run pre-commit\n"
        );
        assert_eq!(
            fs::read_to_string(fixture.path().join(".git/hooks/pre-push")).unwrap(),
            "#!/bin/sh\nkoba run pre-push\n"
        );
    }

    #[test]
    fn apply_does_not_overwrite_existing_hooks() {
        let fixture = GitFixture::new();
        fs::write(fixture.path().join(".git/hooks/pre-commit"), "existing\n").unwrap();

        execute_install(
            fixture.path(),
            InstallOptions {
                adapter: HookAdapter::Native,
                dry_run: false,
                apply: true,
            },
        )
        .unwrap();

        assert_eq!(
            fs::read_to_string(fixture.path().join(".git/hooks/pre-commit")).unwrap(),
            "existing\n"
        );
        assert_eq!(
            fs::read_to_string(fixture.path().join(".git/hooks/pre-push")).unwrap(),
            "#!/bin/sh\nkoba run pre-push\n"
        );
    }

    #[test]
    fn husky_apply_creates_husky_hooks() {
        let fixture = TempTree::new();

        execute_install(
            fixture.path(),
            InstallOptions {
                adapter: HookAdapter::Husky,
                dry_run: false,
                apply: true,
            },
        )
        .unwrap();

        assert_eq!(
            fs::read_to_string(fixture.path().join(".husky/pre-commit")).unwrap(),
            "#!/bin/sh\nkoba run pre-commit\n"
        );
        assert_eq!(
            fs::read_to_string(fixture.path().join(".husky/pre-push")).unwrap(),
            "#!/bin/sh\nkoba run pre-push\n"
        );
    }

    struct GitFixture {
        tree: TempTree,
    }

    impl GitFixture {
        fn new() -> Self {
            let tree = TempTree::new();
            let output = Command::new("git")
                .arg("init")
                .current_dir(tree.path())
                .output()
                .expect("failed to run git init");
            assert!(
                output.status.success(),
                "git init failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            Self { tree }
        }

        fn path(&self) -> &Path {
            self.tree.path()
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
            let path = std::env::temp_dir().join(format!("koba-hooks-test-{id}"));
            fs::create_dir(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
