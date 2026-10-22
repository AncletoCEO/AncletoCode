pub mod commands;
pub mod ide;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::core_version])
        .run(tauri::generate_context!())
        .expect("error mientras se ejecutaba AncletoCode");
}
