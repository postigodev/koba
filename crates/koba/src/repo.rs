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
            koba_yml: is_file(root.join("koba.yml")),
            package_json: is_file(root.join("package.json")),
            cargo_toml: is_file(root.join("Cargo.toml")),
            pyproject_toml: is_file(root.join("pyproject.toml")),
            husky_dir: is_dir(root.join(".husky")),
            native_pre_commit: git_hook_exists(git_dir, "pre-commit"),
            native_pre_push: git_hook_exists(git_dir, "pre-push"),
        },
        github: GithubFiles {
            github_dir: is_dir(root.join(".github")),
            workflows_dir: is_dir(root.join(".github").join("workflows")),
            pull_request_template: is_file(root.join(".github").join("pull_request_template.md")),
            issue_template_dir: is_dir(root.join(".github").join("ISSUE_TEMPLATE")),
            codeowners: is_file(root.join(".github").join("CODEOWNERS")),
            dependabot_yml: is_file(root.join(".github").join("dependabot.yml")),
        },
    }
}

fn git_hook_exists(git_dir: Option<&Path>, hook: &str) -> bool {
    git_dir
        .map(|path| path.join("hooks").join(hook))
        .is_some_and(is_file)
}

fn is_file(path: PathBuf) -> bool {
    path.is_file()
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
    fn detects_all_workflow_and_github_fixture_paths() {
        let fixture = TempTree::new();
        fixture.file("koba.yml");
        fixture.file("Cargo.toml");
        fixture.file("package.json");
        fixture.file("pyproject.toml");
        fixture.dir(".husky");
        fixture.file(".git/hooks/pre-commit");
        fixture.file(".git/hooks/pre-push");
        fixture.dir(".github/workflows");
        fixture.file(".github/pull_request_template.md");
        fixture.dir(".github/ISSUE_TEMPLATE");
        fixture.file(".github/CODEOWNERS");
        fixture.file(".github/dependabot.yml");

        let files = discover(fixture.path(), Some(&fixture.path().join(".git")));

        assert!(files.workflow.koba_yml);
        assert!(files.workflow.cargo_toml);
        assert!(files.workflow.package_json);
        assert!(files.workflow.pyproject_toml);
        assert!(files.workflow.husky_dir);
        assert!(files.workflow.native_pre_commit);
        assert!(files.workflow.native_pre_push);
        assert!(files.github.github_dir);
        assert!(files.github.workflows_dir);
        assert!(files.github.pull_request_template);
        assert!(files.github.issue_template_dir);
        assert!(files.github.codeowners);
        assert!(files.github.dependabot_yml);
    }

    #[test]
    fn missing_fixture_paths_are_reported_as_absent() {
        let fixture = TempTree::new();

        let files = discover(fixture.path(), Some(&fixture.path().join(".git")));

        assert_eq!(
            files.workflow,
            WorkflowFiles {
                koba_yml: false,
                package_json: false,
                cargo_toml: false,
                pyproject_toml: false,
                husky_dir: false,
                native_pre_commit: false,
                native_pre_push: false,
            }
        );
        assert_eq!(
            files.github,
            GithubFiles {
                github_dir: false,
                workflows_dir: false,
                pull_request_template: false,
                issue_template_dir: false,
                codeowners: false,
                dependabot_yml: false,
            }
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
            let path = std::env::temp_dir().join(format!("koba-test-{id}"));
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
