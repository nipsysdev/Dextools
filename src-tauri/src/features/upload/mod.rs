use std::path::PathBuf;
use std::sync::Arc;
use storage_bindings::{upload_file, UploadOptions};
use uuid::Uuid;

use crate::features::connection::StorageManager;
use crate::features::shared::{
    OperationStage, ProgressMessage, StorageError, UploadResultResponse,
};

pub mod commands;

pub async fn upload_file_with_progress(
    file_path: PathBuf,
    _app_handle: tauri::AppHandle,
    storage_manager: &Arc<StorageManager>,
) -> Result<UploadResultResponse, StorageError> {
    let manager = storage_manager;

    let operation_id = Uuid::new_v4().to_string();

    // Register progress sender
    let _rx = manager.register_progress_sender(operation_id.clone()).await;

    // Send initial progress
    let initial_progress =
        ProgressMessage::new(operation_id.clone()).with_stage(OperationStage::Initializing);
    manager.send_progress(&operation_id, initial_progress).await;

    // Get the node
    let node = manager.get_node().await?;

    if !node.is_started() {
        return Err(StorageError::NodeNotStarted);
    }

    // Check if file exists
    if !file_path.exists() {
        return Err(StorageError::FileNotFound(
            file_path.to_string_lossy().to_string(),
        ));
    }

    let file_size = std::fs::metadata(&file_path)
        .map_err(|e| StorageError::Io(e.to_string()))?
        .len() as usize;

    // Send file size info
    let size_progress = ProgressMessage::new(operation_id.clone())
        .with_stage(OperationStage::Uploading)
        .with_bytes(0, Some(file_size))
        .with_message(format!("Starting upload of {} bytes", file_size));
    manager.send_progress(&operation_id, size_progress).await;

    // Create upload options with progress callback
    let operation_id_clone = operation_id.clone();
    let manager_clone = manager.clone();
    let upload_options = UploadOptions::new()
        .filepath(&file_path)
        .on_progress(move |progress| {
            let manager = manager_clone.clone();
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
        .map_err(|e| StorageError::Upload(e.to_string()))?;

    // Send completion progress
    let completion_progress = ProgressMessage::new(operation_id.clone())
        .with_stage(OperationStage::Completed)
        .with_bytes(file_size, Some(file_size))
        .with_message("Upload completed successfully".to_string());
    manager
        .send_progress(&operation_id, completion_progress)
        .await;

    // Clean up progress sender
    manager.unregister_progress_sender(&operation_id).await;

    Ok(UploadResultResponse {
        cid: result.cid,
        size: file_size,
        duration_ms: 0, // TODO: Track actual duration
        verified: true,
    })
}

pub use commands::*;
