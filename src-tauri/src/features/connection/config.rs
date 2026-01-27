use storage_bindings::node::config::RepoKind;
use storage_bindings::{LogLevel, StorageConfig};
use tauri::{AppHandle, Manager};

/// Creates a CodexConfig using the app handle for proper application data storage
pub fn create_storage_config(app_handle: &AppHandle) -> StorageConfig {
    // Use app_data_dir for proper application data storage
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory")
        .join("node_data");

    println!("Storage data directory: {}", data_dir.display());

    // Ensure the directory exists using std::fs
    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        panic!(
            "Failed to create data directory {}: {}",
            data_dir.display(),
            e
        );
    } else {
        println!(
            "Successfully created data directory: {}",
            data_dir.display()
        );
    }

    StorageConfig::new()
        .log_level(LogLevel::Debug)
        .data_dir(&data_dir)
        .storage_quota(1024 * 1024 * 1024) // 1 GB
        .max_peers(50)
        .discovery_port(8089)
        .repo_kind(RepoKind::LevelDb)
}
