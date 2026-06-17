use std::{path::Path, process::Command};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitStatusEntry {
    pub path: String,
    pub old_path: Option<String>,
    pub index_status: char,
    pub worktree_status: char,
    pub staged: bool,
    pub unstaged: bool,
    pub untracked: bool,
    pub deleted: bool,
    pub renamed: bool,
}

impl GitStatusEntry {
    #[cfg(test)]
    pub fn from_status(status: &str, path: impl Into<String>) -> Self {
        let (index_status, worktree_status) = status_chars(status);
        Self::new(path.into(), None, index_status, worktree_status)
    }

    pub fn short_status(&self) -> String {
        if self.untracked {
            "??".to_owned()
        } else {
            format!("{}{}", self.index_status, self.worktree_status)
                .trim()
                .to_owned()
        }
    }

    fn new(
        path: String,
        old_path: Option<String>,
        index_status: char,
        worktree_status: char,
    ) -> Self {
        let untracked = index_status == '?' && worktree_status == '?';
        let staged = !untracked && index_status != ' ';
        let unstaged = !untracked && worktree_status != ' ';
        let deleted = index_status == 'D' || worktree_status == 'D';
        let renamed = index_status == 'R' || worktree_status == 'R';

        Self {
            path,
            old_path,
            index_status,
            worktree_status,
            staged,
            unstaged,
            untracked,
            deleted,
            renamed,
        }
    }
}

pub fn status_entries(cwd: &Path) -> Result<Vec<GitStatusEntry>, String> {
    let output = Command::new("git")
        .args(["status", "--porcelain=v1", "-z", "--untracked-files=all"])
        .current_dir(cwd)
        .output()
        .map_err(|error| format!("failed to run git status --porcelain=v1 -z: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "git status --porcelain=v1 -z failed: {}",
            stderr.trim()
        ));
    }

    parse_porcelain_z(&output.stdout)
}

pub fn parse_porcelain_z(output: &[u8]) -> Result<Vec<GitStatusEntry>, String> {
    let records = output
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
        .collect::<Vec<_>>();
    let mut entries = Vec::new();
    let mut index = 0;

    while index < records.len() {
        let record = records[index];
        if record.len() < 4 {
            return Err("git status returned a malformed porcelain record".to_owned());
        }

        let index_status = record[0] as char;
        let worktree_status = record[1] as char;
        if record[2] != b' ' {
            return Err("git status returned an unexpected porcelain separator".to_owned());
        }

        let renamed = index_status == 'R'
            || index_status == 'C'
            || worktree_status == 'R'
            || worktree_status == 'C';
        let path = bytes_to_string(&record[3..])?;
        let old_path = if renamed {
            index += 1;
            let old_record = records.get(index).ok_or_else(|| {
                "git status returned a rename record without an old path".to_owned()
            })?;
            Some(bytes_to_string(old_record)?)
        } else {
            None
        };

        entries.push(GitStatusEntry::new(
            path,
            old_path,
            index_status,
            worktree_status,
        ));
        index += 1;
    }

    Ok(entries)
}

#[cfg(test)]
fn status_chars(status: &str) -> (char, char) {
    if status == "??" {
        return ('?', '?');
    }

    let mut chars = status.chars();
    let first = chars.next().unwrap_or(' ');
    let second = chars.next().unwrap_or(' ');

    match (first, second) {
        (' ', worktree) => (' ', worktree),
        (index, ' ') if status.len() == 1 => (index, ' '),
        pair => pair,
    }
}

fn bytes_to_string(bytes: &[u8]) -> Result<String, String> {
    String::from_utf8(bytes.to_vec())
        .map_err(|error| format!("git status returned invalid UTF-8: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_paths_with_spaces() {
        let entries = parse_porcelain_z(b" M docs/my file.md\0").unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "docs/my file.md");
        assert!(entries[0].unstaged);
        assert_eq!(entries[0].short_status(), "M");
    }

    #[test]
    fn parses_deleted_files() {
        let entries = parse_porcelain_z(b" D old file.md\0D  staged-old.rs\0").unwrap();

        assert_eq!(entries.len(), 2);
        assert!(entries[0].deleted);
        assert!(entries[0].unstaged);
        assert!(entries[1].deleted);
        assert!(entries[1].staged);
    }

    #[test]
    fn parses_renamed_files_with_old_path() {
        let entries = parse_porcelain_z(b"R  new name.md\0old name.md\0").unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "new name.md");
        assert_eq!(entries[0].old_path.as_deref(), Some("old name.md"));
        assert!(entries[0].renamed);
        assert!(entries[0].staged);
    }

    #[test]
    fn parses_untracked_files() {
        let entries = parse_porcelain_z(b"?? new.md\0").unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "new.md");
        assert!(entries[0].untracked);
        assert!(!entries[0].staged);
        assert!(!entries[0].unstaged);
    }

    #[test]
    fn parses_staged_and_unstaged_status() {
        let entries = parse_porcelain_z(b"MM crates/koba/src/lib.rs\0").unwrap();

        assert_eq!(entries.len(), 1);
        assert!(entries[0].staged);
        assert!(entries[0].unstaged);
        assert_eq!(entries[0].short_status(), "MM");
    }
}
