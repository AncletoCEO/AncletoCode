use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use uuid::Uuid;

use crate::domain::{CoreError, FileNode, Project, Result};
use crate::persistence::Store;
use crate::tasks::{Task, TaskStatus};

#[allow(clippy::unwrap_used)]
#[path = "../../../../third_party/pandacode/src-tauri/src/workspace/file_search.rs"]
mod file_search;

fn resolve_scoped_path(root: &Path, relative: &str) -> std::result::Result<PathBuf, ()> {
    let candidate = root.join(relative).canonicalize().map_err(|_| ())?;
    if candidate.starts_with(root) {
        Ok(candidate)
    } else {
        Err(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

const MAX_ENTRIES: usize = 5_000;
const MAX_DEPTH: usize = 6;
const SKIPPED_DIRECTORIES: [&str; 4] = [".git", ".ancleto", "node_modules", "target"];

pub struct ProjectService {
    store: Store,
}

impl ProjectService {
    pub fn open(database_path: &Path) -> Result<Self> {
        Ok(Self {
            store: Store::open(database_path)?,
        })
    }

    pub fn add(&self, path: &str) -> Result<Project> {
        let canonical =
            fs::canonicalize(path).map_err(|_| CoreError::InvalidPath(path.to_owned()))?;

        if !canonical.is_dir() {
            return Err(CoreError::NotADirectory(
                canonical.to_string_lossy().into_owned(),
            ));
        }

        let root_path = canonical.to_string_lossy().into_owned();

        if let Some(existing) = self.store.find_project_by_root(&root_path)? {
            return self
                .store
                .touch_project(&existing.id, unix_timestamp())?
                .ok_or(CoreError::ProjectNotFound(existing.id));
        }

        let name = canonical
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| root_path.clone());
        let now = unix_timestamp();
        let project = Project {
            id: Uuid::new_v4().to_string(),
            name,
            root_path,
            created_at: now,
            last_opened_at: now,
        };

        self.store.insert_project(&project)?;
        Ok(project)
    }

    pub fn list(&self) -> Result<Vec<Project>> {
        self.store.list_projects()
    }

    pub fn remove(&self, id: &str) -> Result<bool> {
        self.store.remove_project(id)
    }

    pub fn search(&self, id: &str, query: &str, limit: Option<usize>) -> Result<Vec<FileEntry>> {
        let project = self.get(id)?;
        file_search::search_files(Path::new(&project.root_path), query.to_owned(), limit)
            .map_err(CoreError::Process)
    }

    pub fn get(&self, id: &str) -> Result<Project> {
        self.store
            .find_project_by_id(id)?
            .ok_or_else(|| CoreError::ProjectNotFound(id.to_owned()))
    }

    pub fn open_project(&self, id: &str) -> Result<Project> {
        self.store
            .touch_project(id, unix_timestamp())?
            .ok_or_else(|| CoreError::ProjectNotFound(id.to_owned()))
    }

    pub fn tasks(&self, project_id: &str) -> Result<Vec<Task>> {
        self.store.list_tasks(project_id)
    }

    pub fn create_task(&self, project_id: &str, title: &str) -> Result<Task> {
        self.get(project_id)?;
        let now = unix_timestamp();
        let task = Task {
            id: Uuid::new_v4().to_string(),
            project_id: project_id.to_owned(),
            title: title.trim().to_owned(),
            status: TaskStatus::Backlog,
            branch: None,
            worktree_path: None,
            session_id: None,
            pr_url: None,
            created_at: now,
            updated_at: now,
        };
        self.store.insert_task(&task)?;
        Ok(task)
    }

    pub fn update_task(&self, task: &Task) -> Result<Task> {
        let mut next = task.clone();
        next.updated_at = unix_timestamp();
        self.store.update_task(&next)?;
        Ok(next)
    }

    pub fn remove_task(&self, id: &str) -> Result<bool> {
        self.store.delete_task(id)
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        self.store.get_setting(key)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.store.set_setting(key, value)
    }

    pub fn tree(&self, id: &str, limit: Option<usize>) -> Result<FileNode> {
        let project = self
            .store
            .find_project_by_id(id)?
            .ok_or_else(|| CoreError::ProjectNotFound(id.to_owned()))?;
        let mut budget = limit.unwrap_or(MAX_ENTRIES).min(MAX_ENTRIES);
        let children = read_children(Path::new(&project.root_path), MAX_DEPTH, &mut budget);

        Ok(FileNode {
            name: project.name,
            path: project.root_path,
            is_dir: true,
            children,
        })
    }
}

fn read_children(directory: &Path, depth: usize, budget: &mut usize) -> Vec<FileNode> {
    if depth == 0 || *budget == 0 {
        return Vec::new();
    }

    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };

    let mut items = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if SKIPPED_DIRECTORIES.contains(&name.as_str()) {
            continue;
        }
        let is_dir = entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false);
        items.push((is_dir, name, entry.path()));
    }

    items.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.to_lowercase().cmp(&right.1.to_lowercase()))
    });

    let mut nodes = Vec::new();
    for (is_dir, name, path) in items {
        if *budget == 0 {
            break;
        }
        *budget -= 1;
        let children = if is_dir {
            read_children(&path, depth - 1, budget)
        } else {
            Vec::new()
        };
        nodes.push(FileNode {
            name,
            path: path.to_string_lossy().into_owned(),
            is_dir,
            children,
        });
    }

    nodes
}

