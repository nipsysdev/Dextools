use crate::cli::CliArgs;
use storage_bindings::node::config::RepoKind;
use storage_bindings::{LogLevel, StorageConfig};
use tauri::{AppHandle, Manager};

/// Bootstrap node SPR URIs for the Storage network
const BOOTSTRAP_NODES: &[&str] = &[
    "spr:CiUIAhIhAiJvIcA_ZwPZ9ugVKDbmqwhJZaig5zKyLiuaicRcCGqLEgIDARo8CicAJQgCEiECIm8hwD9nA9n26BUoNuarCEllqKDnMrIuK5qJxFwIaosQ3d6esAYaCwoJBJ_f8zKRAnU6KkYwRAIgM0MvWNJL296kJ9gWvfatfmVvT-A7O2s8Mxp8l9c8EW0CIC-h-H-jBVSgFjg3Eny2u33qF7BDnWFzo7fGfZ7_qc9P",
    "spr:CiUIAhIhAlNJ7ary8eOK5GcwQ6q4U8brR7iWjwhMwzHb8BzzmCEDEgIDARpJCicAJQgCEiECU0ntqvLx44rkZzBDqrhTxutHuJaPCEzDMdvwHPOYIQMQsZ67vgYaCwoJBK6Kf1-RAnVEGgsKCQSuin9fkQJ1RCpGMEQCIDxd6lXDvj1PcHgQYnNpHGfgCO5a7fejg3WhSjh2wTimAiB7YHsL1WZYU_zkHcNDWhRgMbkb3C5yRuvUhjBjGOYJYQ",
    "spr:CiUIAhIhAyUvcPkKoGE7-gh84RmKIPHJPdsX5Ugm_IHVJgF-Mmu_EgIDARo8CicAJQgCEiEDJS9w-QqgYTv6CHzhGYog8ck92xflSCb8gdUmAX4ya78QoemesAYaCwoJBES39Q2RAnVOKkYwRAIgLi3rouyaZFS_Uilx8k99ySdQCP1tsmLR21tDb9p8LcgCIG30o5YnEooQ1n6tgm9fCT7s53k6XlxyeSkD_uIO9mb3",
];

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

    let mut config = StorageConfig::new()
        .log_level(LogLevel::Debug)
        .data_dir(&data_dir)
        .storage_quota(1024 * 1024 * 1024) // 1 GB
        .max_peers(50)
        .discovery_port(discovery_port)
        .repo_kind(RepoKind::LevelDb);

    // Add bootstrap nodes for automatic network connection
    for node in BOOTSTRAP_NODES {
        config = config.add_bootstrap_node(*node);
    }

    println!("Added {} bootstrap nodes to configuration", BOOTSTRAP_NODES.len());

    config
}
