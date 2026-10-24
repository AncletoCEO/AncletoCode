use std::sync::Mutex;

use std::path::Path;
use std::sync::Arc;

use ancleto_code_core::git::{
    GitActionOutcome, GitBranch, GitIssue, GitPullRequest, GitRemote, GitStatus, GitWorktree,
};
use ancleto_code_core::ide::SyncResult;
use ancleto_code_core::providers::{
    CommandOutcome, DiscoveryState, InitOptions, MemoryNode, SpecProjectState, SpecProvider,
    SpecsCheck, WriterCheck, WriterConfig, WriterProvider, WriterState,
};
use ancleto_code_core::{
    Conversation, ConversationState, DriverRegistry, DriverStatus, FileEntry, FileNode, HostClient,
    Orchestrator, Project, ProjectService, SavedSession, SessionBackend, SessionInfo, SessionSpec,
    StartConversation, Task, Turn,
};
use tauri::State;

pub struct AppState {
    pub config_dir: std::path::PathBuf,
    pub resource_dir: std::path::PathBuf,
    pub projects: Arc<Mutex<ProjectService>>,
    pub sessions: Arc<dyn SessionBackend>,
    pub host: Option<Arc<HostClient>>,
    pub drivers: DriverRegistry,
    pub orchestrator: Orchestrator,
    pub usage: Arc<crate::usage::UsageService>,
}

fn lock_error() -> String {
    "el estado de proyectos no está disponible".to_owned()
}

async fn run_blocking<T, F>(work: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|error| format!("la tarea en segundo plano falló: {error}"))?
}

async fn with_project_root<T, F>(
    state: State<'_, AppState>,
    id: String,
    run: F,
) -> Result<T, String>
where
    F: FnOnce(String) -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    let root = project_root(&state, &id)?;
    run_blocking(move || run(root)).await
}

