use std::path::Path;
use std::process::{Command, Output};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("git no está instalado o no está en PATH")]
    Missing,
    #[error("no se pudo ejecutar git: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Command(String),
    #[error("nombre de rama inválido")]
    InvalidBranch,
}

pub type Result<T> = std::result::Result<T, GitError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitFile {
    pub path: String,
    pub status: String,
    pub staged: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitStatus {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub dirty: bool,
    pub files: Vec<GitFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitBranch {
    pub name: String,
    pub current: bool,
    pub upstream: Option<String>,
}

fn git(root: &Path, args: &[&str]) -> Result<Output> {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|error| match error.kind() {
            std::io::ErrorKind::NotFound => GitError::Missing,
            _ => GitError::Io(error),
        })
}

fn text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn ensure_success(output: Output) -> Result<String> {
    if output.status.success() {
        Ok(text(&output))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        Err(GitError::Command(if stderr.is_empty() {
            "git terminó con error".to_owned()
        } else {
            stderr
        }))
    }
}

fn is_repo(root: &Path) -> Result<bool> {
    let output = git(root, &["rev-parse", "--is-inside-work-tree"])?;
    Ok(output.status.success() && text(&output) == "true")
}

pub fn status(root: &Path) -> Result<GitStatus> {
    if !is_repo(root)? {
        return Ok(GitStatus {
            is_repo: false,
            branch: None,
            upstream: None,
            ahead: 0,
            behind: 0,
            dirty: false,
            files: Vec::new(),
        });
    }

    let branch = ensure_success(git(root, &["branch", "--show-current"])?)?;
    let branch = (!branch.is_empty()).then_some(branch);

    let upstream = git(
        root,
        &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
    )
    .ok()
    .filter(|output| output.status.success())
    .map(|output| text(&output))
    .filter(|value| !value.is_empty());

    let (ahead, behind) = git(
        root,
        &["rev-list", "--left-right", "--count", "@{u}...HEAD"],
    )
    .ok()
    .filter(|output| output.status.success())
    .map(|output| {
        let value = text(&output);
        let mut parts = value.split_whitespace();
        let behind = parts.next().and_then(|part| part.parse().ok()).unwrap_or(0);
        let ahead = parts.next().and_then(|part| part.parse().ok()).unwrap_or(0);
        (ahead, behind)
    })
    .unwrap_or((0, 0));

    let porcelain = ensure_success(git(root, &["status", "--porcelain=v1"])?)?;
    let files = porcelain
        .lines()
        .filter_map(|line| {
            if line.len() < 4 {
                return None;
            }
            let index = line.as_bytes()[0] as char;
            let worktree = line.as_bytes()[1] as char;
            let path = line.get(3..)?.trim().to_owned();
            let status = if index == '?' {
                "untracked"
            } else if index == 'U' || worktree == 'U' {
                "conflict"
            } else if index == 'R' || worktree == 'R' {
                "renamed"
            } else if index == 'A' {
                "added"
            } else if index == 'D' || worktree == 'D' {
                "deleted"
            } else {
                "modified"
            };
            Some(GitFile {
                path,
                status: status.to_owned(),
                staged: index != ' ' && index != '?',
            })
        })
        .collect::<Vec<_>>();

    Ok(GitStatus {
        is_repo: true,
        branch,
        upstream,
        ahead,
        behind,
        dirty: !files.is_empty(),
        files,
    })
}

pub fn branches(root: &Path) -> Result<Vec<GitBranch>> {
    if !is_repo(root)? {
        return Ok(Vec::new());
    }
    let output = ensure_success(git(
        root,
        &[
            "for-each-ref",
            "--format=%(refname:short)%00%(HEAD)%00%(upstream:short)",
            "refs/heads",
        ],
    )?)?;
    let mut branches = output
        .lines()
        .filter_map(|line| {
            let mut fields = line.split('\0');
            let name = fields.next()?.to_owned();
            if name.is_empty() {
                return None;
            }
            let current = fields.next() == Some("*");
            let upstream = fields
                .next()
                .filter(|value| !value.is_empty())
                .map(str::to_owned);
            Some(GitBranch {
                name,
                current,
                upstream,
            })
        })
        .collect::<Vec<_>>();
    branches.sort_by(|left, right| {
        right
            .current
            .cmp(&left.current)
            .then_with(|| left.name.cmp(&right.name))
    });
    Ok(branches)
}

fn validate_branch(root: &Path, name: &str) -> Result<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.starts_with('-') || trimmed.contains("..") {
        return Err(GitError::InvalidBranch);
    }
    let output = git(root, &["check-ref-format", "--branch", trimmed])?;
    if output.status.success() {
        Ok(())
    } else {
        Err(GitError::InvalidBranch)
    }
}

pub fn create_branch(root: &Path, name: &str, checkout: bool) -> Result<()> {
    validate_branch(root, name)?;
    let args = if checkout {
        vec!["switch", "-c", name]
    } else {
        vec!["branch", name]
    };
    ensure_success(git(root, &args)?)?;
    Ok(())
}

pub fn checkout(root: &Path, name: &str) -> Result<()> {
    validate_branch(root, name)?;
    ensure_success(git(root, &["switch", name])?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use tempfile::tempdir;

    use super::*;

    fn init_repo() -> tempfile::TempDir {
        let dir = tempdir().expect("tempdir");
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(dir.path())
            .status()
            .expect("git init");
        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(dir.path())
            .status()
            .expect("git config email");
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir.path())
            .status()
            .expect("git config name");
        Command::new("git")
            .args(["commit", "--allow-empty", "-q", "-m", "init"])
            .current_dir(dir.path())
            .status()
            .expect("git commit");
        dir
    }

    #[test]
    fn status_reports_branch_and_changes() {
        let dir = init_repo();
        std::fs::write(dir.path().join("a.txt"), "hello").expect("write");
        let status = status(dir.path()).expect("status");
        assert!(status.is_repo);
        assert!(status.branch.is_some());
        assert!(status.dirty);
        assert_eq!(status.files[0].path, "a.txt");
        assert_eq!(status.files[0].status, "untracked");
    }

    #[test]
    fn creates_and_switches_branch() {
        let dir = init_repo();
        create_branch(dir.path(), "feature/ui", true).expect("create");
        assert_eq!(
            status(dir.path()).expect("status").branch.as_deref(),
            Some("feature/ui")
        );
        let names = branches(dir.path()).expect("branches");
        assert!(names
            .iter()
            .any(|branch| branch.name == "feature/ui" && branch.current));
    }

    #[test]
    fn rejects_invalid_branch() {
        let dir = init_repo();
        assert!(matches!(
            create_branch(dir.path(), "-bad", true),
            Err(GitError::InvalidBranch)
        ));
        assert!(matches!(
            create_branch(dir.path(), "a..b", true),
            Err(GitError::InvalidBranch)
        ));
    }

    #[test]
    fn non_repo_is_not_an_error() {
        let dir = tempdir().expect("tempdir");
        let status = status(dir.path()).expect("status");
        assert!(!status.is_repo);
        assert!(branches(dir.path()).expect("branches").is_empty());
    }
}
