use std::sync::Arc;
use tauri::AppHandle;

use crate::cli::CliArgs;
use crate::features::connection::{create_storage_config, StorageManager};
use crate::features::shared::StorageError;

/// Application context that holds all shared state
pub struct AppContext {
    pub storage_manager: Arc<StorageManager>,
    #[allow(dead_code)]
    pub cli_args: CliArgs,
}

impl AppContext {
    /// Create a new application context
    ///
    /// This initializes the storage manager with the given CLI arguments.
    /// This is a synchronous operation that will block until initialization completes.
    pub async fn new(cli_args: CliArgs, app_handle: &AppHandle) -> Result<Self, StorageError> {
        println!("Initializing AppContext with CLI args: {:?}", cli_args);

        // Create storage configuration
        let config = create_storage_config(app_handle, &cli_args);

        // Initialize storage manager
        let storage_manager = Arc::new(StorageManager::new(config).await?);

        println!("AppContext initialized successfully");

        Ok(Self {
            storage_manager,
            cli_args,
        })
    }

    /// Get a reference to the storage manager
    pub fn storage_manager(&self) -> &Arc<StorageManager> {
        &self.storage_manager
    }

    /// Get a reference to the CLI arguments
    #[allow(dead_code)]
    pub fn cli_args(&self) -> &CliArgs {
        &self.cli_args
    }
}
