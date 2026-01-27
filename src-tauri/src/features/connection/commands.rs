use crate::context::AppContext;
use crate::features::shared::{map_storage_error, NodeInfo, StorageConnectionStatus};
use tauri::State;

#[tauri::command]
pub async fn get_node_status(
    context: State<'_, std::sync::Arc<AppContext>>,
) -> Result<StorageConnectionStatus, String> {
    Ok(context.storage_manager().get_status().await)
}

#[tauri::command]
pub async fn get_node_info(
    context: State<'_, std::sync::Arc<AppContext>>,
) -> Result<NodeInfo, String> {
    context
        .storage_manager()
        .get_node_info()
        .await
        .map_err(map_storage_error)
}

#[tauri::command]
pub async fn start_node(context: State<'_, std::sync::Arc<AppContext>>) -> Result<(), String> {
    context
        .storage_manager()
        .start_node()
        .await
        .map_err(map_storage_error)
}

#[tauri::command]
pub async fn stop_node(context: State<'_, std::sync::Arc<AppContext>>) -> Result<(), String> {
    context
        .storage_manager()
        .stop_node()
        .await
        .map_err(map_storage_error)
}

#[tauri::command]
pub async fn connect_to_peer(
    peer_id: String,
    addresses: Vec<String>,
    context: State<'_, std::sync::Arc<AppContext>>,
) -> Result<(), String> {
    context
        .storage_manager()
        .connect_to_peer(peer_id, addresses)
        .await
        .map_err(map_storage_error)
}