fn unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or_default()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn service() -> (TempDir, ProjectService) {
        let directory = tempfile::tempdir().unwrap();
        let service = ProjectService::open(&directory.path().join("state.sqlite")).unwrap();
        (directory, service)
    }

    fn project_directory(parent: &Path, name: &str) -> PathBuf {
        let path = parent.join(name);
        fs::create_dir_all(path.join("src")).unwrap();
        fs::write(path.join("src").join("main.rs"), "fn main() {}\n").unwrap();
        path
    }

    #[test]
    fn rejects_a_missing_path() {
        let (directory, service) = service();
        let missing = directory.path().join("missing");

        let error = service.add(&missing.to_string_lossy()).unwrap_err();

        assert!(matches!(error, CoreError::InvalidPath(_)));
        assert!(service.list().unwrap().is_empty());
    }

    #[test]
    fn adds_and_lists_a_project() {
        let (directory, service) = service();
        let path = project_directory(directory.path(), "demo");

        let project = service.add(&path.to_string_lossy()).unwrap();
        let listed = service.list().unwrap();

        assert_eq!(project.name, "demo");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0], project);
    }

    #[test]
    fn opening_a_project_updates_recency() {
        let (directory, service) = service();
        let path = project_directory(directory.path(), "demo");
        let project = service.add(&path.to_string_lossy()).unwrap();

        let opened = service.open_project(&project.id).unwrap();

        assert_eq!(opened.id, project.id);
        assert!(opened.last_opened_at >= project.last_opened_at);
    }

    #[test]
    fn removes_a_project() {
        let (directory, service) = service();
        let path = project_directory(directory.path(), "demo");
        let project = service.add(&path.to_string_lossy()).unwrap();

        assert!(service.remove(&project.id).unwrap());
        assert!(!service.remove(&project.id).unwrap());
        assert!(service.list().unwrap().is_empty());
    }

    #[test]
    fn creates_updates_lists_and_removes_tasks() {
        let (directory, service) = service();
        let path = project_directory(directory.path(), "demo");
        let project = service.add(&path.to_string_lossy()).unwrap();

        let task = service.create_task(&project.id, "Revisar UI").unwrap();
        assert_eq!(task.status, crate::tasks::TaskStatus::Backlog);
        assert_eq!(service.tasks(&project.id).unwrap().len(), 1);

        let mut updated = task.clone();
        updated.status = crate::tasks::TaskStatus::InProgress;
        updated.branch = Some("feature/ui".to_owned());
        service.update_task(&updated).unwrap();
        let listed = service.tasks(&project.id).unwrap();
        assert_eq!(listed[0].status, crate::tasks::TaskStatus::InProgress);
        assert_eq!(listed[0].branch.as_deref(), Some("feature/ui"));

        assert!(service.remove_task(&task.id).unwrap());
        assert!(service.tasks(&project.id).unwrap().is_empty());
    }

    #[test]
    fn returns_the_project_tree_without_skipped_directories() {
        let (directory, service) = service();
        let path = project_directory(directory.path(), "demo");
        fs::create_dir_all(path.join("node_modules").join("pkg")).unwrap();
        let project = service.add(&path.to_string_lossy()).unwrap();

        let tree = service.tree(&project.id, Some(100)).unwrap();

        assert_eq!(tree.name, "demo");
        assert!(tree.children.iter().any(|node| node.name == "src"));
        assert!(!tree.children.iter().any(|node| node.name == "node_modules"));
        let src = tree
            .children
            .iter()
            .find(|node| node.name == "src")
            .unwrap();
        assert!(src.children.iter().any(|node| node.name == "main.rs"));
    }
}
