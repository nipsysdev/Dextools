use crate::context::AppContext;
use crate::features::shared::map_storage_error;
use crate::features::upload::upload_file_with_progress;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn upload_file_to_storage(
    file_path: String,
    app_handle: AppHandle,
    context: State<'_, std::sync::Arc<AppContext>>,
) -> Result<crate::features::shared::UploadResultResponse, String> {
    upload_file_with_progress(file_path.into(), app_handle, context.storage_manager())
        .await
        .map_err(map_storage_error)
}
