use std::sync::Mutex;

use ancleto_code_core::ide::SyncResult;
use ancleto_code_core::{
    DriverRegistry, DriverStatus, FileNode, Project, ProjectService, SessionInfo, SessionManager,
    SessionSpec,
};
use tauri::State;

pub struct AppState {
    pub projects: Mutex<ProjectService>,
    pub sessions: SessionManager,
    pub drivers: DriverRegistry,
}

fn lock_error() -> String {
    "el estado de proyectos no está disponible".to_owned()
}

#[tauri::command]
pub fn core_version() -> &'static str {
    ancleto_code_core::version()
}

#[tauri::command]
pub fn project_add(state: State<'_, AppState>, path: String) -> Result<Project, String> {
    let service = state.projects.lock().map_err(|_| lock_error())?;
    service.add(&path).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn project_list(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let service = state.projects.lock().map_err(|_| lock_error())?;
    service.list().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn project_remove(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    let service = state.projects.lock().map_err(|_| lock_error())?;
    service.remove(&id).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn project_open(state: State<'_, AppState>, id: String) -> Result<Project, String> {
    let service = state.projects.lock().map_err(|_| lock_error())?;
    service.open_project(&id).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn project_tree(
    state: State<'_, AppState>,
    id: String,
    limit: Option<usize>,
) -> Result<FileNode, String> {
    let service = state.projects.lock().map_err(|_| lock_error())?;
    service.tree(&id, limit).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn session_create(
    state: State<'_, AppState>,
    spec: SessionSpec,
) -> Result<SessionInfo, String> {
    state
        .sessions
        .create(spec)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn session_write(state: State<'_, AppState>, id: String, data: Vec<u8>) -> Result<(), String> {
    state
        .sessions
        .write(&id, &data)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn session_resize(
    state: State<'_, AppState>,
    id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    state
        .sessions
        .resize(&id, cols, rows)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn session_kill(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.sessions.kill(&id).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn session_snapshot(state: State<'_, AppState>, id: String) -> Result<Vec<u8>, String> {
    state
        .sessions
        .snapshot(&id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn session_list(state: State<'_, AppState>) -> Result<Vec<SessionInfo>, String> {
    state.sessions.list().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn drivers_list(state: State<'_, AppState>) -> Vec<DriverStatus> {
    state.drivers.detect_all()
}

fn project_root(state: &State<'_, AppState>, id: &str) -> Result<String, String> {
    let service = state.projects.lock().map_err(|_| lock_error())?;
    service
        .get(id)
        .map(|project| project.root_path)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn ide_open_antigravity(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let root = project_root(&state, &id)?;
    ancleto_code_core::ide::open_in_antigravity(&root).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn ide_sync_antigravity(state: State<'_, AppState>, id: String) -> Result<SyncResult, String> {
    let root = project_root(&state, &id)?;
    ancleto_code_core::ide::sync_antigravity_assets(&root).map_err(|error| error.to_string())
}
