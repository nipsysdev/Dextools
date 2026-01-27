use crate::context::AppContext;
use crate::features::download::download_file_with_progress;
use crate::features::shared::map_storage_error;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn download_file_from_storage(
    cid: String,
    save_path: String,
    app_handle: AppHandle,
    context: State<'_, std::sync::Arc<AppContext>>,
) -> Result<crate::features::shared::DownloadResultResponse, String> {
    download_file_with_progress(cid, save_path.into(), app_handle, context.storage_manager())
        .await
        .map_err(map_storage_error)
}