async fn with_spec<T, F>(state: State<'_, AppState>, id: String, run: F) -> Result<T, String>
where
    F: FnOnce(&SpecProvider, &Path) -> ancleto_code_core::Result<T> + Send + 'static,
    T: Send + 'static,
{
    let root = project_root(&state, &id)?;
    run_blocking(move || {
        let provider = SpecProvider::detect(Path::new(&root));
        run(&provider, Path::new(&root)).map_err(|error| error.to_string())
    })
    .await
}

async fn with_writer<T, F>(state: State<'_, AppState>, id: String, run: F) -> Result<T, String>
where
    F: FnOnce(&WriterProvider, &Path) -> ancleto_code_core::Result<T> + Send + 'static,
    T: Send + 'static,
{
    let root = project_root(&state, &id)?;
    run_blocking(move || {
        let provider = WriterProvider::detect(Path::new(&root));
        run(&provider, Path::new(&root)).map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn ancleto_runtime_status(
    state: State<'_, AppState>,
    check_remote: Option<bool>,
) -> Result<crate::ancleto_runtime::AncletoRuntimeStatus, String> {
    let config = state.config_dir.clone();
    let resource = state.resource_dir.clone();
    let check = check_remote.unwrap_or(false);
    run_blocking(move || Ok(crate::ancleto_runtime::status(&config, &resource, check))).await
}

#[tauri::command]
pub async fn ancleto_runtime_update(
    state: State<'_, AppState>,
) -> Result<crate::ancleto_runtime::AncletoRuntimeStatus, String> {
    let config = state.config_dir.clone();
    let resource = state.resource_dir.clone();
    run_blocking(move || crate::ancleto_runtime::update(&config, &resource)).await
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
pub async fn project_tree(
    state: State<'_, AppState>,
    id: String,
    limit: Option<usize>,
) -> Result<FileNode, String> {
    let service = Arc::clone(&state.projects);
    run_blocking(move || {
        let service = service.lock().map_err(|_| lock_error())?;
        service.tree(&id, limit).map_err(|error| error.to_string())
    })
    .await
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
pub async fn session_snapshot(state: State<'_, AppState>, id: String) -> Result<Vec<u8>, String> {
    let sessions = Arc::clone(&state.sessions);
    run_blocking(move || {
        sessions
            .snapshot_session(&id)
            .map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub fn session_list(state: State<'_, AppState>) -> Result<Vec<SessionInfo>, String> {
    state
        .sessions
        .list_sessions()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn session_saved(state: State<'_, AppState>) -> Result<Vec<SavedSession>, String> {
    let Some(host) = state.host.as_ref().map(Arc::clone) else {
        return Ok(Vec::new());
    };
    run_blocking(move || host.saved_sessions().map_err(|error| error.to_string())).await
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
pub async fn ide_open_antigravity(state: State<'_, AppState>, id: String) -> Result<(), String> {
    with_project_root(state, id, |root| {
        ancleto_code_core::ide::open_in_antigravity(&root).map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn ide_sync_antigravity(
    state: State<'_, AppState>,
    id: String,
) -> Result<SyncResult, String> {
    with_project_root(state, id, |root| {
        ancleto_code_core::ide::sync_antigravity_assets(&root).map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn conversation_start(
    state: State<'_, AppState>,
    session_a: String,
    session_b: String,
    objective: String,
    max_turns: Option<u32>,
    turn_budget_ms: Option<u64>,
    model_a: Option<String>,
    model_b: Option<String>,
    allow_tools: Option<bool>,
) -> Result<Conversation, String> {
    state
        .orchestrator
        .start(StartConversation {
            session_a,
            session_b,
            objective,
            max_turns,
            turn_budget_ms,
            model_a,
            model_b,
            allow_tools: allow_tools.unwrap_or(false),
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
pub async fn spec_status(
    state: State<'_, AppState>,
    id: String,
) -> Result<SpecProjectState, String> {
    with_spec(state, id, |provider, root| provider.project_state(root)).await
}

#[tauri::command]
pub async fn spec_init(
    state: State<'_, AppState>,
    id: String,
    options: InitOptions,
) -> Result<CommandOutcome, String> {
    with_spec(state, id, move |provider, root| {
        provider.init(root, &options)
    })
    .await
}

#[tauri::command]
pub async fn spec_install(
    state: State<'_, AppState>,
    id: String,
    options: InitOptions,
) -> Result<CommandOutcome, String> {
    with_spec(state, id, move |provider, root| {
        provider.install(root, &options)
    })
    .await
}

#[tauri::command]
pub async fn spec_update(state: State<'_, AppState>, id: String) -> Result<CommandOutcome, String> {
    with_spec(state, id, |provider, root| provider.update(root)).await
}

#[tauri::command]
pub async fn spec_upgrade(
    state: State<'_, AppState>,
    id: String,
) -> Result<CommandOutcome, String> {
    with_spec(state, id, |provider, root| provider.upgrade(root)).await
}

#[tauri::command]
pub async fn spec_check(state: State<'_, AppState>, id: String) -> Result<CommandOutcome, String> {
    with_spec(state, id, |provider, root| provider.check(root)).await
}

#[tauri::command]
pub async fn spec_doctor(state: State<'_, AppState>, id: String) -> Result<CommandOutcome, String> {
    with_spec(state, id, |provider, root| provider.doctor(root)).await
}

#[tauri::command]
pub async fn spec_specs_check(
    state: State<'_, AppState>,
    id: String,
    change: Option<String>,
) -> Result<SpecsCheck, String> {
    with_spec(state, id, move |provider, root| {
        provider.specs_check(root, change.as_deref())
    })
    .await
}

#[tauri::command]
pub async fn spec_discovery_check(
    state: State<'_, AppState>,
    id: String,
) -> Result<DiscoveryState, String> {
    with_spec(state, id, |provider, root| provider.discovery_check(root)).await
}

#[tauri::command]
pub async fn spec_discovery_generate(
    state: State<'_, AppState>,
    id: String,
) -> Result<CommandOutcome, String> {
    with_spec(state, id, |provider, root| {
        provider.discovery_generate(root)
    })
    .await
}

#[tauri::command]
pub async fn spec_memory_list(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<MemoryNode>, String> {
    with_spec(state, id, |provider, root| provider.memory_list(root)).await
}

#[tauri::command]
pub async fn spec_memory_context(state: State<'_, AppState>, id: String) -> Result<String, String> {
    with_spec(state, id, |provider, root| provider.memory_context(root)).await
}

#[tauri::command]
pub async fn writer_status(state: State<'_, AppState>, id: String) -> Result<WriterState, String> {
    with_writer(state, id, |provider, root| provider.state(root)).await
}

#[tauri::command]
pub async fn writer_init(state: State<'_, AppState>, id: String) -> Result<CommandOutcome, String> {
    with_writer(state, id, |provider, root| provider.init(root)).await
}

#[tauri::command]
pub async fn writer_build(
    state: State<'_, AppState>,
    id: String,
) -> Result<CommandOutcome, String> {
    with_writer(state, id, |provider, root| provider.build(root)).await
}

#[tauri::command]
pub async fn writer_check(state: State<'_, AppState>, id: String) -> Result<WriterCheck, String> {
    with_writer(state, id, |provider, root| provider.check(root)).await
}

#[tauri::command]
pub async fn writer_lint(state: State<'_, AppState>, id: String) -> Result<CommandOutcome, String> {
    with_writer(state, id, |provider, root| provider.lint(root)).await
}

#[tauri::command]
pub async fn writer_impact(
    state: State<'_, AppState>,
    id: String,
    files: Vec<String>,
) -> Result<CommandOutcome, String> {
    with_writer(state, id, move |provider, root| {
        provider.impact(root, &files)
    })
    .await
}

#[tauri::command]
pub async fn writer_human(
    state: State<'_, AppState>,
    id: String,
) -> Result<CommandOutcome, String> {
    with_writer(state, id, |provider, root| provider.human(root)).await
}

#[tauri::command]
pub async fn writer_doctor(
    state: State<'_, AppState>,
    id: String,
) -> Result<CommandOutcome, String> {
    with_writer(state, id, |provider, root| provider.doctor(root)).await
}

#[tauri::command]
pub async fn writer_set_language_pack(
    state: State<'_, AppState>,
    id: String,
    enabled: bool,
    module: Option<String>,
) -> Result<WriterConfig, String> {
    with_writer(state, id, move |provider, root| {
        provider.set_language_pack(root, enabled, module.as_deref())
    })
    .await
}

#[tauri::command]
pub async fn writer_agent_doc(
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
    with_project_root(state, id, move |root| {
        let config = ancleto_code_core::providers::read_writer_config(Path::new(&root))
            .map_err(|error| error.to_string())?;
        std::fs::read_to_string(
            Path::new(&root)
                .join(config.output_dir)
                .join("agent")
                .join(&name),
        )
        .map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn writer_open_human(state: State<'_, AppState>, id: String) -> Result<(), String> {
    with_project_root(state, id, |root| {
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
    })
    .await
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
pub async fn appearance_save_image(
    state: State<'_, AppState>,
    extension: String,
    bytes: Vec<u8>,
) -> Result<String, String> {
    const ALLOWED: [&str; 5] = ["png", "jpg", "jpeg", "webp", "gif"];
    let extension = extension.trim_start_matches('.').to_ascii_lowercase();
    if !ALLOWED.contains(&extension.as_str()) {
        return Err(format!("extensión de imagen no permitida: {extension}"));
    }
    let config = state.config_dir.clone();
    run_blocking(move || {
        let directory = config.join("appearance");
        std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
        let path = directory.join(format!("{}.{extension}", uuid::Uuid::new_v4()));
        std::fs::write(&path, bytes).map_err(|error| error.to_string())?;
        Ok(path.to_string_lossy().into_owned())
    })
    .await
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
pub async fn project_search(
    state: State<'_, AppState>,
    id: String,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<FileEntry>, String> {
    let service = Arc::clone(&state.projects);
    run_blocking(move || {
        let service = service.lock().map_err(|_| lock_error())?;
        service
            .search(&id, &query, limit)
            .map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn git_status(state: State<'_, AppState>, id: String) -> Result<GitStatus, String> {
    let root = project_root(&state, &id)?;
    run_blocking(move || {
        ancleto_code_core::git::status(Path::new(&root)).map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn git_branches(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<GitBranch>, String> {
    let root = project_root(&state, &id)?;
    run_blocking(move || {
        ancleto_code_core::git::branches(Path::new(&root)).map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn git_create_branch(
    state: State<'_, AppState>,
    id: String,
    name: String,
    checkout: bool,
) -> Result<GitStatus, String> {
    let root = project_root(&state, &id)?;
    run_blocking(move || {
        ancleto_code_core::git::create_branch(Path::new(&root), &name, checkout)
            .map_err(|error| error.to_string())?;
        ancleto_code_core::git::status(Path::new(&root)).map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn git_checkout(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> Result<GitStatus, String> {
    let root = project_root(&state, &id)?;
    run_blocking(move || {
        ancleto_code_core::git::checkout(Path::new(&root), &name)
            .map_err(|error| error.to_string())?;
        ancleto_code_core::git::status(Path::new(&root)).map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn git_diff(
    state: State<'_, AppState>,
    id: String,
    path: Option<String>,
    staged: bool,
) -> Result<String, String> {
    let root = project_root(&state, &id)?;
    run_blocking(move || {
        ancleto_code_core::git::diff(Path::new(&root), path.as_deref(), staged)
            .map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn git_stage(
    state: State<'_, AppState>,
    id: String,
    paths: Vec<String>,
) -> Result<(), String> {
    let root = project_root(&state, &id)?;
    run_blocking(move || {
        ancleto_code_core::git::stage(Path::new(&root), &paths).map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn git_unstage(
    state: State<'_, AppState>,
    id: String,
    paths: Vec<String>,
) -> Result<(), String> {
    let root = project_root(&state, &id)?;
    run_blocking(move || {
        ancleto_code_core::git::unstage(Path::new(&root), &paths).map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn git_commit(
    state: State<'_, AppState>,
    id: String,
    message: String,
) -> Result<GitStatus, String> {
    let root = project_root(&state, &id)?;
    run_blocking(move || {
        ancleto_code_core::git::commit(Path::new(&root), &message)
            .map_err(|error| error.to_string())?;
        ancleto_code_core::git::status(Path::new(&root)).map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn git_worktrees(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<GitWorktree>, String> {
    let root = project_root(&state, &id)?;
    run_blocking(move || {
        ancleto_code_core::git::worktrees(Path::new(&root)).map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn git_worktree_add(
    state: State<'_, AppState>,
    id: String,
    branch: String,
) -> Result<GitWorktree, String> {
    let root = project_root(&state, &id)?;
    run_blocking(move || {
        ancleto_code_core::git::add_worktree(Path::new(&root), &branch)
            .map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn git_worktree_remove(
    state: State<'_, AppState>,
    id: String,
    path: String,
    force: bool,
) -> Result<(), String> {
    let root = project_root(&state, &id)?;
    run_blocking(move || {
        ancleto_code_core::git::remove_worktree(Path::new(&root), &path, force)
            .map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn git_fetch(state: State<'_, AppState>, id: String) -> Result<GitActionOutcome, String> {
    let root = project_root(&state, &id)?;
    run_blocking(move || {
        ancleto_code_core::git::fetch(Path::new(&root)).map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn git_pull(state: State<'_, AppState>, id: String) -> Result<GitActionOutcome, String> {
    let root = project_root(&state, &id)?;
    run_blocking(move || {
        ancleto_code_core::git::pull(Path::new(&root)).map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn git_push(state: State<'_, AppState>, id: String) -> Result<GitActionOutcome, String> {
    let root = project_root(&state, &id)?;
    run_blocking(move || {
        ancleto_code_core::git::push(Path::new(&root)).map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn git_merge(
    state: State<'_, AppState>,
    id: String,
    branch: String,
) -> Result<GitActionOutcome, String> {
    let root = project_root(&state, &id)?;
    run_blocking(move || {
        ancleto_code_core::git::merge(Path::new(&root), &branch).map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn task_list(state: State<'_, AppState>, id: String) -> Result<Vec<Task>, String> {
    let service = Arc::clone(&state.projects);
    run_blocking(move || {
        let service = service.lock().map_err(|_| lock_error())?;
        service.tasks(&id).map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub fn task_create(state: State<'_, AppState>, id: String, title: String) -> Result<Task, String> {
    let service = state.projects.lock().map_err(|_| lock_error())?;
    service
        .create_task(&id, &title)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn task_update(state: State<'_, AppState>, task: Task) -> Result<Task, String> {
    let service = state.projects.lock().map_err(|_| lock_error())?;
    service
        .update_task(&task)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn task_delete(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    let service = state.projects.lock().map_err(|_| lock_error())?;
    service.remove_task(&id).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn task_start(state: State<'_, AppState>, id: String) -> Result<Task, String> {
    let service = state.projects.lock().map_err(|_| lock_error())?;
    service.start_task(&id).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn git_remote(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<GitRemote>, String> {
    let root = project_root(&state, &id)?;
    run_blocking(move || {
        ancleto_code_core::git::remote(Path::new(&root)).map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn git_pull_requests(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<GitPullRequest>, String> {
    let root = project_root(&state, &id)?;
    run_blocking(move || {
        ancleto_code_core::git::pull_requests(Path::new(&root)).map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn git_issues(state: State<'_, AppState>, id: String) -> Result<Vec<GitIssue>, String> {
    let root = project_root(&state, &id)?;
    run_blocking(move || {
        ancleto_code_core::git::issues(Path::new(&root)).map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn usage_report(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    force: Option<bool>,
) -> Result<crate::usage::UsageSnapshot, String> {
    use crate::usage::UsageEvent;
    use tauri::Emitter;

    let service = Arc::clone(&state.usage);
    let snapshot = service.snapshot(crate::usage::now_secs());
    let should_refresh = force.unwrap_or(false) || snapshot.stale;
    if should_refresh && service.begin_refresh() {
        let handle = app.clone();
        let emit: crate::usage::Emitter = Arc::new(move |event: UsageEvent| {
            let name = match &event {
                UsageEvent::Provider { .. } => "usage://provider",
                UsageEvent::Done { .. } => "usage://done",
            };
            let _ = handle.emit(name, &event);
        });
        let worker = Arc::clone(&service);
        drop(tauri::async_runtime::spawn_blocking(move || {
            worker.refresh(emit)
        }));
    }
    Ok(service.snapshot(crate::usage::now_secs()))
}

#[tauri::command]
pub async fn driver_models(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<ancleto_code_core::DriverModel>, String> {
    let drivers = state.drivers.clone();
    run_blocking(move || {
        let binary = drivers
            .detect_all()
            .into_iter()
            .find(|driver| driver.id == id)
            .map(|driver| driver.binary_path.unwrap_or(driver.binary))
            .ok_or_else(|| "driver desconocido".to_owned())?;
        ancleto_code_core::models::list(&id, &binary).map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn writer_languages_status(
    state: State<'_, AppState>,
) -> Result<crate::writer_languages::WriterLanguagesStatus, String> {
    let config = state.config_dir.clone();
    run_blocking(move || Ok(crate::writer_languages::status(&config))).await
}

#[tauri::command]
pub async fn writer_languages_install(
    state: State<'_, AppState>,
) -> Result<crate::writer_languages::WriterLanguagesStatus, String> {
    let config = state.config_dir.clone();
    run_blocking(move || crate::writer_languages::install(&config)).await
}
