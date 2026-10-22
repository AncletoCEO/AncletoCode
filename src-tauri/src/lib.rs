use std::fs;
use std::sync::Mutex;

use ancleto_code_core::ProjectService;
use tauri::Manager;

pub mod commands;
pub mod ide;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let config_directory = app.path().app_config_dir()?;
            fs::create_dir_all(&config_directory)?;
            let service = ProjectService::open(&config_directory.join("ancleto-code.sqlite"))?;
            app.manage(AppState {
                projects: Mutex::new(service),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::core_version,
            commands::project_add,
            commands::project_list,
            commands::project_remove,
            commands::project_open,
            commands::project_tree
        ])
        .run(tauri::generate_context!())
        .expect("error mientras se ejecutaba AncletoCode");
}
