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

pub fn status_porcelain(cwd: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=all"])
        .current_dir(cwd)
        .output()
        .map_err(|error| format!("failed to run git status --porcelain: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git status --porcelain failed: {}", stderr.trim()));
    }

    String::from_utf8(output.stdout)
        .map_err(|error| format!("git status --porcelain returned invalid UTF-8: {error}"))
}

pub fn commits_since_base(cwd: &Path) -> Option<(String, Vec<String>)> {
    let base = default_base_branch(cwd)?;
    let range = format!("{base}..HEAD");
    let output = git_output(cwd, &["log", "--format=%s", &range])?;
    let commits = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();

    Some((base, commits))
}

fn default_base_branch(cwd: &Path) -> Option<String> {
    if let Some(symbolic) = git_output(cwd, &["symbolic-ref", "refs/remotes/origin/HEAD"]) {
        if let Some((_, branch)) = symbolic.rsplit_once('/') {
            return Some(format!("origin/{branch}"));
        }
    }

    ["origin/main", "origin/master"]
        .into_iter()
        .find(|branch| git_output(cwd, &["rev-parse", "--verify", branch]).is_some())
        .map(str::to_owned)
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
