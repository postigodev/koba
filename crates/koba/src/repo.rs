use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowFiles {
    pub koba_yml: bool,
    pub package_json: bool,
    pub cargo_toml: bool,
    pub pyproject_toml: bool,
    pub husky_dir: bool,
    pub native_pre_commit: bool,
    pub native_pre_push: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubFiles {
    pub github_dir: bool,
    pub workflows_dir: bool,
    pub pull_request_template: bool,
    pub issue_template_dir: bool,
    pub codeowners: bool,
    pub dependabot_yml: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoFiles {
    pub workflow: WorkflowFiles,
    pub github: GithubFiles,
}

pub fn discover(root: &Path, git_dir: Option<&Path>) -> RepoFiles {
    RepoFiles {
        workflow: WorkflowFiles {
            koba_yml: exists(root.join("koba.yml")),
            package_json: exists(root.join("package.json")),
            cargo_toml: exists(root.join("Cargo.toml")),
            pyproject_toml: exists(root.join("pyproject.toml")),
            husky_dir: is_dir(root.join(".husky")),
            native_pre_commit: git_hook_exists(git_dir, "pre-commit"),
            native_pre_push: git_hook_exists(git_dir, "pre-push"),
        },
        github: GithubFiles {
            github_dir: is_dir(root.join(".github")),
            workflows_dir: is_dir(root.join(".github").join("workflows")),
            pull_request_template: exists(root.join(".github").join("pull_request_template.md")),
            issue_template_dir: is_dir(root.join(".github").join("ISSUE_TEMPLATE")),
            codeowners: exists(root.join(".github").join("CODEOWNERS")),
            dependabot_yml: exists(root.join(".github").join("dependabot.yml")),
        },
    }
}

fn git_hook_exists(git_dir: Option<&Path>, hook: &str) -> bool {
    git_dir
        .map(|path| path.join("hooks").join(hook))
        .is_some_and(exists)
}

fn exists(path: PathBuf) -> bool {
    path.exists()
}

fn is_dir(path: PathBuf) -> bool {
    path.is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn detects_cargo_toml_and_github_pull_request_template() {
        let fixture = TempTree::new();
        fs::write(fixture.path().join("Cargo.toml"), "").unwrap();
        fs::create_dir(fixture.path().join(".github")).unwrap();
        fs::write(
            fixture
                .path()
                .join(".github")
                .join("pull_request_template.md"),
            "",
        )
        .unwrap();

        let files = discover(fixture.path(), None);

        assert!(files.workflow.cargo_toml);
        assert!(files.github.github_dir);
        assert!(files.github.pull_request_template);
        assert!(!files.workflow.koba_yml);
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
            let path = std::env::temp_dir().join(format!("koba-test-{id}"));
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
