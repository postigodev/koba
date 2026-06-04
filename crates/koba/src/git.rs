use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitInfo {
    pub inside_repo: bool,
    pub root: Option<PathBuf>,
    pub git_dir: Option<PathBuf>,
    pub branch: Option<String>,
    pub has_origin: bool,
    pub has_user_name: bool,
    pub has_user_email: bool,
}

pub fn inspect(cwd: &Path) -> GitInfo {
    let root = git_output(cwd, &["rev-parse", "--show-toplevel"]).map(PathBuf::from);
    let git_dir =
        git_output(cwd, &["rev-parse", "--git-dir"]).map(|path| normalize_git_dir(cwd, path));
    let inside_repo = root.is_some();

    GitInfo {
        inside_repo,
        root,
        git_dir,
        branch: inside_repo
            .then(|| git_output(cwd, &["branch", "--show-current"]))
            .flatten()
            .filter(|branch| !branch.is_empty()),
        has_origin: inside_repo && git_output(cwd, &["remote", "get-url", "origin"]).is_some(),
        has_user_name: inside_repo && git_output(cwd, &["config", "--get", "user.name"]).is_some(),
        has_user_email: inside_repo
            && git_output(cwd, &["config", "--get", "user.email"]).is_some(),
    }
}

fn git_output(cwd: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8(output.stdout).ok()?;
    Some(text.trim().to_owned()).filter(|text| !text.is_empty())
}

fn normalize_git_dir(cwd: &Path, path: String) -> PathBuf {
    let git_dir = PathBuf::from(path);

    if git_dir.is_absolute() {
        git_dir
    } else {
        cwd.join(git_dir)
    }
}
