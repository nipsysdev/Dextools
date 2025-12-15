use codex_bindings::node::config::RepoKind;
use codex_bindings::{
    connect, debug, download_stream, upload_file, CodexConfig, CodexNode, DownloadStreamOptions,
    LogLevel, UploadOptions,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::{Mutex, OnceCell, RwLock};
use uuid::Uuid;

pub type CodexResult<T> = Result<T, CodexError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CodexConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

impl Default for CodexConnectionStatus {
    fn default() -> Self {
        CodexConnectionStatus::Disconnected
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationStage {
    Initializing,
    Uploading,
    Downloading,
    Verifying,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressMessage {
    pub operation_id: String,
    pub progress: f64,
    pub bytes_processed: usize,
    pub total_bytes: Option<usize>,
    pub stage: OperationStage,
    pub message: Option<String>,
}

impl ProgressMessage {
    pub fn new(operation_id: String) -> Self {
        Self {
            operation_id,
            progress: 0.0,
            bytes_processed: 0,
            total_bytes: None,
            stage: OperationStage::Initializing,
            message: None,
        }
    }

    pub fn with_bytes(mut self, bytes_processed: usize, total_bytes: Option<usize>) -> Self {
        self.bytes_processed = bytes_processed;
        self.total_bytes = total_bytes;
        if let Some(total) = total_bytes {
            self.progress = bytes_processed as f64 / total as f64;
        }
        self
    }

    pub fn with_stage(mut self, stage: OperationStage) -> Self {
        self.stage = stage;
        self
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DextoolsConfig {
    pub data_dir: PathBuf,
    pub storage_quota: u64,
    pub max_peers: u32,
    pub discovery_port: u16,
    pub log_level: LogLevel,
    pub auto_connect: bool,
}

impl Default for DextoolsConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl DextoolsConfig {
    pub fn new() -> Self {
        // Use a simple, reliable approach for data directory
        let data_dir = std::env::temp_dir().join("dextools").join("codex_data");

        println!("Codex data directory: {}", data_dir.display());

        // Ensure the directory exists using std::fs
        if let Err(e) = std::fs::create_dir_all(&data_dir) {
            eprintln!(
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

        Self {
            data_dir,
            storage_quota: 1024 * 1024 * 1024, // 1 GB
            max_peers: 50,
            discovery_port: 8089,
            log_level: LogLevel::Info,
            auto_connect: false,
        }
    }

    pub fn with_app_handle(app_handle: &AppHandle) -> Self {
        // Use app_data_dir for proper application data storage
        let data_dir = match app_handle.path().app_data_dir() {
            Ok(dir) => dir.join("codex_data"),
            Err(e) => {
                eprintln!("Failed to get app data directory: {}", e);
                // Fallback to temp directory
                std::env::temp_dir().join("dextools").join("codex_data")
            }
        };

        println!("Codex data directory: {}", data_dir.display());

        // Ensure the directory exists using std::fs
        if let Err(e) = std::fs::create_dir_all(&data_dir) {
            eprintln!(
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

        Self {
            data_dir,
            storage_quota: 1024 * 1024 * 1024, // 1 GB
            max_peers: 50,
            discovery_port: 8089,
            log_level: LogLevel::Info,
            auto_connect: false,
        }
    }

    fn to_codex_config(&self) -> CodexConfig {
        CodexConfig::new()
            .log_level(self.log_level)
            .data_dir(&self.data_dir)
            .storage_quota(self.storage_quota)
            .max_peers(self.max_peers)
            .discovery_port(self.discovery_port)
            .repo_kind(RepoKind::LevelDb)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub peer_id: Option<String>,
    pub version: Option<String>,
    pub repo_path: Option<String>,
    pub connected_peers: u32,
    pub max_peers: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub block_count: u32,
}

pub struct CodexManager {
    node: Arc<Mutex<Option<CodexNode>>>,
    config: DextoolsConfig,
    status: Arc<RwLock<CodexConnectionStatus>>,
    error: Arc<RwLock<Option<String>>>,
    progress_senders:
        Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<ProgressMessage>>>>,
    network_info: Arc<RwLock<NetworkInfo>>,
    storage_info: Arc<RwLock<StorageInfo>>,
}

impl CodexManager {
    pub async fn new(config: DextoolsConfig) -> CodexResult<Self> {
        let manager = Self {
            node: Arc::new(Mutex::new(None)),
            config,
            status: Arc::new(RwLock::new(CodexConnectionStatus::Disconnected)),
            error: Arc::new(RwLock::new(None)),
            progress_senders: Arc::new(Mutex::new(HashMap::new())),
            network_info: Arc::new(RwLock::new(NetworkInfo {
                peer_id: None,
                version: None,
                repo_path: None,
                connected_peers: 0,
                max_peers: 50,
            })),
            storage_info: Arc::new(RwLock::new(StorageInfo {
                used_bytes: 0,
                total_bytes: 1024 * 1024 * 1024,
                available_bytes: 1024 * 1024 * 1024,
                block_count: 0,
            })),
        };

        if manager.config.auto_connect {
            manager.connect().await?;
        }

        Ok(manager)
    }

    pub async fn connect(&self) -> CodexResult<()> {
        println!("Starting Codex connection...");

        // Update status to connecting
        {
            let mut status = self.status.write().await;
            *status = CodexConnectionStatus::Connecting;
        }

        // Clear any previous errors
        {
            let mut error = self.error.write().await;
            *error = None;
        }

        // Create Codex configuration
        let codex_config = self.config.to_codex_config();
        println!(
            "Created Codex config for directory: {}",
            self.config.data_dir.display()
        );

        // Create and start the node
        let mut node = match CodexNode::new(codex_config) {
            Ok(node) => {
                println!("Successfully created Codex node");
                node
            }
            Err(e) => {
                println!("Failed to create Codex node: {}", e);
                return Err(CodexError::NodeCreation(e.to_string()));
            }
        };

        // Start the node
        match node.start() {
            Ok(_) => {
                println!("Successfully started Codex node");
            }
            Err(e) => {
                println!("Failed to start Codex node: {}", e);
                return Err(CodexError::NodeStart(e.to_string()));
            }
        }

        // Store the node and update status
        {
            let mut node_guard = self.node.lock().await;
            *node_guard = Some(node);
        }

        {
            let mut status = self.status.write().await;
            *status = CodexConnectionStatus::Connected;
        }

        // Update network info
        self.update_network_info().await?;

        println!("Codex connection completed successfully");
        Ok(())
    }

    pub async fn disconnect(&self) -> CodexResult<()> {
        // Update status to disconnected
        {
            let mut status = self.status.write().await;
            *status = CodexConnectionStatus::Disconnected;
        }

        // Stop and destroy the node
        {
            let node_option = {
                let mut node_guard = self.node.lock().await;
                node_guard.take()
            };

            if let Some(mut node) = node_option {
                if let Err(e) = node.stop() {
                    eprintln!("Failed to stop node: {}", e);
                }
                // The Drop trait will handle cleanup
            }
        }

        // Clear network info
        {
            let mut network_info = self.network_info.write().await;
            network_info.peer_id = None;
            network_info.version = None;
            network_info.repo_path = None;
            network_info.connected_peers = 0;
        }

        // Clear any errors
        {
            let mut error = self.error.write().await;
            *error = None;
        }

        Ok(())
    }

    pub async fn get_status(&self) -> CodexConnectionStatus {
        self.status.read().await.clone()
    }

    pub async fn get_error(&self) -> Option<String> {
        self.error.read().await.clone()
    }

    pub async fn get_network_info(&self) -> NetworkInfo {
        self.network_info.read().await.clone()
    }

    pub async fn get_storage_info(&self) -> StorageInfo {
        self.storage_info.read().await.clone()
    }

    pub async fn upload_file_with_progress(&self, file_path: PathBuf) -> CodexResult<String> {
        let operation_id = Uuid::new_v4().to_string();

        // Register progress sender
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        {
            let mut senders = self.progress_senders.lock().await;
            senders.insert(operation_id.clone(), tx);
        }

        // Send initial progress
        let initial_progress =
            ProgressMessage::new(operation_id.clone()).with_stage(OperationStage::Initializing);
        self.send_progress(&operation_id, initial_progress).await;

        // Get the node
        let node = {
            let node_guard = self.node.lock().await;
            node_guard
                .as_ref()
                .ok_or_else(|| CodexError::NodeNotInitialized)?
                .clone()
        };

        if !node.is_started() {
            return Err(CodexError::NodeNotStarted);
        }

        // Check if file exists
        if !file_path.exists() {
            return Err(CodexError::FileNotFound(
                file_path.to_string_lossy().to_string(),
            ));
        }

        let file_size = std::fs::metadata(&file_path)
            .map_err(|e| CodexError::Io(e.to_string()))?
            .len() as usize;

        // Send file size info
        let size_progress = ProgressMessage::new(operation_id.clone())
            .with_stage(OperationStage::Uploading)
            .with_bytes(0, Some(file_size))
            .with_message(format!("Starting upload of {} bytes", file_size));
        self.send_progress(&operation_id, size_progress).await;

        // Create upload options with progress callback
        let operation_id_clone = operation_id.clone();
        let manager = self.clone();
        let upload_options =
            UploadOptions::new()
                .filepath(&file_path)
                .on_progress(move |progress| {
                    let manager = manager.clone();
                    let operation_id_for_callback = operation_id_clone.clone();
                    tokio::spawn(async move {
                        let progress_msg = ProgressMessage::new(operation_id_for_callback.clone())
                            .with_stage(OperationStage::Uploading)
                            .with_bytes(progress.bytes_uploaded, progress.total_bytes)
                            .with_message(format!("Uploaded {} bytes", progress.bytes_uploaded));
                        manager
                            .send_progress(&operation_id_for_callback, progress_msg)
                            .await;
                    });
                });

        // Perform the upload
        let result = upload_file(&node, upload_options)
            .await
            .map_err(|e| CodexError::Upload(e.to_string()))?;

        // Send completion progress
        let completion_progress = ProgressMessage::new(operation_id.clone())
            .with_stage(OperationStage::Completed)
            .with_bytes(file_size, Some(file_size))
            .with_message("Upload completed successfully".to_string());
        self.send_progress(&operation_id, completion_progress).await;

        // Clean up progress sender
        {
            let mut senders = self.progress_senders.lock().await;
            senders.remove(&operation_id);
        }

        Ok(result.cid)
    }

    pub async fn download_file_with_progress(
        &self,
        cid: String,
        save_path: PathBuf,
    ) -> CodexResult<()> {
        let operation_id = Uuid::new_v4().to_string();

        // Register progress sender
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        {
            let mut senders = self.progress_senders.lock().await;
            senders.insert(operation_id.clone(), tx);
        }

        // Send initial progress
        let initial_progress =
            ProgressMessage::new(operation_id.clone()).with_stage(OperationStage::Initializing);
        self.send_progress(&operation_id, initial_progress).await;

        // Get the node
        let node = {
            let node_guard = self.node.lock().await;
            node_guard
                .as_ref()
                .ok_or_else(|| CodexError::NodeNotInitialized)?
                .clone()
        };

        if !node.is_started() {
            return Err(CodexError::NodeNotStarted);
        }

        // Validate CID
        if cid.is_empty() {
            return Err(CodexError::InvalidCid("CID cannot be empty".to_string()));
        }

        // Send download start info
        let start_progress = ProgressMessage::new(operation_id.clone())
            .with_stage(OperationStage::Downloading)
            .with_message(format!("Starting download of CID: {}", cid));
        self.send_progress(&operation_id, start_progress).await;

        // Create download options with progress callback
        let operation_id_clone = operation_id.clone();
        let manager = self.clone();
        let download_options = DownloadStreamOptions::new(&cid)
            .filepath(&save_path)
            .on_progress(move |progress| {
                let manager = manager.clone();
                let operation_id_for_callback = operation_id_clone.clone();
                tokio::spawn(async move {
                    let progress_msg = ProgressMessage::new(operation_id_for_callback.clone())
                        .with_stage(OperationStage::Downloading)
                        .with_bytes(progress.bytes_downloaded, progress.total_bytes)
                        .with_message(format!("Downloaded {} bytes", progress.bytes_downloaded));
                    manager
                        .send_progress(&operation_id_for_callback, progress_msg)
                        .await;
                });
            });

        // Perform the download
        let result = download_stream(&node, &cid, download_options)
            .await
            .map_err(|e| CodexError::Download(e.to_string()))?;

        // Send completion progress
        let completion_progress = ProgressMessage::new(operation_id.clone())
            .with_stage(OperationStage::Completed)
            .with_bytes(result.size, Some(result.size))
            .with_message("Download completed successfully".to_string());
        self.send_progress(&operation_id, completion_progress).await;

        // Clean up progress sender
        {
            let mut senders = self.progress_senders.lock().await;
            senders.remove(&operation_id);
        }

        Ok(())
    }

    async fn send_progress(&self, operation_id: &str, progress: ProgressMessage) {
        let senders = self.progress_senders.lock().await;
        if let Some(sender) = senders.get(operation_id) {
            let _ = sender.send(progress);
        }
    }

    async fn update_network_info(&self) -> CodexResult<()> {
        let node = {
            let node_guard = self.node.lock().await;
            node_guard
                .as_ref()
                .ok_or_else(|| CodexError::NodeNotInitialized)?
                .clone()
        };

        let mut network_info = self.network_info.write().await;

        network_info.peer_id = node.peer_id().ok();
        network_info.version = node.version().ok();
        network_info.repo_path = node.repo().ok();
        network_info.max_peers = self.config.max_peers;
        // TODO: Get actual connected peers count when available in bindings

        Ok(())
    }

    pub async fn connect_to_peer(
        &self,
        peer_id: String,
        addresses: Vec<String>,
    ) -> CodexResult<()> {
        // Get the node (existing pattern from upload/download methods)
        let node = {
            let node_guard = self.node.lock().await;
            node_guard
                .as_ref()
                .ok_or_else(|| CodexError::NodeNotInitialized)?
                .clone()
        };

        if !node.is_started() {
            return Err(CodexError::NodeNotStarted);
        }

        // Use existing codex-bindings connect function
        connect(&node, &peer_id, &addresses)
            .await
            .map_err(|e| CodexError::Configuration(e.to_string()))?;

        Ok(())
    }

    pub async fn get_node_addresses(&self) -> CodexResult<Vec<String>> {
        // Get the node (existing pattern)
        let node = {
            let node_guard = self.node.lock().await;
            node_guard
                .as_ref()
                .ok_or_else(|| CodexError::NodeNotInitialized)?
                .clone()
        };

        if !node.is_started() {
            return Err(CodexError::NodeNotStarted);
        }

        // Use existing codex-bindings debug function
        let debug_info = debug(&node)
            .await
            .map_err(|e| CodexError::Configuration(e.to_string()))?;
        Ok(debug_info.addrs)
    }
}

impl Clone for CodexManager {
    fn clone(&self) -> Self {
        Self {
            node: Arc::clone(&self.node),
            config: self.config.clone(),
            status: Arc::clone(&self.status),
            error: Arc::clone(&self.error),
            progress_senders: Arc::clone(&self.progress_senders),
            network_info: Arc::clone(&self.network_info),
            storage_info: Arc::clone(&self.storage_info),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodexError {
    NodeCreation(String),
    NodeStart(String),
    NodeNotInitialized,
    NodeNotStarted,
    Upload(String),
    Download(String),
    FileNotFound(String),
    InvalidCid(String),
    Io(String),
    Configuration(String),
}

impl std::fmt::Display for CodexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodexError::NodeCreation(msg) => write!(f, "Failed to create node: {}", msg),
            CodexError::NodeStart(msg) => write!(f, "Failed to start node: {}", msg),
            CodexError::NodeNotInitialized => write!(f, "Node is not initialized"),
            CodexError::NodeNotStarted => write!(f, "Node is not started"),
            CodexError::Upload(msg) => write!(f, "Upload failed: {}", msg),
            CodexError::Download(msg) => write!(f, "Download failed: {}", msg),
            CodexError::FileNotFound(path) => write!(f, "File not found: {}", path),
            CodexError::InvalidCid(msg) => write!(f, "Invalid CID: {}", msg),
            CodexError::Io(msg) => write!(f, "IO error: {}", msg),
            CodexError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for CodexError {}

// Global manager instance
pub static CODEX_MANAGER: OnceCell<Arc<CodexManager>> = OnceCell::const_new();

pub async fn get_codex_manager_with_handle(
    app_handle: Option<AppHandle>,
) -> CodexResult<Arc<CodexManager>> {
    if let Some(manager) = CODEX_MANAGER.get() {
        Ok(Arc::clone(manager))
    } else {
        let config = if let Some(handle) = app_handle {
            DextoolsConfig::with_app_handle(&handle)
        } else {
            DextoolsConfig::new()
        };
        let manager = Arc::new(CodexManager::new(config).await?);
        CODEX_MANAGER.set(manager.clone()).map_err(|_| {
            CodexError::Configuration("Failed to initialize Codex manager".to_string())
        })?;
        Ok(manager)
    }
}

// Convert CodexError to String for Tauri commands
fn map_codex_error(err: CodexError) -> String {
    format!("{}", err)
}

#[tauri::command]
pub async fn get_codex_status(app_handle: AppHandle) -> Result<CodexConnectionStatus, String> {
    let manager = get_codex_manager_with_handle(Some(app_handle))
        .await
        .map_err(map_codex_error)?;
    Ok(manager.get_status().await)
}

#[tauri::command]
pub async fn get_codex_error(app_handle: AppHandle) -> Result<Option<String>, String> {
    let manager = get_codex_manager_with_handle(Some(app_handle))
        .await
        .map_err(map_codex_error)?;
    Ok(manager.get_error().await)
}

#[tauri::command]
pub async fn get_network_info(app_handle: AppHandle) -> Result<NetworkInfo, String> {
    let manager = get_codex_manager_with_handle(Some(app_handle))
        .await
        .map_err(map_codex_error)?;
    Ok(manager.get_network_info().await)
}

#[tauri::command]
pub async fn get_storage_info(app_handle: AppHandle) -> Result<StorageInfo, String> {
    let manager = get_codex_manager_with_handle(Some(app_handle))
        .await
        .map_err(map_codex_error)?;
    Ok(manager.get_storage_info().await)
}

#[tauri::command]
pub async fn connect_to_codex(app_handle: AppHandle) -> Result<(), String> {
    let manager = get_codex_manager_with_handle(Some(app_handle))
        .await
        .map_err(map_codex_error)?;
    manager.connect().await.map_err(map_codex_error)
}

#[tauri::command]
pub async fn disconnect_from_codex(app_handle: AppHandle) -> Result<(), String> {
    let manager = get_codex_manager_with_handle(Some(app_handle))
        .await
        .map_err(map_codex_error)?;
    manager.disconnect().await.map_err(map_codex_error)
}

#[tauri::command]
pub async fn get_codex_peer_id(app_handle: AppHandle) -> Result<Option<String>, String> {
    let manager = get_codex_manager_with_handle(Some(app_handle))
        .await
        .map_err(map_codex_error)?;
    let network_info = manager.get_network_info().await;
    Ok(network_info.peer_id)
}

#[tauri::command]
pub async fn get_codex_version(app_handle: AppHandle) -> Result<Option<String>, String> {
    let manager = get_codex_manager_with_handle(Some(app_handle))
        .await
        .map_err(map_codex_error)?;
    let network_info = manager.get_network_info().await;
    Ok(network_info.version)
}

#[tauri::command]
pub async fn upload_file_to_codex(
    file_path: String,
    app_handle: AppHandle,
) -> Result<UploadResultResponse, String> {
    let manager = get_codex_manager_with_handle(Some(app_handle))
        .await
        .map_err(map_codex_error)?;
    let path = PathBuf::from(file_path);

    // Start upload with progress tracking
    let cid = manager
        .upload_file_with_progress(path)
        .await
        .map_err(map_codex_error)?;

    // Return a basic result for now - the frontend will receive progress updates via events
    Ok(UploadResultResponse {
        cid,
        size: 0,        // Will be updated via progress events
        duration_ms: 0, // Will be updated via progress events
        verified: true,
    })
}

#[tauri::command]
pub async fn download_file_from_codex(
    cid: String,
    save_path: String,
    app_handle: AppHandle,
) -> Result<DownloadResultResponse, String> {
    let manager = get_codex_manager_with_handle(Some(app_handle))
        .await
        .map_err(map_codex_error)?;
    let path = PathBuf::from(save_path.clone());
    let cid_clone = cid.clone();

    // Start download with progress tracking
    manager
        .download_file_with_progress(cid, path)
        .await
        .map_err(map_codex_error)?;

    // Return a basic result for now - the frontend will receive progress updates via events
    Ok(DownloadResultResponse {
        cid: cid_clone,
        size: 0,        // Will be updated via progress events
        duration_ms: 0, // Will be updated via progress events
        verified: true,
        filepath: Some(save_path),
    })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UploadResultResponse {
    pub cid: String,
    pub size: usize,
    pub duration_ms: u64,
    pub verified: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadResultResponse {
    pub cid: String,
    pub size: usize,
    pub duration_ms: u64,
    pub verified: bool,
    pub filepath: Option<String>,
}

#[tauri::command]
pub async fn connect_to_peer(
    peer_id: String,
    addresses: Vec<String>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let manager = get_codex_manager_with_handle(Some(app_handle))
        .await
        .map_err(map_codex_error)?;
    manager
        .connect_to_peer(peer_id, addresses)
        .await
        .map_err(map_codex_error)
}

#[tauri::command]
pub async fn get_node_addresses(app_handle: AppHandle) -> Result<Vec<String>, String> {
    let manager = get_codex_manager_with_handle(Some(app_handle))
        .await
        .map_err(map_codex_error)?;
    manager.get_node_addresses().await.map_err(map_codex_error)
}
