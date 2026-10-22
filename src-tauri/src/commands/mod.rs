#[tauri::command]
pub fn core_version() -> &'static str {
    ancleto_code_core::version()
}
