use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub created_at: i64,
    pub last_opened_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: String,
    pub session_a: String,
    pub session_b: String,
    pub objective: String,
    pub max_turns: u32,
    pub turn_budget_ms: u64,
    pub state: ConversationState,
    pub reason: Option<String>,
    pub created_at: i64,
    #[serde(default)]
    pub model_a: Option<String>,
    #[serde(default)]
    pub model_b: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationState {
    Running,
    Paused,
    Completed,
    Aborted,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TurnMode {
    OneShot,
    Pty,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Turn {
    pub conversation_id: String,
    pub index: u32,
    pub from_session: String,
    pub to_session: String,
    pub text: String,
    pub mode: TurnMode,
    pub duration_ms: u64,
    pub created_at: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("no existe la ruta: {0}")]
    InvalidPath(String),
    #[error("la ruta no es un directorio: {0}")]
    NotADirectory(String),
    #[error("proyecto no encontrado: {0}")]
    ProjectNotFound(String),
    #[error("sesión no encontrada: {0}")]
    SessionNotFound(String),
    #[error("error de terminal: {0}")]
    Pty(String),
    #[error("error de proceso: {0}")]
    Process(String),
    #[error("entrada inválida: {0}")]
    InvalidInput(String),
    #[error("error del host de sesiones: {0}")]
    Host(String),
    #[error("error de base de datos: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("error de E/S: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, CoreError>;
