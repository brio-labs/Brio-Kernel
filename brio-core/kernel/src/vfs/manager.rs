use super::{diff, reflink};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, instrument};
use uuid::Uuid;

pub struct SessionManager {
    // Map SessionID -> BasePath
    sessions: HashMap<String, PathBuf>,
    root_temp_dir: PathBuf,
}

impl SessionManager {
    pub fn new() -> Self {
        // Use standard temp dir or default to /tmp/brio
        let temp = std::env::temp_dir().join("brio");
        Self {
            sessions: HashMap::new(),
            root_temp_dir: temp,
        }
    }

    /// Creates a new session by copying (reflink) the base directory.
    #[instrument(skip(self))]
    pub fn begin_session(&mut self, base_path: String) -> Result<String, String> {
        let base = PathBuf::from(&base_path);
        if !base.exists() {
            return Err(format!("Base path does not exist: {}", base_path));
        }

        let session_id = Uuid::new_v4().to_string();
        let session_path = self.root_temp_dir.join(&session_id);

        info!("Starting session {} for base {:?}", session_id, base);

        // Perform Reflink Copy
        reflink::copy_dir_reflink(&base, &session_path)
            .map_err(|e| format!("Failed to create session copy: {}", e))?;

        // Store session mapping
        self.sessions.insert(session_id.clone(), base);

        Ok(session_id)
    }

    /// Commits changes from the session back to the base directory.
    #[instrument(skip(self))]
    pub fn commit_session(&mut self, session_id: String) -> Result<(), String> {
        let base_path = self
            .sessions
            .get(&session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;

        let session_path = self.root_temp_dir.join(&session_id);

        if !session_path.exists() {
            return Err(format!("Session directory lost: {:?}", session_path));
        }

        info!("Committing session {} to {:?}", session_id, base_path);

        // 1. Compute Diff
        let changes = diff::compute_diff(&session_path, base_path)
            .map_err(|e| format!("Failed to compute diff: {}", e))?;

        if changes.is_empty() {
            info!("No changes to commit for session {}", session_id);
            return Ok(());
        }

        // 2. Apply Changes
        diff::apply_changes(&session_path, base_path, &changes)
            .map_err(|e| format!("Failed to apply changes: {}", e))?;

        // 3. Cleanup
        Ok(())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
