use std::fs;
use std::sync::{Arc, Mutex};

use ancleto_code_core::{
    ConversationEvent, ConversationState, DriverRegistry, HostClient, HostPaths, Orchestrator,
    ProjectService, SessionBackend, SessionEvent, SessionManager, Store,
};
use serde::Serialize;
use tauri::{Emitter, Manager};

pub mod ancleto_runtime;
pub mod commands;

use commands::AppState;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionOutputPayload {
    data: Vec<u8>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionExitPayload {
    exit_code: Option<i32>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConversationStatePayload {
    state: ConversationState,
    reason: Option<String>,
}

fn local_sessions(handle: tauri::AppHandle) -> SessionManager {
    SessionManager::new(Some(Arc::new(move |event| {
        let result = match event {
            SessionEvent::Output { id, data } => handle.emit(
                &format!("session://output/{id}"),
                SessionOutputPayload { data },
            ),
            SessionEvent::State { id, status } => {
                handle.emit(&format!("session://state/{id}"), status)
            }
            SessionEvent::Exit { id, exit_code } => handle.emit(
                &format!("session://exit/{id}"),
                SessionExitPayload { exit_code },
            ),
        };

        if let Err(error) = result {
            eprintln!("no se pudo emitir un evento de sesión: {error}");
        }
    })))
}

fn forward_host_events(handle: tauri::AppHandle, client: Arc<HostClient>) {
    let events = client.events();
    std::thread::spawn(move || {
        while let Ok(notification) = events.recv() {
            let Some(id) = notification
                .params
                .get("id")
                .and_then(serde_json::Value::as_str)
            else {
                continue;
            };
            let result = match notification.method.as_str() {
                "session.output" => {
                    let data = notification
                        .params
                        .get("data")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    match serde_json::from_value::<Vec<u8>>(data) {
                        Ok(data) => handle.emit(
                            &format!("session://output/{id}"),
                            SessionOutputPayload { data },
                        ),
                        Err(_) => Ok(()),
                    }
                }
                "session.state" => {
                    let status = notification
                        .params
                        .get("status")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null);
                    handle.emit(&format!("session://state/{id}"), status)
                }
                "session.exit" => {
                    let exit_code = notification
                        .params
                        .get("exitCode")
                        .and_then(serde_json::Value::as_i64)
                        .and_then(|code| i32::try_from(code).ok());
                    handle.emit(
                        &format!("session://exit/{id}"),
                        SessionExitPayload { exit_code },
                    )
                }
                _ => Ok(()),
            };
            if let Err(error) = result {
                eprintln!("no se pudo reenviar un evento del host: {error}");
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .setup(|app| {
            let config_directory = app.path().app_config_dir()?;
            fs::create_dir_all(&config_directory)?;
            let resource_directory = app.path().resource_dir()?;
            ancleto_runtime::configure(&config_directory, &resource_directory);
            let database = ancleto_code_core::host::transport::default_database_path();
            let service = ProjectService::open(&database)?;
            let settings_store = Store::open(&database)?;
            let drivers = DriverRegistry::from_store(&settings_store);
            let paths = HostPaths::discover(None);
            let (host, sessions): (Option<Arc<HostClient>>, Arc<dyn SessionBackend>) =
                match HostClient::connect_or_start(&paths, &database) {
                    Ok(client) => {
                        let client = Arc::new(client);
                        forward_host_events(app.handle().clone(), Arc::clone(&client));
                        (Some(Arc::clone(&client)), client as Arc<dyn SessionBackend>)
                    }
                    Err(error) => {
                        eprintln!(
                            "ancleto-code: host no disponible ({error}); sesiones in-process"
                        );
                        let manager = local_sessions(app.handle().clone());
                        (None, Arc::new(manager))
                    }
                };
            let handle = app.handle().clone();
            let sink: ancleto_code_core::orch::ConversationSink = Arc::new(move |event| {
                let result = match event {
                    ConversationEvent::Turn {
                        conversation_id,
                        turn,
                    } => handle.emit(&format!("conversation://turn/{conversation_id}"), turn),
                    ConversationEvent::State {
                        conversation_id,
                        state,
                        reason,
                    } => handle.emit(
                        &format!("conversation://state/{conversation_id}"),
                        ConversationStatePayload { state, reason },
                    ),
                };
                if let Err(error) = result {
                    eprintln!("no se pudo emitir un evento de conversación: {error}");
                }
            });
            let orchestrator = Orchestrator::new(
                settings_store,
                sessions.clone(),
                drivers.clone(),
                Some(sink),
            );
            app.manage(AppState {
                projects: Mutex::new(service),
                sessions,
                host,
                drivers,
                orchestrator,
                config_dir: config_directory,
                resource_dir: resource_directory,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ancleto_runtime_status,
            commands::ancleto_runtime_update,
            commands::core_version,
            commands::project_add,
            commands::project_list,
            commands::project_remove,
            commands::project_open,
            commands::project_tree,
            commands::task_list,
            commands::task_create,
            commands::task_update,
            commands::task_delete,
            commands::task_start,
            commands::project_search,
            commands::session_create,
            commands::session_write,
            commands::session_resize,
            commands::session_kill,
            commands::session_snapshot,
            commands::session_list,
            commands::session_saved,
            commands::drivers_list,
            commands::git_status,
            commands::git_branches,
            commands::git_create_branch,
            commands::git_checkout,
            commands::git_diff,
            commands::git_stage,
            commands::git_unstage,
            commands::git_commit,
            commands::git_fetch,
            commands::git_pull,
            commands::git_push,
            commands::git_merge,
            commands::git_remote,
            commands::git_worktrees,
            commands::git_worktree_add,
            commands::git_worktree_remove,
            commands::ide_open_antigravity,
            commands::ide_sync_antigravity,
            commands::conversation_start,
            commands::conversation_pause,
            commands::conversation_resume,
            commands::conversation_abort,
            commands::conversation_get,
            commands::conversation_turns,
            commands::conversation_list,
            commands::spec_status,
            commands::spec_init,
            commands::spec_install,
            commands::spec_update,
            commands::spec_upgrade,
            commands::spec_check,
            commands::spec_doctor,
            commands::spec_specs_check,
            commands::spec_discovery_check,
            commands::spec_discovery_generate,
            commands::spec_memory_list,
            commands::spec_memory_context,
            commands::writer_status,
            commands::writer_init,
            commands::writer_build,
            commands::writer_check,
            commands::writer_lint,
            commands::writer_impact,
            commands::writer_human,
            commands::writer_doctor,
            commands::writer_set_language_pack,
            commands::writer_agent_doc,
            commands::writer_open_human,
            commands::settings_get,
            commands::settings_put,
            commands::appearance_save_image,
            commands::appearance_native_effect
        ])
        .build(tauri::generate_context!())
        .expect("error mientras se construía AncletoCode");

    app.run(|app_handle, event| {
        if matches!(event, tauri::RunEvent::Exit) {
            if let Some(state) = app_handle.try_state::<AppState>() {
                if state.host.is_none() {
                    let _ = state.sessions.kill_all_sessions();
                }
            }
        }
    });
}
