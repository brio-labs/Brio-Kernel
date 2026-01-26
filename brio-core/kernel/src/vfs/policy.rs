use crate::infrastructure::config::SandboxSettings;
use std::path::{Path, PathBuf};

/// Enforces sandbox security policies on file paths.
pub struct SandboxPolicy {
    allowed_paths: Vec<PathBuf>,
}

impl SandboxPolicy {
    /// Creates a new policy from settings.
    pub fn new(settings: &SandboxSettings) -> Self {
        Self {
            allowed_paths: settings.allowed_paths.iter().map(PathBuf::from).collect(),
        }
    }

    /// Validates that the given path is within one of the allowed sandbox roots.
    pub fn validate_path(&self, target: &Path) -> Result<(), String> {
        if self.allowed_paths.is_empty() {
            return Ok(());
        }

        // Canonicalize target to resolve symlinks/.. etc
        let canonical_target = dunce::canonicalize(target)
            .map_err(|e| format!("Invalid path '{}': {}", target.display(), e))?;

        for allowed_path in &self.allowed_paths {
            let canonical_allowed = dunce::canonicalize(allowed_path).map_err(|e| {
                format!(
                    "Invalid allowed path configuration '{}': {}",
                    allowed_path.display(),
                    e
                )
            })?;

            if canonical_target.starts_with(&canonical_allowed) {
                return Ok(());
            }
        }

        Err(format!(
            "Security Violation: Path '{:?}' is outside the authorized sandbox roots.",
            canonical_target
        ))
    }
}
