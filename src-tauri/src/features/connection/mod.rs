use std::collections::HashMap;
use std::sync::Arc;
use storage_bindings::{connect, debug, StorageNode};
use tokio::sync::{Mutex, RwLock};

use crate::features::shared::{NodeInfo, StorageConnectionStatus, StorageError};

pub mod commands;
pub mod config;

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

        let node = match StorageNode::new(self.config.clone()).await {
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

        let node = {
            let mut node_guard = self.node.lock().await;
            match node_guard.take() {
                Some(node) => node,
                None => {
                    // Node not initialized, initialize it first
                    drop(node_guard);
                    self.initialize_node().await?;
                    let mut node_guard = self.node.lock().await;
                    node_guard.take().ok_or(StorageError::NodeNotInitialized)?
                }
            }
        };

        match node.start().await {
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

            if let Some(node) = node_option {
                if let Err(e) = node.stop().await {
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
                .ok_or(StorageError::NodeNotInitialized)?
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
                .ok_or(StorageError::NodeNotInitialized)?
                .clone()
        };

        let peer_id = node.peer_id().await.ok();
        let version = node.version().await.ok();
        let repo_path = node.repo().await.ok();
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
            .ok_or(StorageError::NodeNotInitialized)
            .cloned()
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

pub use commands::*;
pub use config::*;
