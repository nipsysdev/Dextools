use crate::cli::CliArgs;
use storage_bindings::node::config::RepoKind;
use storage_bindings::{LogLevel, StorageConfig};
use tauri::{AppHandle, Manager};

/// Creates a CodexConfig using the app handle for proper application data storage
pub fn create_storage_config(app_handle: &AppHandle, cli_args: &CliArgs) -> StorageConfig {
    // Use custom data dir from CLI or default
    let data_dir = if let Some(custom_dir) = &cli_args.data_dir {
        std::path::PathBuf::from(custom_dir)
    } else {
        app_handle
            .path()
            .app_data_dir()
            .expect("Failed to get app data directory")
            .join("node_data")
    };

    // Use custom port from CLI or default
    let discovery_port = cli_args.port.unwrap_or(8089);

    println!("Storage data directory: {}", data_dir.display());
    println!("Discovery port: {}", discovery_port);

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
        .discovery_port(discovery_port)
        .repo_kind(RepoKind::LevelDb)
}
