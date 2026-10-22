use std::fs;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::{Project, Result};

pub struct Store {
    connection: Connection,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let connection = Connection::open(path)?;
        connection.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;
             CREATE TABLE IF NOT EXISTS schema_migrations (
                 version INTEGER PRIMARY KEY,
                 applied_at INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS projects (
                 id TEXT PRIMARY KEY,
                 name TEXT NOT NULL,
                 root_path TEXT NOT NULL UNIQUE,
                 created_at INTEGER NOT NULL,
                 last_opened_at INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS projects_last_opened_at
                 ON projects (last_opened_at DESC);
             CREATE TABLE IF NOT EXISTS settings (
                 key TEXT PRIMARY KEY,
                 value TEXT NOT NULL
             );
             INSERT OR IGNORE INTO schema_migrations (version, applied_at)
                 VALUES (1, unixepoch());",
        )?;

        Ok(Self { connection })
    }

    pub fn list_projects(&self) -> Result<Vec<Project>> {
        let mut statement = self.connection.prepare(
            "SELECT id, name, root_path, created_at, last_opened_at
             FROM projects
             ORDER BY last_opened_at DESC, name ASC",
        )?;
        let rows = statement.query_map([], map_project)?;
        let mut projects = Vec::new();
        for row in rows {
            projects.push(row?);
        }
        Ok(projects)
    }

    pub fn find_project_by_id(&self, id: &str) -> Result<Option<Project>> {
        self.connection
            .query_row(
                "SELECT id, name, root_path, created_at, last_opened_at
                 FROM projects WHERE id = ?1",
                params![id],
                map_project,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn find_project_by_root(&self, root_path: &str) -> Result<Option<Project>> {
        self.connection
            .query_row(
                "SELECT id, name, root_path, created_at, last_opened_at
                 FROM projects WHERE root_path = ?1",
                params![root_path],
                map_project,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn insert_project(&self, project: &Project) -> Result<()> {
        self.connection.execute(
            "INSERT INTO projects (id, name, root_path, created_at, last_opened_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                project.id,
                project.name,
                project.root_path,
                project.created_at,
                project.last_opened_at
            ],
        )?;
        Ok(())
    }

    pub fn touch_project(&self, id: &str, timestamp: i64) -> Result<Option<Project>> {
        self.connection.execute(
            "UPDATE projects SET last_opened_at = ?2 WHERE id = ?1",
            params![id, timestamp],
        )?;
        self.find_project_by_id(id)
    }

    pub fn remove_project(&self, id: &str) -> Result<bool> {
        let affected = self
            .connection
            .execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        self.connection
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.connection.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
}

fn map_project(row: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        root_path: row.get(2)?,
        created_at: row.get(3)?,
        last_opened_at: row.get(4)?,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn persists_and_updates_settings() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::open(&directory.path().join("state.sqlite")).unwrap();

        assert_eq!(store.get_setting("theme").unwrap(), None);
        store.set_setting("theme", "nord").unwrap();
        store.set_setting("theme", "carbon").unwrap();

        assert_eq!(
            store.get_setting("theme").unwrap().as_deref(),
            Some("carbon")
        );
    }

    #[test]
    fn keeps_projects_across_reopen() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("state.sqlite");
        let project = Project {
            id: "p1".to_owned(),
            name: "demo".to_owned(),
            root_path: "/tmp/demo".to_owned(),
            created_at: 1,
            last_opened_at: 1,
        };

        {
            let store = Store::open(&database).unwrap();
            store.insert_project(&project).unwrap();
        }

        let store = Store::open(&database).unwrap();
        assert_eq!(store.list_projects().unwrap(), vec![project]);
    }
}
