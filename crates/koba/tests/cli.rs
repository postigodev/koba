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
    assert!(stdout.contains("[plan]"));
    assert!(stdout.contains("Mode"));
    assert!(stdout.contains("preview"));
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
    assert!(stdout.contains("[refuse]"));
    assert!(stdout.contains("already exists"));
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
    assert!(stdout.contains("Koba run"));
    assert!(stdout.contains("Stage: pre-commit"));
    assert!(stdout.contains("[plan]"));
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
    assert!(stdout.contains("[plan]"));
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
    assert!(stdout.contains("Koba GitHub template"));
    assert!(stdout.contains("[plan]"));
    assert!(stdout.contains("## Summary"));
    assert!(stdout.contains("## Notes for reviewer"));
}

#[test]
fn cli_suggest_commit_reports_clean_git_tree() {
    let fixture = TempTree::new();
    fixture.git_init();

    let output = Command::new(env!("CARGO_BIN_EXE_koba"))
        .arg("suggest-commit")
        .current_dir(fixture.path())
        .output()
        .expect("failed to run koba binary");

    assert!(
        output.status.success(),
        "expected success, got status {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Koba suggest-commit"));
    assert!(stdout.contains("Working tree is clean"));
}

#[test]
fn cli_changes_reports_clean_git_tree() {
    let fixture = TempTree::new();
    fixture.git_init();

    let output = Command::new(env!("CARGO_BIN_EXE_koba"))
        .arg("changes")
        .current_dir(fixture.path())
        .output()
        .expect("failed to run koba binary");

    assert!(
        output.status.success(),
        "expected success, got status {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Koba changes"));
    assert!(stdout.contains("Working tree"));
    assert!(stdout.contains("working tree is clean"));
}

#[test]
fn cli_changes_coherent_tree_renders_one_recommended_commit() {
    let fixture = TempTree::new();
    fixture.git_init();
    fixture.file("crates/koba/src/output.rs");

    let output = Command::new(env!("CARGO_BIN_EXE_koba"))
        .arg("changes")
        .current_dir(fixture.path())
        .output()
        .expect("failed to run koba binary");

    assert!(
        output.status.success(),
        "expected success, got status {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Recommended commit"));
    assert!(!stdout.contains("Commit groups"));
    assert!(stdout.contains("feat(output): improve terminal rendering"));
    assert!(stdout.contains("git add -- \"crates/koba/src/output.rs\""));
}

#[test]
fn cli_changes_mixed_tree_renders_groups_and_split_warning() {
    let fixture = TempTree::new();
    fixture.git_init();
    fixture.file("skills/koba/SKILL.md");
    fixture.file("crates/koba/src/output.rs");

    let output = Command::new(env!("CARGO_BIN_EXE_koba"))
        .arg("changes")
        .current_dir(fixture.path())
        .output()
        .expect("failed to run koba binary");

    assert!(
        output.status.success(),
        "expected success, got status {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Commit groups"));
    assert!(stdout.contains("working tree appears to contain multiple commit concepts"));
    assert!(stdout.contains("feat(output): improve terminal rendering"));
    assert!(stdout.contains("docs(skill): update skill documentation"));
}

#[test]
fn cli_suggest_commit_recommends_commands_without_staging() {
    let fixture = TempTree::new();
    fixture.git_init();
    fixture.file("docs/change.md");

    let output = Command::new(env!("CARGO_BIN_EXE_koba"))
        .arg("suggest-commit")
        .current_dir(fixture.path())
        .output()
        .expect("failed to run koba binary");

    assert!(
        output.status.success(),
        "expected success, got status {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("docs/change.md"));
    assert!(stdout.contains("docs: update documentation"));
    assert!(stdout.contains("git add -- \"docs/change.md\""));
    assert!(stdout.contains("git commit -m \"docs: update documentation\""));

    let status = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=all"])
        .current_dir(fixture.path())
        .output()
        .expect("failed to run git status");
    assert_eq!(
        String::from_utf8_lossy(&status.stdout),
        "?? docs/change.md\n"
    );
}

#[test]
fn cli_suggest_commit_matches_changes_primary_plan() {
    let fixture = TempTree::new();
    fixture.git_init();
    fixture.file("crates/koba/src/git_status.rs");
    fixture.file("crates/koba/src/path_classification.rs");
    fixture.file("crates/koba/src/changes.rs");
    fixture.file("crates/koba/src/suggest_commit.rs");
    fixture.file("crates/koba/src/pr.rs");

    let changes = Command::new(env!("CARGO_BIN_EXE_koba"))
        .arg("changes")
        .current_dir(fixture.path())
        .output()
        .expect("failed to run koba binary");
    let suggest = Command::new(env!("CARGO_BIN_EXE_koba"))
        .arg("suggest-commit")
        .current_dir(fixture.path())
        .output()
        .expect("failed to run koba binary");

    assert!(changes.status.success());
    assert!(suggest.status.success());

    let changes_stdout = String::from_utf8_lossy(&changes.stdout);
    let suggest_stdout = String::from_utf8_lossy(&suggest.stdout);
    let message = "refactor(analysis): centralize status and path classification";

    assert!(changes_stdout.contains(message));
    assert!(suggest_stdout.contains(message));
}

#[test]
fn cli_pr_dry_run_previews_without_writing() {
    let fixture = TempTree::new();
    fixture.git_init();
    fixture.file("docs/change.md");

    let output = Command::new(env!("CARGO_BIN_EXE_koba"))
        .args(["pr", "--dry-run"])
        .current_dir(fixture.path())
        .output()
        .expect("failed to run koba binary");

    assert!(
        output.status.success(),
        "expected success, got status {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!fixture.path().join(".koba/pr-body.md").exists());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Koba PR draft"));
    assert!(stdout.contains("Suggested title"));
    assert!(stdout.contains("Body preview"));
    assert!(stdout.contains("docs/change.md"));
}

#[test]
fn cli_pr_title_matches_changes_primary_plan() {
    let fixture = TempTree::new();
    fixture.git_init();
    fixture.file("crates/koba/src/git_status.rs");
    fixture.file("crates/koba/src/path_classification.rs");
    fixture.file("crates/koba/src/changes.rs");
    fixture.file("crates/koba/src/suggest_commit.rs");
    fixture.file("crates/koba/src/pr.rs");

    let output = Command::new(env!("CARGO_BIN_EXE_koba"))
        .args(["pr", "--dry-run"])
        .current_dir(fixture.path())
        .output()
        .expect("failed to run koba binary");

    assert!(output.status.success());
    assert!(!fixture.path().join(".koba/pr-body.md").exists());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Suggested title"));
    assert!(stdout.contains("refactor(analysis): centralize status and path classification"));
}

#[test]
fn cli_init_rust_agent_skill_repo_includes_both_check_surfaces() {
    let fixture = TempTree::new();
    fixture.file("Cargo.toml");
    fixture.file("skills/koba/SKILL.md");

    let output = Command::new(env!("CARGO_BIN_EXE_koba"))
        .arg("init")
        .current_dir(fixture.path())
        .output()
        .expect("failed to run koba binary");

    assert!(output.status.success());
    assert!(!fixture.path().join("koba.yml").exists());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("profile: rust-cli"));
    assert!(stdout.contains("- cargo fmt --check"));
    assert!(stdout.contains("- cargo check"));
    assert!(stdout.contains("- cargo test"));
    assert!(stdout.contains("- \"npx skills add . --list\""));
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
