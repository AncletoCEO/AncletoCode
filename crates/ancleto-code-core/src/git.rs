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

fn relative_path(path: &str) -> Result<&str> {
    let trimmed = path.trim();
    if trimmed.is_empty()
        || Path::new(trimmed).is_absolute()
        || trimmed.split(['/', '\\']).any(|part| part == "..")
    {
        return Err(GitError::Command("ruta inválida".to_owned()));
    }
    Ok(trimmed)
}

pub fn diff(root: &Path, path: Option<&str>, staged: bool) -> Result<String> {
    let mut args = vec!["diff", "--no-color", "--unified=3"];
    if staged {
        args.push("--cached");
    }
    if let Some(path) = path {
        args.push("--");
        args.push(relative_path(path)?);
    }
    ensure_success(git(root, &args)?)
}

pub fn stage(root: &Path, paths: &[String]) -> Result<()> {
    if paths.is_empty() {
        return Ok(());
    }
    let mut args = vec!["add", "--"];
    for path in paths {
        args.push(relative_path(path)?);
    }
    ensure_success(git(root, &args)?)?;
    Ok(())
}

pub fn unstage(root: &Path, paths: &[String]) -> Result<()> {
    if paths.is_empty() {
        return Ok(());
    }
    let mut args = vec!["restore", "--staged", "--"];
    for path in paths {
        args.push(relative_path(path)?);
    }
    ensure_success(git(root, &args)?)?;
    Ok(())
}

