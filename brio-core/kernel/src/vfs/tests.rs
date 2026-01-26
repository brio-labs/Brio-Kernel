use super::manager::SessionManager;
use crate::infrastructure::config::SandboxSettings;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_session_lifecycle() -> anyhow::Result<()> {
    // Setup base directory
    let temp_dir = std::env::temp_dir().join("brio_tests");
    let base_dir = temp_dir.join("base");

    if base_dir.exists() {
        fs::remove_dir_all(&base_dir)?;
    }
    fs::create_dir_all(&base_dir)?;

    // Create initial files
    fs::write(base_dir.join("file1.txt"), "original")?;
    fs::create_dir(base_dir.join("subdir"))?;
    fs::write(base_dir.join("subdir/file2.txt"), "sub")?;

    // 1. Begin Session
    let mut manager = SessionManager::new(Default::default()).map_err(|e| anyhow::anyhow!(e))?;
    let session_id = manager
        .begin_session(
            base_dir
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid base dir"))?
                .to_string(),
        )
        .map_err(|e| anyhow::anyhow!(e))?;

    // Internal check
    let session_path = std::env::temp_dir().join("brio").join(&session_id);
    assert!(session_path.exists());
    assert_eq!(
        fs::read_to_string(session_path.join("file1.txt"))?,
        "original"
    );

    // 2. Modify Session
    fs::write(session_path.join("file1.txt"), "modified")?;
    fs::write(session_path.join("new.txt"), "created")?;
    fs::remove_file(session_path.join("subdir/file2.txt"))?;

    // 3. Commit Session
    manager
        .commit_session(session_id)
        .map_err(|e| anyhow::anyhow!(e))?;

    // 4. Verify Base
    assert_eq!(fs::read_to_string(base_dir.join("file1.txt"))?, "modified");
    assert_eq!(fs::read_to_string(base_dir.join("new.txt"))?, "created");
    // Verify deletion
    assert!(!base_dir.join("subdir/file2.txt").exists());

    // Cleanup
    let _ = fs::remove_dir_all(&base_dir);
    let _ = fs::remove_dir_all(&session_path);
    Ok(())
}

#[test]
fn test_begin_session_sandbox_violation() -> anyhow::Result<()> {
    // Setup a temp dir as our "allowed" root (though we won't put the target there)
    let temp_dir = tempdir()?;
    let allowed_path = temp_dir.path().join("allowed_project");
    fs::create_dir(&allowed_path)?;

    // Setup a target outside the allowed root
    let diff_path = temp_dir.path().join("forbidden_project");
    fs::create_dir(&diff_path)?;

    let sandbox = SandboxSettings {
        allowed_paths: vec![allowed_path.to_string_lossy().to_string()],
    };
    let mut manager = SessionManager::new(sandbox).map_err(|e| anyhow::anyhow!(e))?;

    // Attempt to start session on forbidden path
    let result = manager.begin_session(diff_path.to_string_lossy().to_string());

    assert!(result.is_err());
    let err = result
        .err()
        .ok_or_else(|| anyhow::anyhow!("Expected error"))?;
    assert!(err.contains("Security Violation"));
    Ok(())
}

#[test]
fn test_begin_session_sandbox_ok() -> anyhow::Result<()> {
    let temp_dir = tempdir()?;
    let allowed_path = temp_dir.path().join("allowed_project");
    fs::create_dir(&allowed_path)?;

    let sandbox = SandboxSettings {
        allowed_paths: vec![allowed_path.to_string_lossy().to_string()],
    };
    let mut manager = SessionManager::new(sandbox).map_err(|e| anyhow::anyhow!(e))?;

    let result = manager.begin_session(allowed_path.to_string_lossy().to_string());
    assert!(result.is_ok());
    Ok(())
}
