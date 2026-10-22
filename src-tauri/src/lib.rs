use std::fs;
use std::sync::{Arc, Mutex};

use ancleto_code_core::{ProjectService, SessionEvent, SessionManager};
use serde::Serialize;
use tauri::{Emitter, Manager};

pub mod commands;
pub mod ide;

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
            let service = ProjectService::open(&config_directory.join("ancleto-code.sqlite"))?;
            let sessions = session_manager(app.handle().clone());
            app.manage(AppState {
                projects: Mutex::new(service),
                sessions,
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
            commands::session_list
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
