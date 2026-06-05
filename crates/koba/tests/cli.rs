use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn cli_can_run_scan() {
    let output = Command::new(env!("CARGO_BIN_EXE_koba"))
        .arg("scan")
        .output()
        .expect("failed to run koba binary");

    assert!(
        output.status.success(),
        "expected success, got status {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Koba scan"));
    assert!(stdout.contains("Repository"));
    assert!(stdout.contains("Workflow"));
    assert!(stdout.contains("GitHub"));
}

#[test]
fn cli_can_run_doctor() {
    let output = Command::new(env!("CARGO_BIN_EXE_koba"))
        .arg("doctor")
        .output()
        .expect("failed to run koba binary");

    assert!(
        output.status.success(),
        "expected success, got status {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Koba doctor"));
    assert!(stdout.contains("Repository"));
}

#[test]
fn cli_init_preview_does_not_write_koba_yml() {
    let fixture = TempTree::new();
    fixture.file("Cargo.toml");

    let output = Command::new(env!("CARGO_BIN_EXE_koba"))
        .arg("init")
        .current_dir(fixture.path())
        .output()
        .expect("failed to run koba binary");

    assert!(output.status.success());
    assert!(!fixture.path().join("koba.yml").exists());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("profile: rust-cli"));
    assert!(stdout.contains("Preview only"));
}

#[test]
fn cli_init_apply_writes_koba_yml() {
    let fixture = TempTree::new();
    fixture.file("Cargo.toml");

    let output = Command::new(env!("CARGO_BIN_EXE_koba"))
        .args(["init", "--apply"])
        .current_dir(fixture.path())
        .output()
        .expect("failed to run koba binary");

    assert!(output.status.success());

    let contents = fs::read_to_string(fixture.path().join("koba.yml")).unwrap();
    assert!(contents.contains("profile: rust-cli"));
}

#[test]
fn cli_init_apply_refuses_to_overwrite_koba_yml() {
    let fixture = TempTree::new();
    fixture.file("Cargo.toml");
    fs::write(fixture.path().join("koba.yml"), "existing: true\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_koba"))
        .args(["init", "--apply"])
        .current_dir(fixture.path())
        .output()
        .expect("failed to run koba binary");

    assert!(output.status.success());

    let contents = fs::read_to_string(fixture.path().join("koba.yml")).unwrap();
    assert_eq!(contents, "existing: true\n");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("refusing to overwrite"));
}

#[test]
fn cli_run_pre_commit_dry_run_lists_checks_without_executing() {
    let fixture = TempTree::new();
    let marker = fixture.path().join("marker.txt");
    fs::write(
        fixture.path().join("koba.yml"),
        r#"
checks:
  preCommit:
    - echo changed > marker.txt
  prePush: []
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_koba"))
        .args(["run", "pre-commit", "--dry-run"])
        .current_dir(fixture.path())
        .output()
        .expect("failed to run koba binary");

    assert!(
        output.status.success(),
        "expected success, got status {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!marker.exists());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Koba run pre-commit"));
    assert!(stdout.contains("Dry run"));
    assert!(stdout.contains("echo changed"));
}

#[test]
fn cli_hooks_install_native_dry_run_previews_without_writing() {
    let fixture = TempTree::new();
    let output = Command::new("git")
        .arg("init")
        .current_dir(fixture.path())
        .output()
        .expect("failed to run git init");
    assert!(
        output.status.success(),
        "git init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = Command::new(env!("CARGO_BIN_EXE_koba"))
        .args(["hooks", "install", "--adapter", "native", "--dry-run"])
        .current_dir(fixture.path())
        .output()
        .expect("failed to run koba binary");

    assert!(
        output.status.success(),
        "expected success, got status {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!fixture.path().join(".git/hooks/pre-commit").exists());
    assert!(!fixture.path().join(".git/hooks/pre-push").exists());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Koba hooks install"));
    assert!(stdout.contains("Would write"));
    assert!(stdout.contains("koba run pre-commit"));
}

#[test]
fn cli_github_template_pr_dry_run_previews_without_writing() {
    let fixture = TempTree::new();

    let output = Command::new(env!("CARGO_BIN_EXE_koba"))
        .args(["github", "template", "pr", "--dry-run"])
        .current_dir(fixture.path())
        .output()
        .expect("failed to run koba binary");

    assert!(
        output.status.success(),
        "expected success, got status {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!fixture
        .path()
        .join(".github/pull_request_template.md")
        .exists());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Koba github template pr"));
    assert!(stdout.contains("Would write"));
    assert!(stdout.contains("## Summary"));
    assert!(stdout.contains("## Notes for reviewer"));
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
        let path = std::env::temp_dir().join(format!("koba-cli-test-{id}"));
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
