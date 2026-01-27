use std::collections::HashMap;
use std::sync::Arc;
use storage_bindings::{connect, debug, StorageNode};
use tokio::sync::{Mutex, OnceCell, RwLock};

use crate::features::shared::{NodeInfo, StorageConnectionStatus, StorageError};

pub struct StorageManager {
    node: Arc<Mutex<Option<StorageNode>>>,
    config: storage_bindings::StorageConfig,
    status: Arc<RwLock<StorageConnectionStatus>>,
    progress_senders: Arc<
        Mutex<
            HashMap<
                String,
                tokio::sync::mpsc::UnboundedSender<crate::features::shared::ProgressMessage>,
            >,
        >,
    >,
}

impl StorageManager {
    pub async fn new(config: storage_bindings::StorageConfig) -> Result<Self, StorageError> {
        let manager = Self {
            node: Arc::new(Mutex::new(None)),
            config,
            status: Arc::new(RwLock::new(StorageConnectionStatus::Disconnected)),
            progress_senders: Arc::new(Mutex::new(HashMap::new())),
        };

        manager.initialize_node().await?;

        Ok(manager)
    }

    pub async fn initialize_node(&self) -> Result<(), StorageError> {
        {
            let mut status = self.status.write().await;
            *status = StorageConnectionStatus::Connecting;
        }

        {
            let node_guard = self.node.lock().await;
            if node_guard.is_some() {
                // Node already initialized, just update status
                let mut status = self.status.write().await;
                *status = StorageConnectionStatus::Initialized;
                return Ok(());
            }
        }

        let node = match StorageNode::new(self.config.clone()) {
            Ok(node) => node,
            Err(e) => {
                return Err(StorageError::NodeCreation(e.to_string()));
            }
        };

        {
            let mut node_guard = self.node.lock().await;
            *node_guard = Some(node);
        }

        {
            let mut status = self.status.write().await;
            *status = StorageConnectionStatus::Initialized;
        }

        Ok(())
    }

    pub async fn start_node(&self) -> Result<(), StorageError> {
        {
            let mut status = self.status.write().await;
            *status = StorageConnectionStatus::Connecting;
        }

        let mut node = {
            let mut node_guard = self.node.lock().await;
            match node_guard.take() {
                Some(node) => node,
                None => {
                    // Node not initialized, initialize it first
                    drop(node_guard);
                    self.initialize_node().await?;
                    let mut node_guard = self.node.lock().await;
                    node_guard
                        .take()
                        .ok_or_else(|| StorageError::NodeNotInitialized)?
                }
            }
        };

        match node.start() {
            Ok(_) => {}
            Err(e) => {
                let mut node_guard = self.node.lock().await;
                *node_guard = Some(node);
                return Err(StorageError::NodeStart(e.to_string()));
            }
        }

        {
            let mut node_guard = self.node.lock().await;
            *node_guard = Some(node);
        }

        {
            let mut status = self.status.write().await;
            *status = StorageConnectionStatus::Connected;
        }

        Ok(())
    }

    pub async fn stop_node(&self) -> Result<(), StorageError> {
        {
            let mut status = self.status.write().await;
            *status = StorageConnectionStatus::Disconnected;
        }

        {
            let node_option = {
                let mut node_guard = self.node.lock().await;
                node_guard.take()
            };

            if let Some(mut node) = node_option {
                if let Err(e) = node.stop() {
                    eprintln!("Failed to stop node: {}", e);
                }
                // Put the stopped node back
                let mut node_guard = self.node.lock().await;
                *node_guard = Some(node);
            }
        }

        {
            let mut status = self.status.write().await;
            *status = StorageConnectionStatus::Initialized;
        }

        Ok(())
    }

    pub async fn get_status(&self) -> StorageConnectionStatus {
        self.status.read().await.clone()
    }

    pub async fn connect_to_peer(
        &self,
        peer_id: String,
        addresses: Vec<String>,
    ) -> Result<(), StorageError> {
        // Get the node (existing pattern from upload/download methods)
        let node = {
            let node_guard = self.node.lock().await;
            node_guard
                .as_ref()
                .ok_or_else(|| StorageError::NodeNotInitialized)?
                .clone()
        };

        if !node.is_started() {
            return Err(StorageError::NodeNotStarted);
        }

        connect(&node, &peer_id, &addresses)
            .await
            .map_err(|e| StorageError::Configuration(e.to_string()))?;

        Ok(())
    }

    pub async fn get_node_info(&self) -> Result<NodeInfo, StorageError> {
        let node = {
            let node_guard = self.node.lock().await;
            node_guard
                .as_ref()
                .ok_or_else(|| StorageError::NodeNotInitialized)?
                .clone()
        };

        let peer_id = node.peer_id().ok();
        let version = node.version().ok();
        let repo_path = node.repo().ok();
        let mut debug_info = Option::None;

        if node.is_started() {
            match debug(&node).await {
                Ok(info) => debug_info = Some(info),
                Err(e) => {
                    return Err(StorageError::Configuration(e.to_string()));
                }
            }
        }

        Ok(NodeInfo {
            peer_id,
            version,
            repo_path,
            debug_info,
        })
    }

    // Helper methods for upload/download features
    pub async fn get_node(&self) -> Result<StorageNode, StorageError> {
        let node_guard = self.node.lock().await;
        node_guard
            .as_ref()
            .ok_or_else(|| StorageError::NodeNotInitialized)
            .map(|node| node.clone())
    }

    pub async fn send_progress(
        &self,
        operation_id: &str,
        progress: crate::features::shared::ProgressMessage,
    ) {
        let senders = self.progress_senders.lock().await;
        if let Some(sender) = senders.get(operation_id) {
            let _ = sender.send(progress);
        }
    }

    pub async fn register_progress_sender(
        &self,
        operation_id: String,
    ) -> tokio::sync::mpsc::UnboundedReceiver<crate::features::shared::ProgressMessage> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        {
            let mut senders = self.progress_senders.lock().await;
            senders.insert(operation_id, tx);
        }
        rx
    }

    pub async fn unregister_progress_sender(&self, operation_id: &str) {
        let mut senders = self.progress_senders.lock().await;
        senders.remove(operation_id);
    }
}

impl Clone for StorageManager {
    fn clone(&self) -> Self {
        Self {
            node: Arc::clone(&self.node),
            config: self.config.clone(),
            status: Arc::clone(&self.status),
            progress_senders: Arc::clone(&self.progress_senders),
        }
    }
}

// Global manager instance
pub static STORAGE_MANAGER: OnceCell<Arc<StorageManager>> = OnceCell::const_new();

pub async fn get_storage_manager_with_handle(
    app_handle: Option<tauri::AppHandle>,
) -> Result<Arc<StorageManager>, StorageError> {
    if let Some(manager) = STORAGE_MANAGER.get() {
        Ok(Arc::clone(manager))
    } else {
        let config = if let Some(handle) = app_handle {
            crate::features::connection::create_storage_config(&handle)
        } else {
            return Err(StorageError::Configuration(
                "App handle is required to create storage manager".to_string(),
            ));
        };
        let manager = Arc::new(StorageManager::new(config).await?);
        STORAGE_MANAGER.set(manager.clone()).map_err(|_| {
            StorageError::Configuration("Failed to initialize Storage manager".to_string())
        })?;
        Ok(manager)
    }
}
