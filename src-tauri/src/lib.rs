// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod codex;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            codex::get_codex_status,
            codex::get_codex_error,
            codex::get_network_info,
            codex::get_storage_info,
            codex::connect_to_codex,
            codex::disconnect_from_codex,
            codex::get_codex_peer_id,
            codex::get_codex_version,
            codex::upload_file_to_codex,
            codex::download_file_from_codex,
            codex::connect_to_peer,
            codex::get_node_addresses
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
