use std::sync::Mutex;

use std::path::Path;
use std::sync::Arc;

use ancleto_code_core::git::{GitBranch, GitStatus};
use ancleto_code_core::ide::SyncResult;
use ancleto_code_core::providers::{
    CommandOutcome, DiscoveryState, InitOptions, MemoryNode, SpecProjectState, SpecProvider,
    SpecsCheck, WriterCheck, WriterConfig, WriterProvider, WriterState,
};
use ancleto_code_core::{
    Conversation, ConversationState, DriverRegistry, DriverStatus, FileEntry, FileNode, HostClient,
    Orchestrator, Project, ProjectService, SavedSession, SessionBackend, SessionInfo, SessionSpec,
    StartConversation, Turn,
};
use tauri::State;

pub struct AppState {
    pub config_dir: std::path::PathBuf,
    pub projects: Mutex<ProjectService>,
    pub sessions: Arc<dyn SessionBackend>,
    pub host: Option<Arc<HostClient>>,
    pub drivers: DriverRegistry,
    pub orchestrator: Orchestrator,
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
        .create_session(spec)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn session_write(state: State<'_, AppState>, id: String, data: Vec<u8>) -> Result<(), String> {
    state
        .sessions
        .write_session(&id, &data)
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
        .resize_session(&id, cols, rows)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn session_kill(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .sessions
        .kill_session(&id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn session_snapshot(state: State<'_, AppState>, id: String) -> Result<Vec<u8>, String> {
    state
        .sessions
        .snapshot_session(&id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn session_list(state: State<'_, AppState>) -> Result<Vec<SessionInfo>, String> {
    state
        .sessions
        .list_sessions()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn session_saved(state: State<'_, AppState>) -> Result<Vec<SavedSession>, String> {
    match state.host.as_ref() {
        Some(host) => host.saved_sessions().map_err(|error| error.to_string()),
        None => Ok(Vec::new()),
    }
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

#[tauri::command]
pub fn conversation_start(
    state: State<'_, AppState>,
    session_a: String,
    session_b: String,
    objective: String,
    max_turns: Option<u32>,
    turn_budget_ms: Option<u64>,
) -> Result<Conversation, String> {
    state
        .orchestrator
        .start(StartConversation {
            session_a,
            session_b,
            objective,
            max_turns,
            turn_budget_ms,
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn conversation_pause(
    state: State<'_, AppState>,
    id: String,
) -> Result<ConversationState, String> {
    state
        .orchestrator
        .pause(&id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn conversation_resume(
    state: State<'_, AppState>,
    id: String,
) -> Result<ConversationState, String> {
    state
        .orchestrator
        .resume(&id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn conversation_abort(
    state: State<'_, AppState>,
    id: String,
) -> Result<ConversationState, String> {
    state
        .orchestrator
        .abort(&id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn conversation_get(state: State<'_, AppState>, id: String) -> Result<Conversation, String> {
    state
        .orchestrator
        .conversation(&id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn conversation_turns(state: State<'_, AppState>, id: String) -> Result<Vec<Turn>, String> {
    state
        .orchestrator
        .turns(&id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn conversation_list(state: State<'_, AppState>) -> Result<Vec<Conversation>, String> {
    state
        .orchestrator
        .conversations()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn spec_status(state: State<'_, AppState>, id: String) -> Result<SpecProjectState, String> {
    let root = project_root(&state, &id)?;
    SpecProvider::detect(Path::new(&root))
        .project_state(Path::new(&root))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn spec_init(
    state: State<'_, AppState>,
    id: String,
    options: InitOptions,
) -> Result<CommandOutcome, String> {
    let root = project_root(&state, &id)?;
    SpecProvider::detect(Path::new(&root))
        .init(Path::new(&root), &options)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn spec_install(
    state: State<'_, AppState>,
    id: String,
    options: InitOptions,
) -> Result<CommandOutcome, String> {
    let root = project_root(&state, &id)?;
    SpecProvider::detect(Path::new(&root))
        .install(Path::new(&root), &options)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn spec_update(state: State<'_, AppState>, id: String) -> Result<CommandOutcome, String> {
    let root = project_root(&state, &id)?;
    SpecProvider::detect(Path::new(&root))
        .update(Path::new(&root))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn spec_upgrade(state: State<'_, AppState>, id: String) -> Result<CommandOutcome, String> {
    let root = project_root(&state, &id)?;
    SpecProvider::detect(Path::new(&root))
        .upgrade(Path::new(&root))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn spec_check(state: State<'_, AppState>, id: String) -> Result<CommandOutcome, String> {
    let root = project_root(&state, &id)?;
    SpecProvider::detect(Path::new(&root))
        .check(Path::new(&root))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn spec_doctor(state: State<'_, AppState>, id: String) -> Result<CommandOutcome, String> {
    let root = project_root(&state, &id)?;
    SpecProvider::detect(Path::new(&root))
        .doctor(Path::new(&root))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn spec_specs_check(
    state: State<'_, AppState>,
    id: String,
    change: Option<String>,
) -> Result<SpecsCheck, String> {
    let root = project_root(&state, &id)?;
    SpecProvider::detect(Path::new(&root))
        .specs_check(Path::new(&root), change.as_deref())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn spec_discovery_check(
    state: State<'_, AppState>,
    id: String,
) -> Result<DiscoveryState, String> {
    let root = project_root(&state, &id)?;
    SpecProvider::detect(Path::new(&root))
        .discovery_check(Path::new(&root))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn spec_discovery_generate(
    state: State<'_, AppState>,
    id: String,
) -> Result<CommandOutcome, String> {
    let root = project_root(&state, &id)?;
    SpecProvider::detect(Path::new(&root))
        .discovery_generate(Path::new(&root))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn spec_memory_list(state: State<'_, AppState>, id: String) -> Result<Vec<MemoryNode>, String> {
    let root = project_root(&state, &id)?;
    SpecProvider::detect(Path::new(&root))
        .memory_list(Path::new(&root))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn spec_memory_context(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let root = project_root(&state, &id)?;
    SpecProvider::detect(Path::new(&root))
        .memory_context(Path::new(&root))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn writer_status(state: State<'_, AppState>, id: String) -> Result<WriterState, String> {
    let root = project_root(&state, &id)?;
    WriterProvider::detect(Path::new(&root))
        .state(Path::new(&root))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn writer_init(state: State<'_, AppState>, id: String) -> Result<CommandOutcome, String> {
    with_writer(&state, &id, |provider, root| provider.init(root))
}

#[tauri::command]
pub fn writer_build(state: State<'_, AppState>, id: String) -> Result<CommandOutcome, String> {
    with_writer(&state, &id, |provider, root| provider.build(root))
}

#[tauri::command]
pub fn writer_check(state: State<'_, AppState>, id: String) -> Result<WriterCheck, String> {
    with_writer(&state, &id, |provider, root| provider.check(root))
}

#[tauri::command]
pub fn writer_lint(state: State<'_, AppState>, id: String) -> Result<CommandOutcome, String> {
    with_writer(&state, &id, |provider, root| provider.lint(root))
}

#[tauri::command]
pub fn writer_impact(
    state: State<'_, AppState>,
    id: String,
    files: Vec<String>,
) -> Result<CommandOutcome, String> {
    with_writer(&state, &id, |provider, root| provider.impact(root, &files))
}

#[tauri::command]
pub fn writer_human(state: State<'_, AppState>, id: String) -> Result<CommandOutcome, String> {
    with_writer(&state, &id, |provider, root| provider.human(root))
}

#[tauri::command]
pub fn writer_doctor(state: State<'_, AppState>, id: String) -> Result<CommandOutcome, String> {
    with_writer(&state, &id, |provider, root| provider.doctor(root))
}

#[tauri::command]
pub fn writer_set_language_pack(
    state: State<'_, AppState>,
    id: String,
    enabled: bool,
    module: Option<String>,
) -> Result<WriterConfig, String> {
    with_writer(&state, &id, |provider, root| {
        provider.set_language_pack(root, enabled, module.as_deref())
    })
}

#[tauri::command]
pub fn writer_agent_doc(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> Result<String, String> {
    const ALLOWED: [&str; 5] = [
        "INDEX.md",
        "API.md",
        "SYMBOLS.md",
        "GOTCHAS.md",
        "ARCHITECTURE.md",
    ];
    if !ALLOWED.contains(&name.as_str()) {
        return Err(format!("documento no permitido: {name}"));
    }
    let root = project_root(&state, &id)?;
    let config = ancleto_code_core::providers::read_writer_config(Path::new(&root))
        .map_err(|error| error.to_string())?;
    std::fs::read_to_string(
        Path::new(&root)
            .join(config.output_dir)
            .join("agent")
            .join(&name),
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn writer_open_human(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let root = project_root(&state, &id)?;
    let config = ancleto_code_core::providers::read_writer_config(Path::new(&root))
        .map_err(|error| error.to_string())?;
    let index = Path::new(&root)
        .join(config.output_dir)
        .join("human")
        .join("index.html");
    if !index.is_file() {
        return Err("todavía no existe human/index.html; generá la vista primero".to_owned());
    }

    #[cfg(target_os = "macos")]
    let mut command = std::process::Command::new("open");
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = std::process::Command::new("cmd");
        command.args(["/C", "start", ""]);
        command
    };
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = std::process::Command::new("xdg-open");

    command
        .arg(index)
        .spawn()
        .map_err(|error| format!("no se pudo abrir la vista humana: {error}"))?;
    Ok(())
}

fn with_writer<T>(
    state: &State<'_, AppState>,
    id: &str,
    run: impl FnOnce(&WriterProvider, &Path) -> ancleto_code_core::Result<T>,
) -> Result<T, String> {
    let root = project_root(state, id)?;
    let provider = WriterProvider::detect(Path::new(&root));
    run(&provider, Path::new(&root)).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn settings_get(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    let service = state.projects.lock().map_err(|_| lock_error())?;
    service.get_setting(&key).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn settings_put(state: State<'_, AppState>, key: String, value: String) -> Result<(), String> {
    let service = state.projects.lock().map_err(|_| lock_error())?;
    service
        .set_setting(&key, &value)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn appearance_save_image(
    state: State<'_, AppState>,
    extension: String,
    bytes: Vec<u8>,
) -> Result<String, String> {
    const ALLOWED: [&str; 5] = ["png", "jpg", "jpeg", "webp", "gif"];
    let extension = extension.trim_start_matches('.').to_ascii_lowercase();
    if !ALLOWED.contains(&extension.as_str()) {
        return Err(format!("extensión de imagen no permitida: {extension}"));
    }

    let directory = state.config_dir.join("appearance");
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let path = directory.join(format!("{}.{extension}", uuid::Uuid::new_v4()));
    std::fs::write(&path, bytes).map_err(|error| error.to_string())?;
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn appearance_native_effect(
    window: tauri::WebviewWindow,
    effect: String,
    dark: Option<bool>,
) -> Result<(), String> {
    match effect.as_str() {
        "none" => Ok(()),
        "mica" => window_vibrancy::apply_mica(&window, dark).map_err(|error| error.to_string()),
        "acrylic" => window_vibrancy::apply_acrylic(&window, Some((18, 18, 18, 160)))
            .map_err(|error| error.to_string()),
        "blur" => window_vibrancy::apply_blur(&window, Some((18, 18, 18, 160)))
            .map_err(|error| error.to_string()),
        "vibrancy" => {
            #[cfg(target_os = "macos")]
            {
                window_vibrancy::apply_vibrancy(
                    &window,
                    window_vibrancy::NSVisualEffectMaterial::HudWindow,
                    Some(window_vibrancy::NSVisualEffectState::Active),
                    Some(12.0),
                )
                .map_err(|error| error.to_string())
            }
            #[cfg(not(target_os = "macos"))]
            {
                let _ = dark;
                Err("vibrancy solo está disponible en macOS".to_owned())
            }
        }
        other => Err(format!("efecto de ventana desconocido: {other}")),
    }
}

#[tauri::command]
pub fn project_search(
    state: State<'_, AppState>,
    id: String,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<FileEntry>, String> {
    let service = state.projects.lock().map_err(|_| lock_error())?;
    service
        .search(&id, &query, limit)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn git_status(state: State<'_, AppState>, id: String) -> Result<GitStatus, String> {
    let root = project_root(&state, &id)?;
    ancleto_code_core::git::status(Path::new(&root)).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn git_branches(state: State<'_, AppState>, id: String) -> Result<Vec<GitBranch>, String> {
    let root = project_root(&state, &id)?;
    ancleto_code_core::git::branches(Path::new(&root)).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn git_create_branch(
    state: State<'_, AppState>,
    id: String,
    name: String,
    checkout: bool,
) -> Result<GitStatus, String> {
    let root = project_root(&state, &id)?;
    ancleto_code_core::git::create_branch(Path::new(&root), &name, checkout)
        .map_err(|error| error.to_string())?;
    ancleto_code_core::git::status(Path::new(&root)).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn git_checkout(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> Result<GitStatus, String> {
    let root = project_root(&state, &id)?;
    ancleto_code_core::git::checkout(Path::new(&root), &name).map_err(|error| error.to_string())?;
    ancleto_code_core::git::status(Path::new(&root)).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn git_diff(
    state: State<'_, AppState>,
    id: String,
    path: Option<String>,
    staged: bool,
) -> Result<String, String> {
    let root = project_root(&state, &id)?;
    ancleto_code_core::git::diff(Path::new(&root), path.as_deref(), staged)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn git_stage(state: State<'_, AppState>, id: String, paths: Vec<String>) -> Result<(), String> {
    let root = project_root(&state, &id)?;
    ancleto_code_core::git::stage(Path::new(&root), &paths).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn git_unstage(
    state: State<'_, AppState>,
    id: String,
    paths: Vec<String>,
) -> Result<(), String> {
    let root = project_root(&state, &id)?;
    ancleto_code_core::git::unstage(Path::new(&root), &paths).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn git_commit(
    state: State<'_, AppState>,
    id: String,
    message: String,
) -> Result<GitStatus, String> {
    let root = project_root(&state, &id)?;
    ancleto_code_core::git::commit(Path::new(&root), &message)
        .map_err(|error| error.to_string())?;
    ancleto_code_core::git::status(Path::new(&root)).map_err(|error| error.to_string())
}
