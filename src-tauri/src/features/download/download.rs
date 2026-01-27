use std::path::PathBuf;
use storage_bindings::{download_stream, DownloadStreamOptions};
use uuid::Uuid;

use crate::features::connection::get_storage_manager_with_handle;
use crate::features::shared::{
    DownloadResultResponse, OperationStage, ProgressMessage, StorageError,
};

pub async fn download_file_with_progress(
    cid: String,
    save_path: PathBuf,
    app_handle: tauri::AppHandle,
) -> Result<DownloadResultResponse, StorageError> {
    let manager = get_storage_manager_with_handle(Some(app_handle)).await?;

    let operation_id = Uuid::new_v4().to_string();
    let cid_clone = cid.clone();

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

    // Validate CID
    if cid.is_empty() {
        return Err(StorageError::InvalidCid("CID cannot be empty".to_string()));
    }

    // Send download start info
    let start_progress = ProgressMessage::new(operation_id.clone())
        .with_stage(OperationStage::Downloading)
        .with_message(format!("Starting download of CID: {}", cid));
    manager.send_progress(&operation_id, start_progress).await;

    // Create download options with progress callback
    let operation_id_clone = operation_id.clone();
    let manager_clone = manager.clone();
    let download_options = DownloadStreamOptions::new(&cid)
        .filepath(&save_path)
        .on_progress(move |progress| {
            let manager = manager_clone.clone();
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
        .map_err(|e| StorageError::Download(e.to_string()))?;

    // Send completion progress
    let completion_progress = ProgressMessage::new(operation_id.clone())
        .with_stage(OperationStage::Completed)
        .with_bytes(result.size, Some(result.size))
        .with_message("Download completed successfully".to_string());
    manager
        .send_progress(&operation_id, completion_progress)
        .await;

    // Clean up progress sender
    manager.unregister_progress_sender(&operation_id).await;

    Ok(DownloadResultResponse {
        cid: cid_clone,
        size: result.size,
        duration_ms: 0, // TODO: Track actual duration
        verified: true,
        filepath: Some(save_path.to_string_lossy().to_string()),
    })
}
