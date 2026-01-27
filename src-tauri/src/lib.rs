mod cli;
mod context;
mod features;

use tauri::Manager;
use tauri_plugin_fs::FsExt;

#[cfg(desktop)]
use tauri_plugin_cli::CliExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(desktop)]
            {
                // Initialize CLI plugin
                app.handle().plugin(tauri_plugin_cli::init())?;

                // Get CLI matches using the CliExt trait
                let cli = app.handle().cli();
                let matches = cli.matches().unwrap();

                // Handle help flag
                if crate::cli::handle_cli_help(&matches) {
                    std::process::exit(0);
                }

                // Parse CLI arguments
                let cli_args = crate::cli::parse_cli_args(&matches);

                // Initialize AppContext synchronously
                // This will block until initialization completes
                let app_handle = app.handle().clone();
                let context = tauri::async_runtime::block_on(async {
                    crate::context::AppContext::new(cli_args, &app_handle).await
                });

                match context {
                    Ok(ctx) => {
                        println!("AppContext initialized successfully");
                        app.manage(std::sync::Arc::new(ctx));
                    }
                    Err(e) => {
                        eprintln!("Failed to initialize AppContext: {}", e);
                        return Err(Box::new(e));
                    }
                }
            }

            #[cfg(not(desktop))]
            {
                // For mobile, use default CLI args
                let cli_args = crate::cli::CliArgs::default();
                let app_handle = app.handle().clone();
                let context = tauri::async_runtime::block_on(async {
                    crate::context::AppContext::new(cli_args, &app_handle).await
                });

                match context {
                    Ok(ctx) => {
                        println!("AppContext initialized successfully");
                        app.manage(std::sync::Arc::new(ctx));
                    }
                    Err(e) => {
                        eprintln!("Failed to initialize AppContext: {}", e);
                        return Err(Box::new(e));
                    }
                }
            }

            let fs = app.fs_scope();

            if let Ok(app_data_dir) = app.path().app_data_dir() {
                let storage_dir = app_data_dir.join("storage_data");
                fs.allow_directory(&storage_dir, true)
                    .expect("Failed to allow Storage data directory");
                println!("Allowed Storage data directory: {}", storage_dir.display());
            }

            if let Ok(app_local_data_dir) = app.path().app_local_data_dir() {
                let storage_local_dir = app_local_data_dir.join("storage_data");
                fs.allow_directory(&storage_local_dir, true)
                    .expect("Failed to allow Storage local data directory");
                println!(
                    "Allowed Storage local data directory: {}",
                    storage_local_dir.display()
                );
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            features::connection::get_node_status,
            features::upload::upload_file_to_storage,
            features::download::download_file_from_storage,
            features::connection::connect_to_peer,
            features::connection::get_node_info,
            features::connection::start_node,
            features::connection::stop_node
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
