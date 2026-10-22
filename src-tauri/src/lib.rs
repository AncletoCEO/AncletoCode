use std::fs;
use std::sync::{Arc, Mutex};

use ancleto_code_core::{
    ConversationEvent, ConversationState, DriverRegistry, Orchestrator, ProjectService,
    SessionEvent, SessionManager, Store,
};
use serde::Serialize;
use tauri::{Emitter, Manager};

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

fn session_manager(handle: tauri::AppHandle) -> SessionManager {
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .setup(|app| {
            let config_directory = app.path().app_config_dir()?;
            fs::create_dir_all(&config_directory)?;
            let database = config_directory.join("ancleto-code.sqlite");
            let service = ProjectService::open(&database)?;
            let sessions = session_manager(app.handle().clone());
            let drivers = DriverRegistry::new();
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
                Store::open(&database)?,
                sessions.clone(),
                drivers.clone(),
                Some(sink),
            );
            app.manage(AppState {
                projects: Mutex::new(service),
                sessions,
                drivers,
                orchestrator,
                config_dir: config_directory,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::core_version,
            commands::project_add,
            commands::project_list,
            commands::project_remove,
            commands::project_open,
            commands::project_tree,
            commands::session_create,
            commands::session_write,
            commands::session_resize,
            commands::session_kill,
            commands::session_snapshot,
            commands::session_list,
            commands::drivers_list,
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
                let _ = state.sessions.kill_all();
            }
        }
    });
}
