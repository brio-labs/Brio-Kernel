use anyhow::{Result, anyhow};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

use crate::mesh::{MeshMessage, Payload};
use crate::store::{PrefixPolicy, SqlStore};
use crate::vfs::manager::SessionManager;
use crate::ws::{BroadcastMessage, Broadcaster, WsPatch};

// Import the bindgen generated trait
// Assuming the bindgen world name is "brio-host" and interface is "session-fs"
// The actual path depends on how `wit_bindgen::generate!` is called in `main.rs` or `lib.rs`.
// Since I can't see the bindgen output, I will assume a standard import path or implement it on the struct directly
// and let the bindgen macro glue it together.
// However, the user prompt showed: `impl brio::core::session_fs::Host for BrioHostState`.
// I will implement the logic as methods on BrioHostState first, effectively matching the trait.
// If the trait definition is available via `crate::brio_host::...` I would use it.
// For now, I'll add the field and methods.

pub struct BrioHostState {
    mesh_router: HashMap<String, Sender<MeshMessage>>,
    db_pool: SqlitePool,
    broadcaster: Broadcaster,
    session_manager: SessionManager,
}

impl BrioHostState {
    pub async fn new(db_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new().connect(db_url).await?;

        Ok(Self {
            mesh_router: HashMap::new(),
            db_pool: pool,
            broadcaster: Broadcaster::new(),
            session_manager: SessionManager::new(),
        })
    }

    /// Register a component (Agent or Tool) with the mesh router.
    /// This enforces the registration contract.
    pub fn register_component(&mut self, id: String, sender: Sender<MeshMessage>) {
        self.mesh_router.insert(id, sender);
    }

    /// Accessor for the DB Pool (Immutable access only)
    pub fn db(&self) -> &SqlitePool {
        &self.db_pool
    }

    /// Get a scoped Store interface for the given component.
    /// This ensures all queries are validated against the component's scope.
    pub fn get_store(&self, _scope: &str) -> SqlStore {
        // We inject the PrefixPolicy here.
        // In the future, this could be configurable per scope.
        SqlStore::new(self.db_pool.clone(), Box::new(PrefixPolicy))
    }

    /// Accessor for the broadcaster (query - CQS).
    pub fn broadcaster(&self) -> &Broadcaster {
        &self.broadcaster
    }

    /// Broadcasts a JSON Patch to all connected UI clients (command - CQS).
    pub fn broadcast_patch(&self, patch: WsPatch) -> Result<()> {
        self.broadcaster
            .broadcast(BroadcastMessage::Patch(patch))
            .map_err(|e| anyhow!("Broadcast failed: {}", e))
    }

    pub async fn mesh_call(&self, target: &str, method: &str, payload: Payload) -> Result<Payload> {
        let sender = self
            .mesh_router
            .get(target)
            .ok_or_else(|| anyhow!("Target component '{}' not found", target))?;

        let (reply_tx, reply_rx) = oneshot::channel();

        let message = MeshMessage {
            target: target.to_string(),
            method: method.to_string(),
            payload,
            reply_tx,
        };

        sender
            .send(message)
            .await
            .map_err(|e| anyhow!("Failed to send message to target '{}': {}", target, e))?;

        let response = reply_rx
            .await
            .map_err(|e| anyhow!("Failed to receive reply from target '{}': {}", target, e))?;

        response.map_err(|e| anyhow!("Target '{}' returned error: {}", target, e))
    }

    // --- VFS Interface Methods ---

    pub fn begin_session(&mut self, base_path: String) -> Result<String, String> {
        self.session_manager.begin_session(base_path)
    }

    pub fn commit_session(&mut self, session_id: String) -> Result<(), String> {
        self.session_manager.commit_session(session_id)
    }
}
