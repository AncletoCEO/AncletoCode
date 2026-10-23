use std::fs;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::{Conversation, ConversationState, Project, Result, Turn, TurnMode};
use crate::session::{SessionSpec, SessionStatus};
use crate::tasks::{Task, TaskStatus};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedSession {
    pub id: String,
    pub spec: SessionSpec,
    pub status: SessionStatus,
    pub pid: Option<u32>,
    pub created_at: i64,
}

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
             CREATE TABLE IF NOT EXISTS conversations (
                 id TEXT PRIMARY KEY,
                 session_a TEXT NOT NULL,
                 session_b TEXT NOT NULL,
                 objective TEXT NOT NULL,
                 max_turns INTEGER NOT NULL,
                 turn_budget_ms INTEGER NOT NULL,
                 state TEXT NOT NULL,
                 reason TEXT,
                 created_at INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS turns (
                 conversation_id TEXT NOT NULL,
                 turn_index INTEGER NOT NULL,
                 from_session TEXT NOT NULL,
                 to_session TEXT NOT NULL,
                 text TEXT NOT NULL,
                 mode TEXT NOT NULL,
                 duration_ms INTEGER NOT NULL,
                 created_at INTEGER NOT NULL,
                 PRIMARY KEY (conversation_id, turn_index)
             );
             INSERT OR IGNORE INTO schema_migrations (version, applied_at)
                 VALUES (1, unixepoch());
             CREATE TABLE IF NOT EXISTS host_sessions (
                 id TEXT PRIMARY KEY,
                 spec TEXT NOT NULL,
                 status TEXT NOT NULL,
                 pid INTEGER,
                 created_at INTEGER NOT NULL
             );
             INSERT OR IGNORE INTO schema_migrations (version, applied_at)
                 VALUES (2, unixepoch());
             INSERT OR IGNORE INTO schema_migrations (version, applied_at)
                 VALUES (3, unixepoch());
             CREATE TABLE IF NOT EXISTS tasks (
                 id TEXT PRIMARY KEY,
                 project_id TEXT NOT NULL,
                 title TEXT NOT NULL,
                 status TEXT NOT NULL,
                 branch TEXT,
                 worktree_path TEXT,
                 session_id TEXT,
                 pr_url TEXT,
                 created_at INTEGER NOT NULL,
                 updated_at INTEGER NOT NULL,
                 FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
             );
             CREATE INDEX IF NOT EXISTS tasks_project_updated
                 ON tasks (project_id, updated_at DESC);
             INSERT OR IGNORE INTO schema_migrations (version, applied_at)
                 VALUES (4, unixepoch());",
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

    pub fn list_tasks(&self, project_id: &str) -> Result<Vec<Task>> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, title, status, branch, worktree_path, session_id,
                    pr_url, created_at, updated_at
             FROM tasks WHERE project_id = ?1
             ORDER BY updated_at DESC, created_at DESC",
        )?;
        let rows = statement.query_map(params![project_id], map_task)?;
        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(row?);
        }
        Ok(tasks)
    }

    pub fn find_task(&self, id: &str) -> Result<Option<Task>> {
        self.connection
            .query_row(
                "SELECT id, project_id, title, status, branch, worktree_path, session_id,
                        pr_url, created_at, updated_at
                 FROM tasks WHERE id = ?1",
                params![id],
                map_task,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn insert_task(&self, task: &Task) -> Result<()> {
        self.connection.execute(
            "INSERT INTO tasks
             (id, project_id, title, status, branch, worktree_path, session_id, pr_url,
              created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                task.id,
                task.project_id,
                task.title,
                task.status.as_str(),
                task.branch,
                task.worktree_path,
                task.session_id,
                task.pr_url,
                task.created_at,
                task.updated_at
            ],
        )?;
        Ok(())
    }

    pub fn update_task(&self, task: &Task) -> Result<()> {
        self.connection.execute(
            "UPDATE tasks SET title = ?2, status = ?3, branch = ?4, worktree_path = ?5,
                    session_id = ?6, pr_url = ?7, updated_at = ?8
             WHERE id = ?1",
            params![
                task.id,
                task.title,
                task.status.as_str(),
                task.branch,
                task.worktree_path,
                task.session_id,
                task.pr_url,
                task.updated_at
            ],
        )?;
        Ok(())
    }

    pub fn delete_task(&self, id: &str) -> Result<bool> {
        let affected = self
            .connection
            .execute("DELETE FROM tasks WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    pub fn create_conversation(&self, conversation: &Conversation) -> Result<()> {
        self.connection.execute(
            "INSERT INTO conversations
             (id, session_a, session_b, objective, max_turns, turn_budget_ms, state, reason, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                conversation.id,
                conversation.session_a,
                conversation.session_b,
                conversation.objective,
                conversation.max_turns,
                conversation.turn_budget_ms as i64,
                state_to_str(conversation.state),
                conversation.reason,
                conversation.created_at
            ],
        )?;
        Ok(())
    }

    pub fn update_conversation_state(
        &self,
        id: &str,
        state: ConversationState,
        reason: Option<&str>,
    ) -> Result<()> {
        self.connection.execute(
            "UPDATE conversations SET state = ?2, reason = ?3 WHERE id = ?1",
            params![id, state_to_str(state), reason],
        )?;
        Ok(())
    }

    pub fn get_conversation(&self, id: &str) -> Result<Option<Conversation>> {
        self.connection
            .query_row(
                "SELECT id, session_a, session_b, objective, max_turns, turn_budget_ms,
                        state, reason, created_at
                 FROM conversations WHERE id = ?1",
                params![id],
                map_conversation,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_conversations(&self) -> Result<Vec<Conversation>> {
        let mut statement = self.connection.prepare(
            "SELECT id, session_a, session_b, objective, max_turns, turn_budget_ms,
                    state, reason, created_at
             FROM conversations ORDER BY created_at DESC",
        )?;
        let rows = statement.query_map([], map_conversation)?;
        let mut conversations = Vec::new();
        for row in rows {
            conversations.push(row?);
        }
        Ok(conversations)
    }

    pub fn add_turn(&self, turn: &Turn) -> Result<()> {
        self.connection.execute(
            "INSERT INTO turns
             (conversation_id, turn_index, from_session, to_session, text, mode, duration_ms, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                turn.conversation_id,
                turn.index,
                turn.from_session,
                turn.to_session,
                turn.text,
                mode_to_str(turn.mode),
                turn.duration_ms as i64,
                turn.created_at
            ],
        )?;
        Ok(())
    }

    pub fn list_turns(&self, conversation_id: &str) -> Result<Vec<Turn>> {
        let mut statement = self.connection.prepare(
            "SELECT conversation_id, turn_index, from_session, to_session, text, mode,
                    duration_ms, created_at
             FROM turns WHERE conversation_id = ?1 ORDER BY turn_index ASC",
        )?;
        let rows = statement.query_map(params![conversation_id], map_turn)?;
        let mut turns = Vec::new();
        for row in rows {
            turns.push(row?);
        }
        Ok(turns)
    }

    pub fn save_session(&self, session: &SavedSession) -> Result<()> {
        let spec = serde_json::to_string(&session.spec)
            .map_err(|error| crate::domain::CoreError::InvalidInput(error.to_string()))?;
        let status = serde_json::to_value(session.status)
            .ok()
            .and_then(|value| value.as_str().map(str::to_owned))
            .unwrap_or_else(|| "crashed".to_owned());
        self.connection.execute(
            "INSERT INTO host_sessions (id, spec, status, pid, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                 spec = excluded.spec,
                 status = excluded.status,
                 pid = excluded.pid",
            params![session.id, spec, status, session.pid, session.created_at],
        )?;
        Ok(())
    }

    pub fn set_session_status(&self, id: &str, status: SessionStatus) -> Result<()> {
        let status = serde_json::to_value(status)
            .ok()
            .and_then(|value| value.as_str().map(str::to_owned))
            .unwrap_or_else(|| "crashed".to_owned());
        self.connection.execute(
            "UPDATE host_sessions SET status = ?2 WHERE id = ?1",
            params![id, status],
        )?;
        Ok(())
    }

    pub fn saved_sessions(&self) -> Result<Vec<SavedSession>> {
        let mut statement = self.connection.prepare(
            "SELECT id, spec, status, pid, created_at FROM host_sessions
             ORDER BY created_at DESC",
        )?;
        let rows = statement.query_map([], |row| {
            let spec: String = row.get(1)?;
            let status: String = row.get(2)?;
            Ok((
                row.get::<_, String>(0)?,
                spec,
                status,
                row.get(3)?,
                row.get(4)?,
            ))
        })?;

        let mut sessions = Vec::new();
        for row in rows {
            let (id, spec, status, pid, created_at) = row?;
            let Ok(spec) = serde_json::from_str::<SessionSpec>(&spec) else {
                continue;
            };
            let status = serde_json::from_value::<SessionStatus>(serde_json::Value::String(status))
                .unwrap_or(SessionStatus::Crashed);
            sessions.push(SavedSession {
                id,
                spec,
                status,
                pid,
                created_at,
            });
        }
        Ok(sessions)
    }

    pub fn crash_active_sessions(&self) -> Result<usize> {
        let affected = self.connection.execute(
            "UPDATE host_sessions SET status = 'crashed'
             WHERE status NOT IN ('stopped', 'crashed')",
            [],
        )?;
        Ok(affected)
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

fn state_to_str(state: ConversationState) -> &'static str {
    match state {
        ConversationState::Running => "running",
        ConversationState::Paused => "paused",
        ConversationState::Completed => "completed",
        ConversationState::Aborted => "aborted",
        ConversationState::Failed => "failed",
    }
}

fn state_from_str(value: &str) -> ConversationState {
    match value {
        "paused" => ConversationState::Paused,
        "completed" => ConversationState::Completed,
        "aborted" => ConversationState::Aborted,
        "failed" => ConversationState::Failed,
        _ => ConversationState::Running,
    }
}

fn mode_to_str(mode: TurnMode) -> &'static str {
    match mode {
        TurnMode::OneShot => "one_shot",
        TurnMode::Pty => "pty",
    }
}

fn mode_from_str(value: &str) -> TurnMode {
    match value {
        "pty" => TurnMode::Pty,
        _ => TurnMode::OneShot,
    }
}

fn map_conversation(row: &rusqlite::Row<'_>) -> rusqlite::Result<Conversation> {
    let state: String = row.get(6)?;
    Ok(Conversation {
        id: row.get(0)?,
        session_a: row.get(1)?,
        session_b: row.get(2)?,
        objective: row.get(3)?,
        max_turns: row.get(4)?,
        turn_budget_ms: row.get::<_, i64>(5)? as u64,
        state: state_from_str(&state),
        reason: row.get(7)?,
        created_at: row.get(8)?,
    })
}

fn map_turn(row: &rusqlite::Row<'_>) -> rusqlite::Result<Turn> {
    let mode: String = row.get(5)?;
    Ok(Turn {
        conversation_id: row.get(0)?,
        index: row.get(1)?,
        from_session: row.get(2)?,
        to_session: row.get(3)?,
        text: row.get(4)?,
        mode: mode_from_str(&mode),
        duration_ms: row.get::<_, i64>(6)? as u64,
        created_at: row.get(7)?,
    })
}

fn map_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    let status: String = row.get(3)?;
    Ok(Task {
        id: row.get(0)?,
        project_id: row.get(1)?,
        title: row.get(2)?,
        status: TaskStatus::parse(&status).unwrap_or(TaskStatus::Backlog),
        branch: row.get(4)?,
        worktree_path: row.get(5)?,
        session_id: row.get(6)?,
        pr_url: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
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