pub fn commit(root: &Path, message: &str) -> Result<()> {
    let message = message.trim();
    if message.is_empty() {
        return Err(GitError::Command(
            "el mensaje de commit está vacío".to_owned(),
        ));
    }
    ensure_success(git(root, &["commit", "-m", message])?)?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitWorktree {
    pub path: String,
    pub branch: Option<String>,
    pub head: Option<String>,
    pub is_main: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitActionOutcome {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

fn action(root: &Path, args: &[&str]) -> Result<GitActionOutcome> {
    let output = git(root, args)?;
    Ok(GitActionOutcome {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    })
}

pub fn fetch(root: &Path) -> Result<GitActionOutcome> {
    action(root, &["fetch", "--all", "--prune"])
}

pub fn pull(root: &Path) -> Result<GitActionOutcome> {
    action(root, &["pull", "--ff-only"])
}

pub fn push(root: &Path) -> Result<GitActionOutcome> {
    action(root, &["push"])
}

pub fn merge(root: &Path, branch: &str) -> Result<GitActionOutcome> {
    validate_branch(root, branch)?;
    action(root, &["merge", branch])
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitRemote {
    pub name: String,
    pub url: String,
    pub provider: String,
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub web_url: Option<String>,
}

fn parse_remote(name: &str, raw: &str) -> GitRemote {
    let url = raw
        .trim()
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .to_owned();
    let normalized = url
        .strip_prefix("ssh://")
        .unwrap_or(&url)
        .strip_prefix("git@")
        .map(|value| value.replacen(':', "/", 1))
        .unwrap_or_else(|| url.clone());
    let without_scheme = normalized
        .strip_prefix("https://")
        .or_else(|| normalized.strip_prefix("http://"))
        .or_else(|| normalized.strip_prefix("git://"))
        .unwrap_or(&normalized);
    let mut parts = without_scheme.split('/');
    let host = parts.next().unwrap_or_default().to_lowercase();
    let segments = parts
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    let owner = segments
        .len()
        .checked_sub(2)
        .and_then(|index| segments.get(index))
        .map(|value| (*value).to_owned());
    let repo = segments.last().map(|value| (*value).to_owned());
    let provider = if host.contains("github") {
        "github"
    } else if host.contains("gitlab") {
        "gitlab"
    } else if host.contains("bitbucket") {
        "bitbucket"
    } else if host.contains("dev.azure.com") || host.contains("visualstudio.com") {
        "azure"
    } else {
        "git"
    };
    let web_url = (provider != "git" && !host.is_empty())
        .then(|| format!("https://{host}/{}", segments.join("/")));
    GitRemote {
        name: name.to_owned(),
        url: raw.trim().to_owned(),
        provider: provider.to_owned(),
        owner,
        repo,
        web_url,
    }
}

pub fn remote(root: &Path) -> Result<Option<GitRemote>> {
    if !is_repo(root)? {
        return Ok(None);
    }
    let names = ensure_success(git(root, &["remote"])?)?;
    let Some(name) = names
        .lines()
        .map(str::trim)
        .find(|name| *name == "origin")
        .or_else(|| names.lines().map(str::trim).find(|name| !name.is_empty()))
    else {
        return Ok(None);
    };
    let url = ensure_success(git(root, &["remote", "get-url", name])?)?;
    if url.is_empty() {
        return Ok(None);
    }
    Ok(Some(parse_remote(name, &url)))
}

pub fn worktrees(root: &Path) -> Result<Vec<GitWorktree>> {
    if !is_repo(root)? {
        return Ok(Vec::new());
    }
    let output = ensure_success(git(root, &["worktree", "list", "--porcelain"])?)?;
    let mut entries = Vec::new();
    let mut current: Option<GitWorktree> = None;
    for line in output.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            current = Some(GitWorktree {
                path: path.to_owned(),
                branch: None,
                head: None,
                is_main: false,
            });
        } else if let Some(head) = line.strip_prefix("HEAD ") {
            if let Some(entry) = current.as_mut() {
                entry.head = Some(head.to_owned());
            }
        } else if let Some(branch) = line.strip_prefix("branch refs/heads/") {
            if let Some(entry) = current.as_mut() {
                entry.branch = Some(branch.to_owned());
            }
        } else if line == "bare" {
            if let Some(entry) = current.as_mut() {
                entry.is_main = true;
            }
        }
    }
    if let Some(entry) = current {
        entries.push(entry);
    }
    if let Some(first) = entries.first_mut() {
        first.is_main = true;
    }
    Ok(entries)
}

fn worktree_path(root: &Path, branch: &str) -> Result<std::path::PathBuf> {
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("repo");
    let parent = root
        .parent()
        .ok_or_else(|| GitError::Command("ruta sin padre".to_owned()))?;
    let safe = branch.replace(['/', '\\'], "-");
    Ok(parent.join(format!("{name}-{safe}")))
}

pub fn add_worktree(root: &Path, branch: &str) -> Result<GitWorktree> {
    validate_branch(root, branch)?;
    let target = worktree_path(root, branch)?;
    if target.exists() {
        return Err(GitError::Command(format!(
            "el worktree ya existe en {}",
            target.display()
        )));
    }
    let target_text = target.to_string_lossy().into_owned();
    ensure_success(git(
        root,
        &["worktree", "add", "-b", branch, &target_text, "HEAD"],
    )?)?;
    worktrees(root)?
        .into_iter()
        .find(|entry| entry.path == target_text)
        .ok_or_else(|| GitError::Command("git no listó el worktree creado".to_owned()))
}

pub fn remove_worktree(root: &Path, path: &str, force: bool) -> Result<()> {
    let mut args = vec!["worktree", "remove"];
    if force {
        args.push("--force");
    }
    args.push(path);
    ensure_success(git(root, &args)?)?;
    let _ = git(root, &["worktree", "prune"]);
    Ok(())
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
    fn stages_and_commits_changes() {
        let dir = init_repo();
        std::fs::write(dir.path().join("b.txt"), "world").expect("write");
        stage(dir.path(), &["b.txt".to_owned()]).expect("stage");
        let before = status(dir.path()).expect("status");
        assert!(before.files[0].staged);
        assert!(diff(dir.path(), Some("b.txt"), true)
            .expect("diff")
            .contains("b.txt"));
        commit(dir.path(), "add b").expect("commit");
        let after = status(dir.path()).expect("status");
        assert!(!after.dirty);
    }

    #[test]
    fn rejects_paths_outside_the_repository() {
        let dir = init_repo();
        assert!(matches!(
            stage(dir.path(), &["../escape".to_owned()]),
            Err(GitError::Command(_))
        ));
    }

    #[test]
    fn creates_lists_and_removes_worktrees() {
        let dir = init_repo();
        let worktree = add_worktree(dir.path(), "wt/test").expect("add worktree");
        assert!(Path::new(&worktree.path).is_dir());
        assert_eq!(worktree.branch.as_deref(), Some("wt/test"));
        let all = worktrees(dir.path()).expect("worktrees");
        assert!(all.iter().any(|entry| entry.path == worktree.path));
        assert!(all.iter().any(|entry| entry.is_main));
        remove_worktree(dir.path(), &worktree.path, true).expect("remove worktree");
        assert!(!Path::new(&worktree.path).exists());
    }

    #[test]
    fn fetch_pull_push_and_merge_report_outcomes() {
        let dir = init_repo();
        let fetch = fetch(dir.path()).expect("fetch");
        assert!(fetch.success);
        let merge = merge(dir.path(), "main").expect("merge");
        assert!(merge.success || !merge.stderr.is_empty());
    }

    #[test]
    fn parses_common_remote_urls() {
        let https = parse_remote("origin", "https://github.com/acme/demo.git");
        assert_eq!(https.provider, "github");
        assert_eq!(https.owner.as_deref(), Some("acme"));
        assert_eq!(https.repo.as_deref(), Some("demo"));
        assert_eq!(
            https.web_url.as_deref(),
            Some("https://github.com/acme/demo")
        );

        let ssh = parse_remote("origin", "git@gitlab.com:acme/demo.git");
        assert_eq!(ssh.provider, "gitlab");
        assert_eq!(ssh.web_url.as_deref(), Some("https://gitlab.com/acme/demo"));
    }

    #[test]
    fn non_repo_is_not_an_error() {
        let dir = tempdir().expect("tempdir");
        let status = status(dir.path()).expect("status");
        assert!(!status.is_repo);
        assert!(branches(dir.path()).expect("branches").is_empty());
    }
}
